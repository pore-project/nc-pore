<?php

declare(strict_types=1);

namespace OCP {
	class IConfig {
		public function getSystemValueString(string $key): string { return ''; }
		public function getSystemValue(string $key, mixed $default = null): mixed { return $default; }
	}
}

namespace {
	require_once __DIR__ . '/../lib/Service/ArtifactManifestStore.php';

	use OCA\PoRe\Service\ArtifactManifestStore;
	use OCP\IConfig;

	final class FakeArtifactManifestConfig extends IConfig {
		public function __construct(private readonly string $dataDirectory, private readonly string $instanceId) {}
		public function getSystemValue(string $key, mixed $default = null): mixed {
			return match ($key) {
				'datadirectory' => $this->dataDirectory,
				'instanceid' => $this->instanceId,
				default => $default,
			};
		}
	}

	function check(bool $condition, string $message): void {
		if (!$condition) throw new \RuntimeException($message);
	}

	$directory = sys_get_temp_dir() . '/nc-pore-artifact-manifest-' . bin2hex(random_bytes(6));
	$store = new ArtifactManifestStore(new FakeArtifactManifestConfig($directory, 'instance-1'));

	$provenance = [
		'schemaVersion' => 1,
		'capture' => [
			'startedAt' => '2026-09-26T12:00:00.000Z',
			'sampleRate' => 48000,
			'sampleSize' => 24,
			'channelCount' => 1,
			'processing' => [
				'echoCancellation' => false,
				'noiseSuppression' => false,
				'autoGainControl' => false,
			],
		],
		'sourceSegments' => [],
	];

	$store->stagePreparedArtifact([
		'artifact_id' => 'capture-1',
		'production_id' => 'production-1',
		'production_label' => 'Interview',
		'recording_id' => 'recording-1',
		'recording_session_id' => 'session-1',
		'participant_label' => 'Host',
		'filename' => 'Host.wav',
		'size' => 47,
		'payload_sha256' => hash('sha256', 'payload'),
		'capture_provenance' => $provenance,
	]);

	$pending = $store->get('capture-1');
	check(($pending['status'] ?? null) === 'pending_verification', 'Capture metadata must remain pending before verification.');
	check($pending['capture']['sampleRate'] === 48000, 'Capture sample rate must be stored.');
	check($pending['capture']['sampleSize'] === 24, 'Capture sample size must be stored.');
	check($pending['capture']['processing']['noiseSuppression'] === false, 'Capture processing provenance must be stored.');

	$receipt = $store->persistVerifiedArtifact([
		'artifact_id' => 'capture-1',
		'file_id' => 42,
		'path' => 'audio/Interview/Host.wav',
		'filename' => 'Host.wav',
		'size' => 47,
		'sha256' => hash('sha256', 'payload'),
		'preservation' => [
			'format' => 'audio/wav',
			'encoding' => 'pcm_s24le',
			'sampleRate' => 48000,
			'channels' => 1,
			'bitsPerSample' => 24,
		],
	]);

	check(($receipt['status'] ?? null) === 'verified', 'Verified artifact record must be marked verified.');
	check(strlen((string)$receipt['manifest_hash']) === 64, 'Server manifest hash must be a SHA-256 hex digest.');

	$again = $store->persistVerifiedArtifact([
		'artifact_id' => 'capture-1',
		'file_id' => 42,
		'path' => 'audio/Interview/Host.wav',
		'filename' => 'Host.wav',
		'size' => 47,
		'sha256' => hash('sha256', 'payload'),
		'preservation' => $receipt['preservation'],
	]);
	check($again['manifest_hash'] === $receipt['manifest_hash'], 'Repeated verification must be idempotent.');

	try {
		$store->persistVerifiedArtifact([
			'artifact_id' => 'capture-1',
			'file_id' => 42,
			'path' => 'audio/Interview/Host.wav',
			'filename' => 'Host.wav',
			'size' => 48,
			'sha256' => hash('sha256', 'different'),
			'preservation' => $receipt['preservation'],
		]);
		throw new \RuntimeException('Expected a manifest conflict.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'artifact_manifest_conflict', 'Conflicting verification must be rejected.');
	}

	$artifactPath = $directory . '/appdata_instance-1/pore/artifacts/' . hash('sha256', 'capture-1') . '.json';
	check(is_file($artifactPath), 'Artifact manifest must be stored in private appdata.');
	$storedJson = json_decode((string)file_get_contents($artifactPath), true);
	check(($storedJson['manifest_hash'] ?? null) === $receipt['manifest_hash'], 'Persisted manifest hash must match the returned record.');

	@unlink($artifactPath);
	@unlink($artifactPath . '.lock');
	@rmdir(dirname($artifactPath));
	@rmdir(dirname(dirname($artifactPath)));
	@rmdir(dirname(dirname(dirname($artifactPath))));

	echo "Artifact manifest store contract checks passed.\n";
}
