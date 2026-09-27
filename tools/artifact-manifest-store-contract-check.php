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
		'target_user_id' => 'owner',
		'filename' => 'Host.wav',
		'size' => 47,
		'payload_sha256' => hash('sha256', 'payload'),
		'capture_provenance' => $provenance,
	]);

	$pending = $store->get('capture-1');
	check(($pending['status'] ?? null) === 'pending_verification', 'Capture metadata must remain pending before verification.');
	check($pending['remote']['target_user_id'] === 'owner', 'Target storage owner must remain in the server-side record.');
	check($pending['capture']['sampleRate'] === 48000, 'Capture sample rate must be stored.');
	check($pending['capture']['sampleSize'] === 24, 'Capture sample size must be stored.');
	check($pending['capture']['processing']['noiseSuppression'] === false, 'Capture processing provenance must be stored.');

	$receipt = $store->persistVerifiedArtifact([
		'artifact_id' => 'capture-1',
		'target_user_id' => 'owner',
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
		'target_user_id' => 'owner',
		'file_id' => 42,
		'path' => 'audio/Interview/Host.wav',
		'filename' => 'Host.wav',
		'size' => 47,
		'sha256' => hash('sha256', 'payload'),
		'preservation' => $receipt['preservation'],
	]);
	check($again['manifest_hash'] === $receipt['manifest_hash'], 'Repeated verification must be idempotent.');

	// TEST-01: Mutable remote locator changes update the record but do not
	// change the immutable manifest hash.
	$relocated = $store->persistVerifiedArtifact([
		'artifact_id' => 'capture-1',
		'target_user_id' => 'owner',
		'file_id' => 99,
		'path' => 'audio/Interview/Renamed/Host-renamed.wav',
		'filename' => 'Host-renamed.wav',
		'size' => 47,
		'sha256' => hash('sha256', 'payload'),
		'preservation' => $receipt['preservation'],
	]);
	check($relocated['manifest_hash'] === $receipt['manifest_hash'], 'Remote filename, path and file id must not change the immutable manifest hash.');
	check($relocated['remote']['file_id'] === 99, 'Verified record must refresh the current remote file id.');
	check($relocated['remote']['path'] === 'audio/Interview/Renamed/Host-renamed.wav', 'Verified record must refresh the current remote path.');
	check($relocated['remote']['filename'] === 'Host-renamed.wav', 'Verified record must refresh the current remote filename.');

	// TEST-02: A conditional reconciliation must never remove a record that
	// was changed to point at another remote File-ID in the meantime.
	check($store->removeIfVerifiedRemoteFileIdMatches('capture-1', 42) === false, 'Stale reconciliation must not remove a record with a different File-ID.');
	check($store->get('capture-1') !== null, 'Stale reconciliation must preserve the current record.');
	check($store->removeIfVerifiedRemoteFileIdMatches('capture-1', 99) === true, 'Matching reconciliation must remove the verified record.');
	check($store->get('capture-1') === null, 'Matching reconciliation must remove the record.');

	$store->stagePreparedArtifact([
		'artifact_id' => 'capture-1',
		'production_id' => 'production-1',
		'production_label' => 'Interview',
		'recording_id' => 'recording-1',
		'recording_session_id' => 'session-1',
		'participant_label' => 'Host',
		'target_user_id' => 'owner',
		'filename' => 'Host-renamed.wav',
		'size' => 47,
		'payload_sha256' => hash('sha256', 'payload'),
		'capture_provenance' => $provenance,
	]);

	// TEST-03: A repeated Prepare with changed provenance for the same Artifact
	// is a conflict, rather than a silent rewrite.
	$changedProvenance = $provenance;
	$changedProvenance['capture']['sampleRate'] = 44100;
	try {
		$store->stagePreparedArtifact([
			'artifact_id' => 'capture-1',
			'production_id' => 'production-1',
			'production_label' => 'Interview',
			'recording_id' => 'recording-1',
			'recording_session_id' => 'session-1',
			'participant_label' => 'Host',
			'target_user_id' => 'owner',
			'filename' => 'Host-other.wav',
			'size' => 47,
			'payload_sha256' => hash('sha256', 'payload'),
			'capture_provenance' => $changedProvenance,
		]);
		throw new \RuntimeException('Expected a provenance conflict.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'artifact_manifest_conflict', 'Changed provenance must be rejected.');
	}

	try {
		$store->stagePreparedArtifact([
			'artifact_id' => 'capture-1',
			'production_id' => 'production-1',
			'production_label' => 'Interview',
			'recording_id' => 'recording-1',
			'recording_session_id' => 'session-1',
			'participant_label' => 'Guest',
			'target_user_id' => 'owner',
			'filename' => 'Host.wav',
			'size' => 47,
			'payload_sha256' => hash('sha256', 'payload'),
			'capture_provenance' => $provenance,
		]);
		throw new \RuntimeException('Expected a participant conflict.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'artifact_manifest_conflict', 'Changed participant identity must be rejected.');
	}

	// TEST-04: Verify cannot create a canonical record without the prepare
	// context that carries the authoritative production/recording/provenance data.
	try {
		$store->persistVerifiedArtifact([
			'artifact_id' => 'never-prepared',
			'file_id' => 7,
			'size' => 47,
			'sha256' => hash('sha256', 'payload'),
			'preservation' => $receipt['preservation'],
		]);
		throw new \RuntimeException('Expected missing prepare context.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'artifact_manifest_context_missing', 'Verify without prepare context must be rejected.');
	}

	$pendingRecordId = 'pending-expired';
	$store->stagePreparedArtifact([
		'artifact_id' => $pendingRecordId,
		'production_id' => 'production-1',
		'production_label' => 'Interview',
		'recording_id' => 'recording-1',
		'recording_session_id' => 'session-1',
		'participant_label' => 'Guest',
		'target_user_id' => 'owner',
		'filename' => 'Guest.wav',
		'size' => 47,
		'payload_sha256' => hash('sha256', 'payload'),
		'capture_provenance' => $provenance,
	]);
	$pendingPath = $directory . '/appdata_instance-1/pore/artifacts/' . hash('sha256', $pendingRecordId) . '.json';
	$pendingStored = json_decode((string)file_get_contents($pendingPath), true);
	$pendingStored['prepared_at'] = '2026-09-26T12:30:00+00:00';
	file_put_contents($pendingPath, json_encode($pendingStored) . "\n");

	$store->stagePreparedArtifact([
		'artifact_id' => $pendingRecordId,
		'production_id' => 'production-1',
		'production_label' => 'Interview',
		'recording_id' => 'recording-1',
		'recording_session_id' => 'session-1',
		'participant_label' => 'Guest',
		'target_user_id' => 'owner',
		'filename' => 'Guest.wav',
		'size' => 47,
		'payload_sha256' => hash('sha256', 'payload'),
		'capture_provenance' => $provenance,
	]);

	// TEST-05: A renewed prepare refreshes neither its technical identity nor
	// the retention clock; cleanup uses the current record under the lock.
	check($store->cleanupExpiredPending(new \DateTimeImmutable('2026-09-26T14:00:00+00:00')) === 0, 'A renewed pending record must not be removed as stale.');

	@unlink($pendingPath);
	@unlink($pendingPath . '.lock');
	$artifactPath = $directory . '/appdata_instance-1/pore/artifacts/' . hash('sha256', 'capture-1') . '.json';
	@unlink($artifactPath);
	@unlink($artifactPath . '.lock');
	@rmdir(dirname($artifactPath));
	@rmdir(dirname(dirname($artifactPath)));
	@rmdir(dirname(dirname(dirname($artifactPath))));

	echo "Artifact manifest store contract checks passed.\n";
}
