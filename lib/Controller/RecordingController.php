<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\BackgroundJob\CheckProductionArtifactTimeoutJob;
use OCA\PoRe\Service\RecordingRuntimeService;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\Attribute\PublicPage;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\BackgroundJob\IJobList;
use OCP\IRequest;
use OCA\PoRe\Service\TalkSessionAccessService;
use RuntimeException;

final class RecordingController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly RecordingRuntimeService $runtime,
		private readonly TalkSessionAccessService $talkSessionAccess,
		private readonly IJobList $jobList,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[PublicPage]
	#[NoAdminRequired]
	public function command(
		string $sessionId,
		string $recordingId,
		string $command,
		string $requestId = '',
		string $participants = '[]',
		string $ownerId = '',
		string $artifactId = '',
	): DataResponse {
		try {
			$actorId = $this->talkSessionAccess->resolve($sessionId)['actor_id'];
		} catch (RuntimeException) {
			return $this->rejected('talk_context_unauthorized', 403, $requestId);
		}

		$allowed = ['ensure', 'begin', 'ready', 'trigger_opening', 'confirm_opening', 'start', 'stop', 'acknowledge_stop', 'complete', 'snapshot'];
		if (!in_array($command, $allowed, true)) {
			return $this->rejected('unsupported_command', 400, $requestId);
		}

		try {
			$participantIds = json_decode($participants, true, 512, JSON_THROW_ON_ERROR);
		} catch (\JsonException) {
			return $this->rejected('invalid_participants', 400, $requestId);
		}
		if (!is_array($participantIds) || array_filter($participantIds, static fn ($id): bool => !is_string($id)) !== []) {
			return $this->rejected('invalid_participants', 400, $requestId);
		}
		$requestId = $requestId !== '' ? $requestId : bin2hex(random_bytes(16));

		try {
			if ($command === 'ensure') {
				$response = $this->execute($requestId, $sessionId, $recordingId, $actorId, ['EnsureRecording' => null]);
			} else {
				try {
					$runtimeCommand = match ($command) {
						'begin' => ['Begin' => ['participants' => array_values($participantIds)]],
						'ready' => ['MarkReady' => null],
						'trigger_opening' => ['TriggerOpening' => null],
						'confirm_opening' => ['ConfirmOpening' => null],
						'start' => ['Start' => null],
						'stop' => ['RequestStop' => null],
						'acknowledge_stop' => ['AcknowledgeStop' => null],
						'complete' => ['Complete' => ['artifact_id' => $this->requiredArtifactId($artifactId)]],
						'snapshot' => ['Snapshot' => null],
					};
				} catch (RuntimeException) {
					return $this->rejected('invalid_artifact', 400, $requestId);
				}
				$response = $this->execute($requestId, $sessionId, $recordingId, $actorId, $runtimeCommand);
			}
		} catch (\Throwable) {
			return $this->rejected('runtime_unavailable', 503, $requestId);
		}

		$status = ($response['status'] ?? null) === 'ok' ? 200 : 409;

		if ($command === 'stop' && $status === 200) {
			$this->jobList->scheduleAfter(
				CheckProductionArtifactTimeoutJob::class,
				time() + (24 * 60 * 60),
				['production_id' => $sessionId],
			);
		}

		return new DataResponse($response, $status);
	}

	/** @param array<string, mixed> $command */
	private function execute(string $requestId, string $sessionId, string $recordingId, string $actorId, array $command): array {
		[$variant, $value] = [array_key_first($command), array_values($command)[0]];
		return $this->runtime->command([
			'request_id' => $requestId,
			'session_id' => $sessionId,
			'actor_id' => $actorId,
			'recording_id' => $recordingId,
			'command' => [$variant => $value],
		]);
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
