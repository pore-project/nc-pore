<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Http\CoordinationEventStreamResponse;
use OCA\PoRe\Service\RecordingCoordinationService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\Attribute\NoCSRFRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\Http\Response;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use OCP\IUserSession;
use InvalidArgumentException;
use RuntimeException;

final class RecordingCoordinationController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly RecordingCoordinationService $coordination,
		private readonly IUserSession $userSession,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[NoAdminRequired]
	#[NoCSRFRequired]
	public function events(string $sessionId, string $recordingId): Response {
		$user = $this->userSession->getUser();
		if ($user === null) {
			$response = new Response();
			$response->setStatus(401);
			return $response;
		}

		try {
			$this->coordination->authorizeStream($sessionId, $recordingId, $user->getUID());
			$lastEventId = $this->lastEventId();
			return new CoordinationEventStreamResponse(
				function (callable $emit) use ($sessionId, $recordingId, $lastEventId): void {
					$this->coordination->stream($sessionId, $recordingId, $lastEventId, $emit);
				},
			);
		} catch (InvalidArgumentException) {
			$response = new Response();
			$response->setStatus(400);
			return $response;
		} catch (RuntimeException $exception) {
			$status = $exception->getMessage() === 'coordination_unauthorized' ? 403 : 503;
			$response = new Response();
			$response->setStatus($status);
			return $response;
		}
	}

	#[NoAdminRequired]
	public function publish(
		string $sessionId,
		string $recordingId,
		string $eventType,
		string $requestId = '',
	): DataResponse {
		$user = $this->userSession->getUser();
		if ($user === null) return $this->rejected('unauthorized', 401, $requestId);

		try {
			$result = $this->coordination->publish($sessionId, $recordingId, $eventType, $user->getUID());
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

	private function lastEventId(): int {
		$header = trim($this->request->getHeader('Last-Event-ID'));
		$query = trim((string)$this->request->getParam('lastEventId', ''));
		$value = $header !== '' ? $header : $query;
		return ctype_digit($value) ? (int)$value : 0;
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
