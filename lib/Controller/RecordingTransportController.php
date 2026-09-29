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
		string $payload_format = 'audio/flac',
	): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($production_id)['actor_id'];
		} catch (RuntimeException) {
			return $this->rejected('talk_context_unauthorized', 403);
		}

		$prepared = null;
		try {
			$this->authorizeRecordingTransport($actorId, $production_id, $recording_id);
			$provenance = $this->decodeProvenance($capture_provenance);

			$preferredFileId = null;
			$existing = $this->artifactManifestStore->get($capture_id);
			if (($existing['status'] ?? null) === 'verified') {
				$remote = is_array($existing['remote'] ?? null) ? $existing['remote'] : [];
				if (is_int($remote['file_id'] ?? null) && $remote['file_id'] > 0) {
					$preferredFileId = $remote['file_id'];
				}
			}

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
				$this->required($payload_format, 'payload_format'),
				$preferredFileId,
			);

			$staged = $this->stageArtifact(
				$production_id,
				$production_label,
				$recording_id,
				$capture_id,
				$recording_session_id,
				$participant_label,
				$prepared,
				$provenance,
			);

			if (($staged['status'] ?? null) === 'verified') {
				$verifiedRemote = is_array($staged['remote'] ?? null) ? $staged['remote'] : [];
				$verifiedFileId = is_int($verifiedRemote['file_id'] ?? null) && $verifiedRemote['file_id'] > 0
					? $verifiedRemote['file_id']
					: null;

				if ($verifiedFileId === null) {
					throw new RuntimeException('artifact_manifest_invalid');
				}

				// A preferred File-ID can disappear while another request
				// completes the same Artifact. Only the record that still points
				// to our original preferred File-ID may be replaced here.
				if ($preferredFileId !== null
					&& $prepared['upload_required'] === true
					&& $verifiedFileId === $preferredFileId) {
					if (!$this->artifactManifestStore->removeIfVerifiedRemoteFileIdMatches($capture_id, $verifiedFileId)) {
						try {
							$this->connector->close($prepared['transfer_id'], $actorId);
						} catch (\Throwable) {
						}
						throw new RuntimeException('artifact_manifest_conflict');
					}

					$replacementStage = $this->stageArtifact(
						$production_id,
						$production_label,
						$recording_id,
						$capture_id,
						$recording_session_id,
						$participant_label,
						$prepared,
						$provenance,
					);

					if (($replacementStage['status'] ?? null) === 'verified') {
						// A concurrent request won the race after the stale record
						// was removed. Do not upload through our now-superfluous
						// handle; reuse the winner's verified File-ID instead.
						try {
							$this->connector->close($prepared['transfer_id'], $actorId);
						} catch (\Throwable) {
						}

						$replacementRemote = is_array($replacementStage['remote'] ?? null) ? $replacementStage['remote'] : [];
						$replacementFileId = is_int($replacementRemote['file_id'] ?? null) && $replacementRemote['file_id'] > 0
							? $replacementRemote['file_id']
							: null;
						if ($replacementFileId === null) {
							throw new RuntimeException('artifact_manifest_invalid');
						}

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
							$replacementFileId,
						);
					}
				} else {
					// Another request has already established the canonical
					// verified file, or no stale verified record existed to
					// replace. Close our provisional handle and bind the
					// transport to the current verified File-ID.
					try {
						$this->connector->close($prepared['transfer_id'], $actorId);
					} catch (\Throwable) {
					}

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
						$verifiedFileId,
					);
				}
			}

			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'prepared',
				'transfer_id' => $prepared['transfer_id'],
				'upload_url' => $prepared['upload_url'],
				'upload_username' => $prepared['upload_username'],
				'upload_password' => $prepared['upload_password'],
				'filename' => $prepared['filename'],
				'size' => $prepared['size'],
				'sha256' => $prepared['sha256'],
				'payload_format' => $prepared['payload_format'] ?? $payload_format,
				'canonical_filename' => $prepared['canonical_filename'] ?? $prepared['filename'],
				'upload_required' => $prepared['upload_required'],
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			if ($prepared !== null) {
				try {
					$this->connector->close($prepared['transfer_id'], $actorId);
				} catch (\Throwable) {
				}
			}

			return $this->mapPrepareError($exception);
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
			if ($exception->getMessage() === 'artifact_manifest_context_missing') return $this->rejected('artifact_manifest_context_missing', 409);
			if ($exception->getMessage() === 'artifact_manifest_storage_unavailable') return $this->rejected('artifact_manifest_storage_unavailable', 503);
			if ($exception->getMessage() === 'artifact_preservation_invalid') return $this->rejected('artifact_preservation_invalid', 422);
			if ($exception->getMessage() === 'Nextcloud transport artifact has not arrived.') return $this->rejected('transport_artifact_missing', 409);
			if ($exception->getMessage() === 'Nextcloud transport artifact size does not match.'
				|| $exception->getMessage() === 'Nextcloud transport artifact SHA-256 does not match.') {
				return $this->rejected('transport_artifact_mismatch', 422);
			}
			if ($exception->getMessage() === 'Nextcloud transport artifact is not a valid WAV container.'
				|| $exception->getMessage() === 'Nextcloud transport artifact is not a supported PoRE PCM WAV.') {
				return $this->rejected('artifact_preservation_invalid', 422);
			}
			if (str_starts_with($exception->getMessage(), 'Invalid transport handle.')
				|| str_starts_with($exception->getMessage(), 'Invalid transport handle payload.')
				|| $exception->getMessage() === 'Incomplete transport handle.') {
				return $this->rejected('transport_handle_invalid', 409);
			}
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

	/**
	 * @param array<string, mixed> $prepared
	 * @param array<string, mixed> $provenance
	 * @return array<string, mixed>
	 */
	private function stageArtifact(
		string $productionId,
		string $productionLabel,
		string $recordingId,
		string $captureId,
		string $recordingSessionId,
		string $participantLabel,
		array $prepared,
		array $provenance,
	): array {
		return $this->artifactManifestStore->stagePreparedArtifact([
			'artifact_id' => $captureId,
			'production_id' => $productionId,
			'production_label' => $productionLabel,
			'recording_id' => $recordingId,
			'recording_session_id' => $this->required($recordingSessionId, 'recording_session_id'),
			'participant_label' => $participantLabel,
			'target_user_id' => $prepared['target_user_id'] ?? null,
			'filename' => $prepared['filename'] ?? null,
			'size' => $prepared['size'] ?? null,
			'payload_sha256' => $prepared['sha256'] ?? null,
			'capture_provenance' => $provenance,
		]);
	}

	private function decodeProvenance(string $json): array {
		if (strlen($json) > 65536) throw new RuntimeException('artifact_provenance_invalid');

		try {
			$decoded = json_decode($json === '' ? '{}' : $json, true, 512, JSON_THROW_ON_ERROR);
		} catch (\JsonException) {
			throw new RuntimeException('artifact_provenance_invalid');
		}
		if (!is_array($decoded)) throw new RuntimeException('artifact_provenance_invalid');
		return $decoded;
	}

	private function mapPrepareError(\Throwable $exception): DataResponse {
		return match ($exception->getMessage()) {
			'PoRE transport authorization is not available.' => $this->rejected('runtime_unavailable', 503),
			'PoRE transport authorization is not permitted for this recording' => $this->rejected('transport_unauthorized', 403),
			'artifact_provenance_invalid' => $this->rejected('artifact_provenance_invalid', 400),
			'artifact_payload_format_invalid' => $this->rejected('artifact_payload_format_invalid', 400),
			'artifact_manifest_invalid' => $this->rejected('artifact_manifest_invalid', 400),
			'artifact_manifest_conflict' => $this->rejected('artifact_manifest_conflict', 409),
			'Nextcloud recorded artifact payload has changed.' => $this->rejected('artifact_manifest_conflict', 409),
			default => $this->rejected(),
		};
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
