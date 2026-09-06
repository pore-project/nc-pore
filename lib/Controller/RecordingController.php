<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\RecordingRuntimeService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use OCP\IUserSession;
use RuntimeException;

final class RecordingController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly RecordingRuntimeService $runtime,
		private readonly IUserSession $userSession,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[NoAdminRequired]
	public function command(
		string $sessionId,
		string $recordingId,
		string $command,
		string $requestId = '',
		string $participants = '[]',
		string $artifactId = '',
	): DataResponse {
		$user = $this->userSession->getUser();
		if ($user === null) {
			return $this->rejected('unauthorized', 401, $requestId);
		}

		$allowed = ['ensure', 'begin', 'ready', 'start', 'stop', 'acknowledge_stop', 'complete', 'snapshot'];
		if (!in_array($command, $allowed, true)) {
			return $this->rejected('unsupported_command', 400, $requestId);
		}

		$participantIds = json_decode($participants, true, 512, JSON_THROW_ON_ERROR);
		if (!is_array($participantIds) || array_filter($participantIds, static fn ($id): bool => !is_string($id)) !== []) {
			return $this->rejected('invalid_participants', 400, $requestId);
		}

		$runtimeCommand = match ($command) {
			'ensure' => ['EnsureRecording' => new \stdClass()],
			'begin' => ['Begin' => ['participants' => array_values($participantIds)]],
			'ready' => ['MarkReady' => new \stdClass()],
			'start' => ['Start' => new \stdClass()],
			'stop' => ['RequestStop' => new \stdClass()],
			'acknowledge_stop' => ['AcknowledgeStop' => new \stdClass()],
			'complete' => ['Complete' => ['artifact_id' => $this->requiredArtifactId($artifactId)]],
			'snapshot' => ['Snapshot' => new \stdClass()],
		};

		[$variant, $value] = [array_key_first($runtimeCommand), array_values($runtimeCommand)[0]];
		$request = [
			'request_id' => $requestId !== '' ? $requestId : bin2hex(random_bytes(16)),
			'session_id' => $sessionId,
			'actor_id' => $user->getUID(),
			'recording_id' => $recordingId,
			'command' => [$variant => $value],
		];

		try {
			$response = $this->runtime->command($request);
		} catch (\Throwable $exception) {
			return $this->rejected('runtime_unavailable', 503, $request['request_id']);
		}

		$status = ($response['status'] ?? null) === 'ok' ? 200 : 409;
		return new DataResponse($response, $status);
	}

	private function requiredArtifactId(string $artifactId): string {
		if (trim($artifactId) === '') {
			throw new RuntimeException('artifactId is required for completion.');
		}
		return $artifactId;
	}

	private function rejected(string $errorCode, int $status, string $requestId): DataResponse {
		return new DataResponse([
			'protocol_version' => 1,
			'request_id' => $requestId,
			'status' => 'rejected',
			'state' => null,
			'error_code' => $errorCode,
		], $status);
	}
}
