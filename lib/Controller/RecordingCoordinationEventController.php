<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Http\CoordinationEventStreamResponse;
use OCA\PoRe\Service\RecordingCoordinationService;
use OCP\AppFramework\Controller;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\Attribute\NoCSRFRequired;
use OCP\AppFramework\Http\Response;
use OCP\IRequest;
use OCP\IUserSession;
use InvalidArgumentException;
use RuntimeException;

final class RecordingCoordinationEventController extends Controller {
	public function __construct(
		IRequest $request,
		private readonly RecordingCoordinationService $coordination,
		private readonly IUserSession $userSession,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[NoAdminRequired]
	#[NoCSRFRequired]
	// sessionId and recordingId are query parameters on this GET route. Defaults keep
	// the AppFramework dispatcher from passing null into non-nullable string parameters
	// when a malformed request omits one; validateIdentity() then rejects it cleanly.
	public function events(string $sessionId = '', string $recordingId = ''): Response {
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

	private function lastEventId(): int {
		$header = trim($this->request->getHeader('Last-Event-ID'));
		$query = trim((string)$this->request->getParam('lastEventId', ''));
		$value = $header !== '' ? $header : $query;
		return ctype_digit($value) ? (int)$value : 0;
	}
}
