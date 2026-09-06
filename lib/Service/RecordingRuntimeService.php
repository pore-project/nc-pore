<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCP\IConfig;
use RuntimeException;

final class RecordingRuntimeService {
	private const MAX_FRAME_LENGTH = 1024 * 1024;

	public function __construct(private readonly IConfig $config) {
	}

	/**
	 * Execute one host-neutral recording command through the PoRE runtime.
	 *
	 * This class is deliberately only a transport adapter: it does not interpret
	 * recording lifecycle state and does not duplicate Core/Application logic.
	 *
	 * @param array<string, mixed> $request
	 * @return array<string, mixed>
	 */
	public function command(array $request): array {
		$binary = trim((string)$this->config->getSystemValue('pore_runtime_binary', ''));
		$sessionStore = trim((string)$this->config->getSystemValue('pore_runtime_session_store', ''));

		if ($binary === '' || $sessionStore === '') {
			throw new RuntimeException('PoRE runtime is not configured.');
		}
		if (!is_file($binary) || !is_executable($binary)) {
			throw new RuntimeException('Configured PoRE runtime binary is not executable.');
		}
		if (!is_dir($sessionStore) || !is_writable($sessionStore)) {
			throw new RuntimeException('Configured PoRE runtime session store is not writable.');
		}

		$request['protocol_version'] = 1;
		$request['operation'] = 'recording.command';
		$payload = json_encode($request, JSON_THROW_ON_ERROR | JSON_UNESCAPED_UNICODE);
		$frame = pack('N', strlen($payload)) . $payload;

		$environment = getenv();
		if (!is_array($environment)) {
			throw new RuntimeException('Unable to read the process environment for the PoRE runtime.');
		}
		$environment['PORE_SESSION_STORE'] = $sessionStore;

		$descriptors = [
			0 => ['pipe', 'r'],
			1 => ['pipe', 'w'],
			2 => ['pipe', 'w'],
		];
		$process = proc_open([$binary], $descriptors, $pipes, null, $environment);
		if (!is_resource($process)) {
			throw new RuntimeException('Unable to start the PoRE runtime.');
		}

		try {
			if (!fwrite($pipes[0], $frame)) {
				throw new RuntimeException('Unable to send command to the PoRE runtime.');
			}
			fclose($pipes[0]);

			stream_set_timeout($pipes[1], 10);
			$lengthBytes = $this->readExact($pipes[1], 4);
			if (strlen($lengthBytes) !== 4) {
				throw new RuntimeException('PoRE runtime returned an incomplete response.');
			}
			$length = unpack('Nlength', $lengthBytes)['length'];
			if ($length === 0 || $length > self::MAX_FRAME_LENGTH) {
				throw new RuntimeException('PoRE runtime returned an invalid response frame.');
			}
			$response = $this->readExact($pipes[1], $length);
			if (strlen($response) !== $length) {
				throw new RuntimeException('PoRE runtime returned an incomplete response payload.');
			}
			$decoded = json_decode($response, true, 512, JSON_THROW_ON_ERROR);
			if (!is_array($decoded)) {
				throw new RuntimeException('PoRE runtime response is not a JSON object.');
			}
			return $decoded;
		} finally {
			if (isset($pipes[1]) && is_resource($pipes[1])) fclose($pipes[1]);
			if (isset($pipes[2]) && is_resource($pipes[2])) fclose($pipes[2]);
			proc_close($process);
		}
	}

	private function readExact($stream, int $length): string {
		$result = '';
		while (strlen($result) < $length && !feof($stream)) {
			$chunk = fread($stream, $length - strlen($result));
			if ($chunk === false || $chunk === '') break;
			$result .= $chunk;
		}
		return $result;
	}
}
