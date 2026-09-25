<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCA\Talk\Exceptions\RoomNotFoundException;
use OCA\Talk\Manager;
use OCA\Talk\Model\Attendee;
use OCA\Talk\Participant;
use OCA\Talk\Service\ParticipantService;
use OCA\Talk\TalkSession;
use OCP\IUserSession;
use RuntimeException;

final class TalkSessionAccessService {
	public function __construct(
		private readonly TalkSession $talkSession,
		private readonly Manager $manager,
		private readonly ParticipantService $participantService,
		private readonly IUserSession $userSession,
	) {
	}

	/**
	 * Resolve the actor for the current Talk browser session.
	 *
	 * The Talk session is the trust anchor. A normal Nextcloud user is validated
	 * by user id; a genuine Talk guest is resolved by the Talk session itself.
	 *
	 * @return array{actor_type:string,actor_id:string,display_name:string,participant_type:int,talk_session_id:string,owner_id:string}
	 */
	public function resolve(string $conversationToken, bool $includeOwner = false): array {
		if (trim($conversationToken) === '') {
			throw new RuntimeException('talk_session_unavailable');
		}

		$talkSessionId = $this->talkSession->getSessionForRoom($conversationToken);
		if ($talkSessionId === null || $talkSessionId === '') {
			throw new RuntimeException('talk_session_unavailable');
		}

		$user = $this->userSession->getUser();
		$userId = $user?->getUID();

		try {
			$room = $this->manager->getRoomForSession($userId, $talkSessionId);
			$participant = $this->participantService->getParticipantBySession($room, $talkSessionId);
		} catch (RoomNotFoundException $exception) {
			throw new RuntimeException('talk_session_unauthorized', 0, $exception);
		}

		$attendee = $participant->getAttendee();
		$actorType = $attendee->getActorType();
		if (!in_array($actorType, [Attendee::ACTOR_USERS, Attendee::ACTOR_GUESTS], true)) {
			throw new RuntimeException('talk_actor_unsupported');
		}

		$actorId = trim($attendee->getActorId());
		if ($actorId === '') {
			throw new RuntimeException('talk_session_unauthorized');
		}

		$ownerId = '';
		if ($includeOwner) {
			foreach ($this->participantService->getParticipantsForRoom($room) as $candidate) {
				if (!$candidate instanceof Participant || !$candidate->isOwner()) {
					continue;
				}
				if ($candidate->getAttendee()->getActorType() !== Attendee::ACTOR_USERS) {
					continue;
				}
				$ownerId = trim($candidate->getAttendee()->getActorId());
				break;
			}
		}

		return [
			'actor_type' => $actorType,
			'actor_id' => $actorId,
			'display_name' => trim((string)$attendee->getDisplayName()),
			'participant_type' => $attendee->getParticipantType(),
			'talk_session_id' => $talkSessionId,
			'owner_id' => $ownerId,
		];
	}

	public function isParticipant(string $sessionId, string $actorId): bool {
		$actorId = trim($actorId);
		if ($actorId === '') {
			return false;
		}

		try {
			$context = $this->resolve($sessionId);
			return hash_equals($context['actor_id'], $actorId);
		} catch (RuntimeException) {
			return false;
		}
	}
}
