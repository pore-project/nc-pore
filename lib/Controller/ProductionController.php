<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\RecordingRuntimeService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IConfig;
use OCP\IRequest;
use OCP\IUserSession;

final class ProductionController extends OCSController {
	private const OWNER_PREFIX = 'production_owner_';
	private const OWNER_HASH_LENGTH = 47;

	public function __construct(
		IRequest $request,
		private readonly RecordingRuntimeService $runtime,
		private readonly IUserSession $userSession,
		private readonly IConfig $config,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[NoAdminRequired]
	public function command(
		string $sessionId,
		string $command,
		string $requestId = '',
		string $participants = '[]',
		string $ownerId = '',
	): DataResponse {
		$user = $this->userSession->getUser();
		if ($user === null) {
			return $this->rejected('unauthorized', 401, $requestId);
		}

		if (!in_array($command, ['ensure', 'start', 'force_close'], true)) {
			return $this->rejected('unsupported_command', 400, $requestId);
		}

		$participantIds = json_decode($participants, true);
		if (!is_array($participantIds) || array_filter($participantIds, static fn ($id): bool => !is_string($id)) !== []) {
			return $this->rejected('invalid_participants', 400, $requestId);
		}

		$requestId = $requestId !== '' ? $requestId : bin2hex(random_bytes(16));
		$runtimeCommand = match ($command) {
    'ensure' => ['Ensure' => null],
    'start' => ['Start' => null],
    'force_close' => ['ForceClose' => null],
};

		try {
			$response = $this->runtime->command([
				'request_id' => $requestId,
				'session_id' => $sessionId,
				'actor_id' => $user->getUID(),
				'owner_id' => $ownerId,
				'participants' => array_values($participantIds),
				'command' => $runtimeCommand,
			], 'production.command');
		} catch (\Throwable) {
			return $this->rejected('runtime_unavailable', 503, $requestId);
		}

		if (($response['status'] ?? null) === 'ok' && $ownerId !== '' && $ownerId === $user->getUID()) {
			$this->config->setAppValue(Application::APP_ID, self::ownerKey($sessionId), $ownerId);
		}

		$status = ($response['status'] ?? null) === 'ok' ? 200 : 409;
		return new DataResponse($response, $status);
	}

	public static function ownerKey(string $sessionId): string {
		return self::OWNER_PREFIX . substr(hash('sha256', $sessionId), 0, self::OWNER_HASH_LENGTH);
	}

	private function rejected(string $errorCode, int $status, string $requestId): DataResponse {
		return new DataResponse([
			'protocol_version' => 1,
			'request_id' => $requestId,
			'status' => 'rejected',
			'production_status' => null,
			'participants' => [],
			'completion_reason' => null,
			'error_code' => $errorCode,
		], $status);
	}
}
