<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\TalkSessionAccessService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\Attribute\PublicPage;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use RuntimeException;

final class TalkContextController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly TalkSessionAccessService $talkSessionAccess,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[PublicPage]
	#[NoAdminRequired]
	public function context(string $sessionId = ''): DataResponse {
		try {
			$context = $this->talkSessionAccess->resolve($sessionId, true);
			return new DataResponse([
				'protocol_version' => 1,
				'status' => 'ok',
				'actor_type' => $context['actor_type'],
				'actor_id' => $context['actor_id'],
				'display_name' => $context['display_name'],
				'participant_type' => $context['participant_type'],
				'talk_session_id' => $context['talk_session_id'],
				'owner_id' => $context['owner_id'],
				'guest' => $context['actor_type'] === 'guests',
				'error_code' => null,
			]);
		} catch (RuntimeException $exception) {
			$errorCode = in_array($exception->getMessage(), ['talk_session_unavailable', 'talk_session_unauthorized', 'talk_actor_unsupported'], true)
				? $exception->getMessage()
				: 'talk_context_unavailable';
			return new DataResponse([
				'protocol_version' => 1,
				'status' => 'rejected',
				'error_code' => $errorCode,
			], 403);
		}
	}
}
