<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\RecordingCoordinationService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\Attribute\PublicPage;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use OCA\PoRe\Service\TalkSessionAccessService;
use InvalidArgumentException;
use RuntimeException;

final class RecordingCoordinationController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly RecordingCoordinationService $coordination,
		private readonly TalkSessionAccessService $talkSessionAccess,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[PublicPage]
	#[NoAdminRequired]
	public function publish(
		string $sessionId,
		string $recordingId,
		string $eventType,
		string $requestId = '',
	): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($sessionId)['actor_id'];
		} catch (RuntimeException) {
			return $this->rejected('talk_context_unauthorized', 403, $requestId);
		}

		try {
			$result = $this->coordination->publish($sessionId, $recordingId, $eventType, $actorId);
			return new DataResponse([
				'protocol_version' => 1,
				'request_id' => $requestId !== '' ? $requestId : bin2hex(random_bytes(16)),
				...$result,
			]);
		} catch (InvalidArgumentException) {
			return $this->rejected('invalid_coordination_event', 400, $requestId);
		} catch (RuntimeException $exception) {
			$errorCode = match ($exception->getMessage()) {
				'coordination_unauthorized' => 'coordination_unauthorized',
				'coordination_forbidden' => 'coordination_forbidden',
				'coordination_unavailable' => 'coordination_unavailable',
				default => 'coordination_publish_failed',
			};
			$status = in_array($errorCode, ['coordination_unauthorized', 'coordination_forbidden'], true) ? 403 : 503;
			return $this->rejected($errorCode, $status, $requestId);
		} catch (\Throwable) {
			return $this->rejected('coordination_publish_failed', 503, $requestId);
		}
	}

	private function rejected(string $errorCode, int $status, string $requestId): DataResponse {
		return new DataResponse([
			'protocol_version' => 1,
			'request_id' => $requestId !== '' ? $requestId : bin2hex(random_bytes(16)),
			'status' => 'rejected',
			'error_code' => $errorCode,
		], $status);
	}
}
