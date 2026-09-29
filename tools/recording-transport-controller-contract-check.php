<?php

declare(strict_types=1);

namespace OCA\PoRe\AppInfo {
	final class Application {
		public const APP_ID = 'pore';
	}
}

namespace OCA\PoRe\Service {
	class ArtifactManifestStore {
		public function __construct() {}
		public function get(string $artifactId): ?array { return null; }
		public function stagePreparedArtifact(array $submission): array { return []; }
		public function removeIfVerifiedRemoteFileIdMatches(string $artifactId, int $expectedFileId): bool { return false; }
	}
	class NextcloudArtifactConnector {
		public function prepare(...$args): array { return []; }
		public function close(string $handle, string $actorUserId): void {}
		public function verify(string $handle, string $actorUserId): array { return []; }
	}
	class RecordingRuntimeService {
		public function command(array $command, string $name): array { return ['status' => 'ok', 'state' => ['role' => 'host']]; }
	}
	class TalkSessionAccessService {
		public function resolve(string $sessionId): array { return ['actor_id' => 'actor-1']; }
	}
}

namespace OCP\AppFramework\Http {
	class DataResponse {
		public function __construct(public readonly array $data, public readonly int $status = 200) {}
	}
}

namespace OCP\AppFramework\Http\Attribute {
	#[\Attribute(\Attribute::TARGET_METHOD)]
	class PublicPage {}

	#[\Attribute(\Attribute::TARGET_METHOD)]
	class NoAdminRequired {}
}

namespace OCP\AppFramework {
	class OCSController {
		public function __construct(string $appName, object $request) {}
	}
}

namespace OCP {
	class IRequest {}
}

namespace {
	require_once __DIR__ . '/../lib/Controller/RecordingTransportController.php';

	use OCA\PoRe\Controller\RecordingTransportController;
	use OCA\PoRe\Service\ArtifactManifestStore;
	use OCA\PoRe\Service\NextcloudArtifactConnector;
	use OCA\PoRe\Service\RecordingRuntimeService;
	use OCA\PoRe\Service\TalkSessionAccessService;
	use OCP\IRequest;

	final class FakeStore extends ArtifactManifestStore {
		public ?array $existing = null;
		public array $staged = [];
		public array $removedIds = [];
		public function get(string $artifactId): ?array { return $this->existing; }
		public function stagePreparedArtifact(array $submission): array {
			$this->staged[] = $submission;
			return $this->stageQueue !== [] ? array_shift($this->stageQueue) : $this->nextStage;
		}
		public array $stageQueue = [];
		public array $nextStage = [];
		public function removeIfVerifiedRemoteFileIdMatches(string $artifactId, int $expectedFileId): bool {
			$this->removedIds[] = [$artifactId, $expectedFileId];
			return $this->removeResult;
		}
		public bool $removeResult = true;
	}

	final class FakeConnector extends NextcloudArtifactConnector {
		public array $preparedQueue = [];
		public array $prepareCalls = [];
		public array $closed = [];
		public ?\Throwable $prepareException = null;
		public ?\Throwable $verifyException = null;
		public function verify(string $handle, string $actorUserId): array {
			if ($this->verifyException !== null) throw $this->verifyException;
			return [];
		}
		public function prepare(...$args): array {
			$this->prepareCalls[] = $args;
			if ($this->prepareException !== null) throw $this->prepareException;
			if ($this->preparedQueue === []) throw new \RuntimeException('no prepared result');
			return array_shift($this->preparedQueue);
		}
		public function close(string $handle, string $actorUserId): void { $this->closed[] = [$handle, $actorUserId]; }
	}

	function check(bool $condition, string $message): void {
		if (!$condition) throw new \RuntimeException($message);
	}

	function controller(FakeStore $store, FakeConnector $connector): RecordingTransportController {
		return new RecordingTransportController(
			new IRequest(),
			$connector,
			$store,
			new RecordingRuntimeService(),
			new TalkSessionAccessService(),
		);
	}

	function runPrepare(RecordingTransportController $controller, string $payloadFormat = 'audio/wav', ?string $canonicalFilenameHint = null): \OCP\AppFramework\Http\DataResponse {
		return $controller->prepareFinalizedArtifact(
			'production-1',
			'Interview',
			'recording-1',
			'capture-1',
			'2026-09-27T09:00:00+00:00',
			'Host',
			44,
			str_repeat('a', 64),
			'session-1',
			'{"schemaVersion":1,"capture":null,"sourceSegments":[]}',
			$payloadFormat,
			$canonicalFilenameHint,
		);
	}

	// TEST-01: If the previously verified File-ID is gone, the fresh upload
	// handle must survive the stale-record replacement path.
	$store = new FakeStore();
	$store->existing = [
		'status' => 'verified',
		'artifact_id' => 'capture-1',
		'production_id' => 'production-1',
		'recording_id' => 'recording-1',
		'recording_session_id' => 'session-1',
		'participant_label' => 'Host',
		'remote' => ['file_id' => 17],
	];
	$store->stageQueue = [
		['status' => 'verified', 'remote' => ['file_id' => 17]],
		['status' => 'pending_verification'],
	];
	$connector = new FakeConnector();
	$connector->preparedQueue[] = [
		'transfer_id' => 'replacement-handle',
		'upload_url' => '/public.php/dav/files/share',
		'upload_username' => 'anonymous',
		'upload_password' => 'secret',
		'filename' => 'Host (2).wav',
		'size' => 44,
		'sha256' => str_repeat('a', 64),
		'upload_required' => true,
		'target_user_id' => 'owner',
	];

