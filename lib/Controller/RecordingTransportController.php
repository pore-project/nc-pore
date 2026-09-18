<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\NextcloudArtifactConnector;
use OCA\PoRe\Service\RecordingRuntimeService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use OCP\IUserSession;
use RuntimeException;

final class RecordingTransportController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly NextcloudArtifactConnector $connector,
		private readonly RecordingRuntimeService $runtime,
		private readonly IUserSession $userSession,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

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
		$user = $this->userSession->getUser();
		if ($user === null) return $this->rejected('unauthorized', 401);

		try {
			$this->authorizeRecordingTransport($user->getUID(), $production_id, $recording_id);
			$prepared = $this->connector->prepare(
				$this->required($production_id, 'production_id'),
				$this->required($production_label, 'production_label'),
				$this->required($recording_id, 'recording_id'),
				$this->required($capture_id, 'capture_id'),
				$this->required($started_at, 'started_at'),
				$participant_label,
				$size,
				$this->required($payload_sha256, 'payload_sha256'),
				$user->getUID(),
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

	#[NoAdminRequired]
	public function verifyFinalizedArtifact(string $transfer_id): DataResponse {
		$user = $this->userSession->getUser();
		if ($user === null) return $this->rejected('unauthorized', 401);

		try {
			$receipt = $this->connector->verify($this->required($transfer_id, 'transfer_id'), $user->getUID());
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

	#[NoAdminRequired]
	public function closeFinalizedArtifactTransfer(string $transfer_id): DataResponse {
		$user = $this->userSession->getUser();
		if ($user === null) return $this->rejected('unauthorized', 401);

		try {
			$this->connector->close($this->required($transfer_id, 'transfer_id'), $user->getUID());
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
			'error_code' => 'nextcloud_transport_failed',
		], 500);
	}
}
