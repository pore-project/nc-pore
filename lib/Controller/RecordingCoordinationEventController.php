<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Http\CoordinationEventStreamResponse;
use OCA\PoRe\Service\RecordingCoordinationService;
use OCA\PoRe\Service\TalkSessionAccessService;
use OCP\AppFramework\Controller;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\Attribute\PublicPage;
use OCP\AppFramework\Http\Attribute\NoCSRFRequired;
use OCP\AppFramework\Http\Response;
use OCP\IRequest;
use InvalidArgumentException;
use RuntimeException;

final class RecordingCoordinationEventController extends Controller {
	public function __construct(
		IRequest $request,
		private readonly RecordingCoordinationService $coordination,
		private readonly TalkSessionAccessService $talkSessionAccess,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[PublicPage]
	#[NoAdminRequired]
	#[NoCSRFRequired]
	// sessionId and recordingId are query parameters on this GET route. Defaults keep
	// the AppFramework dispatcher from passing null into non-nullable string parameters
	// when a malformed request omits one; validateIdentity() then rejects it cleanly.
	public function events(string $sessionId = '', string $recordingId = ''): Response {
		try {
			$actorId = $this->talkSessionAccess->resolve($sessionId)['actor_id'];
		} catch (RuntimeException) {
			$response = new Response();
			$response->setStatus(403);
			return $response;
		}

		try {
			$this->coordination->authorizeStream($sessionId, $recordingId, $actorId);
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
			if (
				in_array($exception->getMessage(), [
					'coordination_session_not_found',
					'coordination_recording_not_found',
				], true)
				&& $this->talkSessionAccess->isParticipant($sessionId, $actorId)
			) {
				$lastEventId = $this->lastEventId();
				return new CoordinationEventStreamResponse(
					function (callable $emit) use ($sessionId, $recordingId, $lastEventId): void {
						$this->coordination->stream($sessionId, $recordingId, $lastEventId, $emit);
					},
				);
			}
			$status = in_array($exception->getMessage(), [
				'coordination_unauthorized',
				'coordination_forbidden',
			], true) ? 403 : 503;
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
