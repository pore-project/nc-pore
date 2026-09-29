<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCP\App\IAppManager;
use OCP\IConfig;
use RuntimeException;

final class RecordingRuntimeService {
	private const MAX_FRAME_LENGTH = 1024 * 1024;
	private const DEFAULT_COMMAND_TIMEOUT_SECONDS = 10;
	private const PROCESS_POLL_INTERVAL_MICROSECONDS = 10_000;
	private const PROCESS_TERMINATION_GRACE_MICROSECONDS = 250_000;

	/** Stateless runtime operations do not mutate the recording session store. */
	private const LOCK_FREE_OPERATIONS = [
		'artifact.convert_flac_to_wav',
	];

	public function __construct(
		private readonly IConfig $config,
		private readonly IAppManager $appManager,
	) {
	}

	/**
	 * Execute one host-neutral PoRE command through the runtime.
	 *
	 * This class is deliberately only a transport adapter: it does not interpret
	 * lifecycle state and does not duplicate Core/Application logic.
	 *
	 * @param array<string, mixed> $request
	 * @return array<string, mixed>
	 */
	public function command(array $request, string $operation = 'recording.command', int $timeoutSeconds = self::DEFAULT_COMMAND_TIMEOUT_SECONDS): array {
		if ($timeoutSeconds <= 0) throw new RuntimeException('PoRE runtime command timeout must be positive.');
		$binary = trim((string)$this->config->getSystemValue('pore_runtime_binary', ''));
		if ($binary === '') {
			$binary = rtrim($this->appManager->getAppPath('pore'), '/') . '/runtime/bin/pore-runtime';
		}

		$sessionStore = trim((string)$this->config->getSystemValue('pore_runtime_session_store', ''));
		if ($sessionStore === '') {
			$dataDirectory = trim((string)$this->config->getSystemValue('datadirectory', ''));
			$instanceId = trim((string)$this->config->getSystemValue('instanceid', ''));
			if ($dataDirectory === '' || $instanceId === '') {
				throw new RuntimeException('Unable to determine the Nextcloud app data directory for the PoRE runtime.');
			}
			$sessionStore = rtrim($dataDirectory, '/') . '/appdata_' . $instanceId . '/pore/runtime/sessions';
		}

		if (!is_file($binary) || !is_executable($binary)) {
			throw new RuntimeException('Configured PoRE runtime binary is not executable.');
		}
		if (!is_dir($sessionStore) && !mkdir($sessionStore, 0770, true) && !is_dir($sessionStore)) {
			throw new RuntimeException('Unable to create the PoRE runtime session store.');
		}
		if (!is_writable($sessionStore)) {
			throw new RuntimeException('PoRE runtime session store is not writable.');
		}

		$lock = null;
		if ($this->requiresSessionLock($operation)) {
			$lockPath = rtrim($sessionStore, '/') . '/.runtime.lock';
			$lock = fopen($lockPath, 'c');
			if ($lock === false) {
				throw new RuntimeException('Unable to open the PoRE runtime session lock.');
			}
			if (!flock($lock, LOCK_EX)) {
				fclose($lock);
				$lock = null;
				throw new RuntimeException('Unable to acquire the PoRE runtime session lock.');
			}
		}

		$request['protocol_version'] = 1;
		$request['operation'] = $operation;
		$payload = json_encode($request, JSON_THROW_ON_ERROR | JSON_UNESCAPED_UNICODE);
		$frame = pack('N', strlen($payload)) . $payload;

		$environment = getenv();
		if (!is_array($environment)) {
			if (is_resource($lock)) {
				flock($lock, LOCK_UN);
				fclose($lock);
			}
			throw new RuntimeException('Unable to read the process environment for the PoRE runtime.');
		}
		$environment['PORE_SESSION_STORE'] = $sessionStore;

		$descriptors = [
			0 => ['pipe', 'r'],
			1 => ['pipe', 'w'],
			2 => ['pipe', 'w'],
		];
		$process = null;
		$deadline = microtime(true) + $timeoutSeconds;

		try {
			$process = proc_open([$binary], $descriptors, $pipes, null, $environment);
			if (!is_resource($process)) {
				throw new RuntimeException('Unable to start the PoRE runtime.');
			}

			stream_set_blocking($pipes[2], false);

			if (!fwrite($pipes[0], $frame)) {
				throw new RuntimeException('Unable to send command to the PoRE runtime.');
			}
			fclose($pipes[0]);
			$pipes[0] = null;

			$lengthBytes = $this->readExact($pipes[1], 4, $deadline, $process, $pipes[2]);
			if (strlen($lengthBytes) !== 4) {
				throw new RuntimeException('PoRE runtime returned an incomplete response.');
			}
			$length = unpack('Nlength', $lengthBytes)['length'];
			if ($length === 0 || $length > self::MAX_FRAME_LENGTH) {
				throw new RuntimeException('PoRE runtime returned an invalid response frame.');
			}
			$response = $this->readExact($pipes[1], $length, $deadline, $process, $pipes[2]);
			if (strlen($response) !== $length) {
				throw new RuntimeException('PoRE runtime returned an incomplete response payload.');
			}
			$decoded = json_decode($response, true, 512, JSON_THROW_ON_ERROR);
			if (!is_array($decoded)) {
				throw new RuntimeException('PoRE runtime response is not a JSON object.');
			}

			$this->waitForProcess($process, $pipes[2], $deadline);
			return $decoded;
		} finally {
			if (isset($pipes[0]) && is_resource($pipes[0])) fclose($pipes[0]);
			if (isset($pipes[1]) && is_resource($pipes[1])) fclose($pipes[1]);
			if (isset($pipes[2]) && is_resource($pipes[2])) fclose($pipes[2]);
			if (is_resource($process)) proc_close($process);
			if (is_resource($lock)) {
				flock($lock, LOCK_UN);
				fclose($lock);
			}
		}
	}
	private function requiresSessionLock(string $operation): bool {
		return !in_array($operation, self::LOCK_FREE_OPERATIONS, true);
	}

