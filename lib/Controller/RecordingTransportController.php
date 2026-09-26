<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\ArtifactManifestStore;
use OCA\PoRe\Service\NextcloudArtifactConnector;
use OCA\PoRe\Service\RecordingRuntimeService;
use OCA\PoRe\Service\TalkSessionAccessService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\Attribute\PublicPage;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use RuntimeException;

final class RecordingTransportController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly NextcloudArtifactConnector $connector,
		private readonly ArtifactManifestStore $artifactManifestStore,
		private readonly RecordingRuntimeService $runtime,
		private readonly TalkSessionAccessService $talkSessionAccess,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[PublicPage]
	#[NoAdminRequired]
	public function prepareFinalizedArtifact(
		string $production_id,
		string $production_label,
		string $recording_id,
		string $capture_id,
		string $started_at,
		string $participant_label,
		int $size,
		string $payload_sha256,
		string $recording_session_id,
		string $capture_provenance = '{}',
	): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($production_id)['actor_id'];
		} catch (RuntimeException) {
			return $this->rejected('talk_context_unauthorized', 403);
		}

		try {
			$this->authorizeRecordingTransport($actorId, $production_id, $recording_id);
			$provenance = $this->decodeProvenance($capture_provenance);
			$prepared = $this->connector->prepare(
				$this->required($production_id, 'production_id'),
				$this->required($production_label, 'production_label'),
				$this->required($recording_id, 'recording_id'),
				$this->required($capture_id, 'capture_id'),
				$this->required($started_at, 'started_at'),
				$participant_label,
				$size,
				$this->required($payload_sha256, 'payload_sha256'),
				$actorId,
			);
			try {
				$this->artifactManifestStore->stagePreparedArtifact([
					'artifact_id' => $capture_id,
					'production_id' => $production_id,
					'production_label' => $production_label,
					'recording_id' => $recording_id,
					'recording_session_id' => $this->required($recording_session_id, 'recording_session_id'),
					'participant_label' => $participant_label,
					'filename' => $prepared['filename'] ?? null,
					'size' => $prepared['size'] ?? $size,
					'payload_sha256' => $prepared['sha256'] ?? $payload_sha256,
					'capture_provenance' => $provenance,
				]);
			} catch (\Throwable $exception) {
				try { $this->connector->close($prepared['transfer_id'], $actorId); } catch (\Throwable) {
					// The transport handle may already have been absent/expired.
				}
				throw $exception;
			}
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'prepared',
				...$prepared,
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			if ($exception->getMessage() === 'PoRE transport authorization is not available.') return $this->rejected('runtime_unavailable', 503);
			if ($exception->getMessage() === 'PoRE transport authorization is not permitted for this recording') return $this->rejected('transport_unauthorized', 403);
			if ($exception->getMessage() === 'artifact_provenance_invalid') return $this->rejected('artifact_provenance_invalid', 400);
			if ($exception->getMessage() === 'artifact_manifest_conflict') return $this->rejected('artifact_manifest_conflict', 409);
			return $this->rejected();
		}
	}

	#[PublicPage]
	#[NoAdminRequired]
	public function verifyFinalizedArtifact(string $transfer_id, string $session_id = ''): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($session_id)['actor_id'];
			$receipt = $this->connector->verify($this->required($transfer_id, 'transfer_id'), $actorId);
			$record = $this->artifactManifestStore->persistVerifiedArtifact($receipt);
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'verified',
				...$receipt,
				'manifest_hash' => $record['manifest_hash'],
				'artifact_record_status' => 'verified',
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			if ($exception->getMessage() === 'artifact_manifest_conflict') return $this->rejected('artifact_manifest_conflict', 409);
			if ($exception->getMessage() === 'artifact_manifest_storage_unavailable') return $this->rejected('artifact_manifest_storage_unavailable', 503);
			if ($exception->getMessage() === 'artifact_preservation_invalid') return $this->rejected('artifact_preservation_invalid', 422);
			return $this->rejected();
		}
	}

	#[PublicPage]
	#[NoAdminRequired]
	public function closeFinalizedArtifactTransfer(string $transfer_id, string $session_id = ''): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($session_id)['actor_id'];
			$this->connector->close($this->required($transfer_id, 'transfer_id'), $actorId);
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'closed',
				'error_code' => null,
			]);
		} catch (\Throwable) {
			return $this->rejected();
		}
	}


	private function decodeProvenance(string $json): array {
		try {
			$decoded = json_decode($json === '' ? '{}' : $json, true, 512, JSON_THROW_ON_ERROR);
		} catch (\JsonException) {
			throw new RuntimeException('artifact_provenance_invalid');
		}
		if (!is_array($decoded)) throw new RuntimeException('artifact_provenance_invalid');
		return $decoded;
	}

	private function required(string $value, string $name): string {
		if (trim($value) === '') throw new RuntimeException(sprintf('%s is required.', $name));
		return $value;
	}

	private function authorizeRecordingTransport(string $actorId, string $productionId, string $recordingId): void {
		try {
			$response = $this->runtime->command([
				'request_id' => bin2hex(random_bytes(16)),
				'session_id' => $productionId,
				'actor_id' => $actorId,
				'recording_id' => $recordingId,
				'command' => ['Snapshot' => null],
			], 'recording.command');
		} catch (\Throwable $exception) {
			throw new RuntimeException('PoRE transport authorization is not available.', 0, $exception);
		}
		if (($response['status'] ?? null) !== 'ok' || !is_array($response['state'] ?? null) || !in_array($response['state']['role'] ?? null, ['host', 'participant'], true)) {
			throw new RuntimeException('PoRE transport authorization is not permitted for this recording');
		}
	}

	private function rejected(string $errorCode = 'nextcloud_transport_failed', int $status = 500): DataResponse {
		return new DataResponse([
			'protocol_version' => 2,
			'status' => 'rejected',
			'error_code' => $errorCode,
		], $status);
	}
}
