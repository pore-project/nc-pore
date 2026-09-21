<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCP\IConfig;
use RuntimeException;
use InvalidArgumentException;

final class RecordingCoordinationService {
	private const PROTOCOL_VERSION = 1;
	private const MAX_EVENT_COUNT = 256;
	private const STREAM_LIFETIME_SECONDS = 20;
	private const HEARTBEAT_INTERVAL_SECONDS = 10;
	private const WAIT_MICROSECONDS = 250000;

	private const ALLOWED_EVENTS = [
		'begin',
		'ready',
		'opening',
		'stop',
		'production_closed',
	];

	private const HOST_EVENTS = [
		'begin',
		'opening',
		'stop',
		'production_closed',
	];

	public function __construct(
		private readonly IConfig $config,
		private readonly RecordingRuntimeService $runtime,
	) {
	}

	/**
	 * @return array<string, mixed>
	 */
	public function publish(string $sessionId, string $recordingId, string $eventType, string $actorId): array {
		$this->validateIdentity($sessionId, $recordingId, $actorId);
		if (!in_array($eventType, self::ALLOWED_EVENTS, true)) {
			throw new InvalidArgumentException('Unsupported recording coordination event.');
		}

		$state = $this->authorizeParticipant($sessionId, $recordingId, $actorId);
		$role = $state['role'] ?? null;
		if (in_array($eventType, self::HOST_EVENTS, true) && $role !== 'host') {
			throw new RuntimeException('coordination_forbidden');
		}

		$event = $this->appendEvent($sessionId, $recordingId, $eventType, $actorId);

		return [
			'protocol_version' => self::PROTOCOL_VERSION,
			'status' => 'published',
			'event' => $event,
			'error_code' => null,
		];
	}

	public function authorizeStream(string $sessionId, string $recordingId, string $actorId): void {
		$this->validateIdentity($sessionId, $recordingId, $actorId);
		$this->authorizeParticipant($sessionId, $recordingId, $actorId);
	}

	/**
	 * Stream newly published events until the short-lived SSE request expires.
	 *
	 * @param callable(string): void $emit
	 */
	public function stream(string $sessionId, string $recordingId, int $lastEventId, callable $emit): void {
		$this->validateIdentity($sessionId, $recordingId, 'stream');
		set_time_limit((int)self::STREAM_LIFETIME_SECONDS + 5);

		$startedAt = microtime(true);
		$lastHeartbeatAt = $startedAt;
		$cursor = max(0, $lastEventId);

		$emit(": pore-recording-stream\n\n");

		$logPath = $this->logPath($sessionId);
		while ((microtime(true) - $startedAt) < self::STREAM_LIFETIME_SECONDS) {
			if (connection_aborted()) return;

			foreach ($this->readEventsAfter($logPath, $recordingId, $cursor) as $event) {
				$cursor = max($cursor, (int)$event['id']);
				$emit($this->formatEvent($event));
				if (connection_aborted()) return;
			}

			$now = microtime(true);
			if (($now - $lastHeartbeatAt) >= self::HEARTBEAT_INTERVAL_SECONDS) {
				$emit(": keep-alive\n\n");
				$lastHeartbeatAt = $now;
				if (connection_aborted()) return;
			}

			usleep(self::WAIT_MICROSECONDS);
		}
	}

	/**
	 * @return array<string, mixed>
	 */
	private function authorizeParticipant(string $sessionId, string $recordingId, string $actorId): array {
		try {
			$response = $this->runtime->command([
				'request_id' => bin2hex(random_bytes(16)),
				'session_id' => $sessionId,
				'actor_id' => $actorId,
				'recording_id' => $recordingId,
				'command' => ['Snapshot' => null],
			], 'recording.command');
		} catch (\Throwable $exception) {
			throw new RuntimeException('coordination_unavailable', 0, $exception);
		}

		if (($response['status'] ?? null) !== 'ok' || !is_array($response['state'] ?? null)) {
			throw new RuntimeException('coordination_unauthorized');
		}

		$state = $response['state'];
		$role = $state['role'] ?? null;
		if (!in_array($role, ['host', 'participant'], true)) {
			throw new RuntimeException('coordination_unauthorized');
		}

		$participants = is_array($state['participants'] ?? null) ? $state['participants'] : [];
		foreach ($participants as $participant) {
			if (($participant['id'] ?? null) === $actorId) return $state;
		}

		throw new RuntimeException('coordination_unauthorized');
	}