	private function readExact($stream, int $length, float $deadline, $process, $stderr): string {
		$result = '';
		stream_set_timeout($stream, 0, 100_000);

		while (strlen($result) < $length && !feof($stream)) {
			$this->terminateOnDeadline($process, $deadline);
			$chunk = fread($stream, $length - strlen($result));
			$this->drainStream($stderr);
			if ($chunk === false) {
				throw new RuntimeException('Unable to read the PoRE runtime response.');
			}
			if ($chunk !== '') {
				$result .= $chunk;
				continue;
			}
			if ((stream_get_meta_data($stream)['timed_out'] ?? false) === true) {
				continue;
			}
		}

		$this->terminateOnDeadline($process, $deadline);
		return $result;
	}

	private function waitForProcess($process, $stderr, float $deadline): void {
		while (true) {
			$status = proc_get_status($process);
			if (!($status['running'] ?? false)) {
				return;
			}
			$this->terminateOnDeadline($process, $deadline);
			$this->drainStream($stderr);
			usleep(self::PROCESS_POLL_INTERVAL_MICROSECONDS);
		}
	}

	private function terminateOnDeadline($process, float $deadline): void {
		if (microtime(true) < $deadline) {
			return;
		}

		$status = proc_get_status($process);
		if (($status['running'] ?? false) === true) {
			@proc_terminate($process);

			$graceDeadline = microtime(true) + (self::PROCESS_TERMINATION_GRACE_MICROSECONDS / 1_000_000);
			while (microtime(true) < $graceDeadline) {
				$status = proc_get_status($process);
				if (!($status['running'] ?? false)) {
					break;
				}
				usleep(10_000);
			}

			if (($status['running'] ?? false) === true && PHP_OS_FAMILY !== 'Windows') {
				@proc_terminate($process, 9);
			}
		}

		throw new RuntimeException('PoRE runtime command timed out.');
	}

	private function drainStream($stream): void {
		while (($chunk = fread($stream, 8192)) !== false && $chunk !== '') {
		}
	}
}
