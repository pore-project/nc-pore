<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCA\Talk\Exceptions\RoomNotFoundException;
use OCA\Talk\Manager;
use OCP\Server;

final class TalkSessionAccessService {
	/**
	 * Check whether the authenticated user is a participant of the Talk conversation.
	 *
	 * This is deliberately kept at the Talk adapter boundary. The recording
	 * coordination service itself remains provider-neutral.
	 */
	public function isParticipant(string $sessionId, string $actorId): bool {
		if (trim($sessionId) === '' || trim($actorId) === '') {
			return false;
		}
		if (!class_exists(Manager::class)) {
			return false;
		}

		try {
			Server::get(Manager::class)->getRoomByActor($sessionId, 'users', $actorId);
			return true;
		} catch (RoomNotFoundException) {
			return false;
		}
	}
}