	$response = runPrepare(controller($store, $connector));
	check($response->data['status'] === 'prepared', 'Stale-record replacement must remain a successful prepare.');
	check($response->data['transfer_id'] === 'replacement-handle', 'Fresh replacement handle must be returned to the browser.');
	check($connector->closed === [], 'Fresh replacement handle must not be closed before upload.');
	check($store->removedIds === [['capture-1', 17]], 'Stale verified record must be conditionally removed by its original File-ID.');

	// TEST-02: If another request has already established a different verified
	// File-ID, the stale request must close its provisional handle and bind to
	// the winner instead of deleting the winner.
	$store = new FakeStore();
	$store->existing = [
		'status' => 'verified',
		'artifact_id' => 'capture-1',
		'production_id' => 'production-1',
		'recording_id' => 'recording-1',
		'recording_session_id' => 'session-1',
		'participant_label' => 'Host',
		'remote' => ['file_id' => 17],
	];
	$store->stageQueue = [
		['status' => 'verified', 'remote' => ['file_id' => 22]],
		['status' => 'verified', 'remote' => ['file_id' => 22]],
	];
	$connector = new FakeConnector();
	$connector->preparedQueue[] = [
		'transfer_id' => 'stale-provisional',
		'upload_url' => '/public.php/dav/files/share',
		'upload_username' => 'anonymous',
		'upload_password' => 'secret',
		'filename' => 'Host (3).wav',
		'size' => 44,
		'sha256' => str_repeat('a', 64),
		'upload_required' => true,
		'target_user_id' => 'owner',
	];
	$connector->preparedQueue[] = [
		'transfer_id' => 'winner-handle',
		'upload_url' => '',
		'upload_username' => '',
		'upload_password' => '',
		'filename' => 'Host (2).wav',
		'size' => 44,
		'sha256' => str_repeat('a', 64),
		'upload_required' => false,
		'target_user_id' => 'owner',
	];

	$response = runPrepare(controller($store, $connector));
	check($response->data['transfer_id'] === 'winner-handle', 'Concurrent winner File-ID must become the transport handle.');
	check($response->data['upload_required'] === false, 'Concurrent winner must not trigger a duplicate upload.');
	check($connector->closed === [['stale-provisional', 'actor-1']], 'Stale provisional handle must be closed.');
	check($store->removedIds === [], 'A newer verified File-ID must never be removed by the stale request.');

	// TEST-03: Invalid remote preservation is a client-side verification error.
	$store = new FakeStore();
	$connector = new FakeConnector();
	$connector->verifyException = new \RuntimeException('Nextcloud transport artifact is not a supported PoRE PCM WAV.');
	$response = controller($store, $connector)->verifyFinalizedArtifact('transport-handle', 'session-1');
	check($response->status === 422, 'Invalid V1 preservation must map to HTTP 422.');
	check($response->data['error_code'] === 'artifact_preservation_invalid', 'Invalid V1 preservation must use the artifact preservation error code.');

	// TEST-FLAC-01: The controller must pass the explicit transport format
	// through the prepare boundary and expose canonical naming separately.
	$store = new FakeStore();
	$store->stageQueue = [['status' => 'pending_verification']];
	$connector = new FakeConnector();
	$connector->preparedQueue[] = [
		'transfer_id' => 'flac-handle',
		'upload_url' => '/public.php/dav/files/flac-share',
		'upload_username' => 'anonymous',
		'upload_password' => 'secret',
		'filename' => 'Host.flac',
		'canonical_filename' => 'Host.wav',
		'payload_format' => 'audio/flac',
		'size' => 51,
		'sha256' => str_repeat('b', 64),
		'upload_required' => true,
		'target_user_id' => 'owner',
	];
	$response = runPrepare(controller($store, $connector), 'audio/flac');
	check($response->data['payload_format'] === 'audio/flac', 'Controller response must expose the requested FLAC transport format.');
	check($response->data['canonical_filename'] === 'Host.wav', 'Controller response must expose the canonical WAV filename.');
	check(($connector->prepareCalls[0][10] ?? null) === 'audio/flac', 'Controller must pass the explicit FLAC payload format to the connector.');
	$store = new FakeStore();
	$store->stageQueue = [['status' => 'pending_verification']];
	$connector = new FakeConnector();
	$connector->preparedQueue[] = ['transfer_id' => 'flac-hint-handle', 'upload_url' => '/public.php/dav/files/flac-hint-share', 'upload_username' => 'anonymous', 'upload_password' => 'secret', 'filename' => 'Host (2).flac', 'canonical_filename' => 'Host (2).wav', 'payload_format' => 'audio/flac', 'size' => 51, 'sha256' => str_repeat('b', 64), 'upload_required' => true, 'target_user_id' => 'owner'];
	$response = runPrepare(controller($store, $connector), 'audio/flac', 'Host (2).wav');
	$lastPrepare = $connector->prepareCalls[count($connector->prepareCalls) - 1] ?? [];
	check(($lastPrepare[11] ?? null) === 'Host (2).wav', 'Controller must pass a canonical filename recovery hint to the connector.');

	// TEST-FLAC-02: Unsupported transport formats are client errors, not generic server failures.
	$store = new FakeStore();
	$connector = new FakeConnector();
	$connector->prepareException = new \RuntimeException('artifact_payload_format_invalid');
	$response = runPrepare(controller($store, $connector), 'audio/ogg');
	check($response->status === 400, 'Unsupported transport format must map to HTTP 400.');
	check($response->data['error_code'] === 'artifact_payload_format_invalid', 'Unsupported transport format must preserve its error code.');

	echo "Recording transport controller contract checks passed.\n";
}
