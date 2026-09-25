<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
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
	): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($production_id)['actor_id'];
		} catch (RuntimeException) {
			return $this->rejected('talk_context_unauthorized', 403);
		}

		try {
			$this->authorizeRecordingTransport($actorId, $production_id, $recording_id);
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
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'prepared',
				...$prepared,
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			if ($exception->getMessage() === 'PoRE transport authorization is not available.') return $this->rejected('runtime_unavailable', 503);
			if ($exception->getMessage() === 'PoRE transport authorization is not permitted for this recording') return $this->rejected('transport_unauthorized', 403);
			return $this->rejected();
		}
	}

	#[PublicPage]
	#[NoAdminRequired]
	public function verifyFinalizedArtifact(string $transfer_id, string $session_id = ''): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($session_id)['actor_id'];
			$receipt = $this->connector->verify($this->required($transfer_id, 'transfer_id'), $actorId);
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'verified',
				...$receipt,
				'error_code' => null,
			]);
		} catch (\Throwable) {
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