	private function validateIdentity(string $sessionId, string $recordingId, string $actorId): void {
		if (trim($sessionId) === '' || trim($recordingId) === '' || trim($actorId) === '') {
			throw new InvalidArgumentException('Recording coordination identity is incomplete.');
		}
		if (strlen($sessionId) > 255 || strlen($recordingId) > 255 || strlen($actorId) > 255) {
			throw new InvalidArgumentException('Recording coordination identity is too long.');
		}
	}

	/**
	 * @return array<string, mixed>
	 */
	private function appendEvent(string $sessionId, string $recordingId, string $eventType, string $actorId): array {
		$directory = $this->coordinationDirectory();
		if (!is_dir($directory) && !mkdir($directory, 0770, true) && !is_dir($directory)) {
			throw new RuntimeException('Unable to create the PoRE coordination directory.');
		}

		$lock = fopen($this->lockPath($sessionId), 'c');
		if ($lock === false) throw new RuntimeException('Unable to open the PoRE coordination lock.');
		if (!flock($lock, LOCK_EX)) {
			fclose($lock);
			throw new RuntimeException('Unable to acquire the PoRE coordination lock.');
		}

		try {
			$logPath = $this->logPath($sessionId);
			$lastEventId = $this->readLastEventId($logPath);
			$event = [
				'id' => $lastEventId + 1,
				'version' => self::PROTOCOL_VERSION,
				'type' => $eventType,
				'sessionId' => $sessionId,
				'recordingId' => $recordingId,
				'actorId' => $actorId,
				'createdAt' => gmdate('c'),
			];

			$handle = fopen($logPath, 'ab');
			if ($handle === false) throw new RuntimeException('Unable to open the PoRE coordination event log.');
			try {
				$line = json_encode($event, JSON_THROW_ON_ERROR | JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES) . "\n";
				if (fwrite($handle, $line) === false) throw new RuntimeException('Unable to append the PoRE coordination event.');
				fflush($handle);
			} finally {
				fclose($handle);
			}

			$this->compactLog($logPath);
			return $event;
		} finally {
			flock($lock, LOCK_UN);
			fclose($lock);
		}
	}

	private function compactLog(string $logPath): void {
		if (!is_file($logPath)) return;
		$lines = file($logPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
		if (!is_array($lines) || count($lines) <= self::MAX_EVENT_COUNT) return;

		$lines = array_slice($lines, -self::MAX_EVENT_COUNT);
		file_put_contents($logPath, implode("\n", $lines) . "\n", LOCK_EX);
	}

	private function readLastEventId(string $logPath): int {
		if (!is_file($logPath)) return 0;
		$lines = file($logPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
		if (!is_array($lines)) return 0;

		for ($index = count($lines) - 1; $index >= 0; $index--) {
			$decoded = json_decode($lines[$index], true);
			if (is_array($decoded) && is_numeric($decoded['id'] ?? null)) return (int)$decoded['id'];
		}
		return 0;
	}

	/**
	 * @return list<array<string, mixed>>
	 */
	private function readEventsAfter(string $logPath, string $recordingId, int $cursor): array {
		if (!is_file($logPath)) return [];
		$lines = file($logPath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
		if (!is_array($lines)) return [];

		$events = [];
		foreach ($lines as $line) {
			$event = json_decode($line, true);
			if (!is_array($event)) continue;
			if (($event['recordingId'] ?? null) !== $recordingId) continue;
			if ((int)($event['id'] ?? 0) <= $cursor) continue;
			if (($event['version'] ?? null) !== self::PROTOCOL_VERSION) continue;
			$events[] = $event;
		}
		return $events;
	}

	/**
	 * @param array<string, mixed> $event
	 */
	private function formatEvent(array $event): string {
		$data = json_encode($event, JSON_THROW_ON_ERROR | JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
		return 'id: ' . (int)$event['id'] . "\nevent: pore-recording\ndata: " . $data . "\n\n";
	}

	private function coordinationDirectory(): string {
		$dataDirectory = trim((string)$this->config->getSystemValue('datadirectory', ''));
		$instanceId = trim((string)$this->config->getSystemValue('instanceid', ''));
		if ($dataDirectory === '' || $instanceId === '') {
			throw new RuntimeException('Unable to determine the Nextcloud app data directory for PoRE coordination.');
		}
		return rtrim($dataDirectory, '/') . '/appdata_' . $instanceId . '/pore/coordination';
	}

	private function logPath(string $sessionId): string {
		return $this->coordinationDirectory() . '/' . hash('sha256', $sessionId) . '.events';
	}

	private function lockPath(string $sessionId): string {
		return $this->coordinationDirectory() . '/' . hash('sha256', $sessionId) . '.lock';
	}
}
