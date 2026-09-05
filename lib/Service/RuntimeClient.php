<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCP\IConfig;
use RuntimeException;

final class RuntimeClient {
	private const PROTOCOL_VERSION = 1;
	private const OPERATION = 'recording.submit_finalized_artifact';
	private const MAX_HEADER_LENGTH = 1024 * 1024;
	private const MAX_RESPONSE_LENGTH = 1024 * 1024;
	private const CHUNK_SIZE = 1024 * 1024;

	public function __construct(private readonly IConfig $config) {
	}

	/**
	 * @param array<string, mixed> $metadata
	 * @return array{protocol_version:int, request_id:string, status:string, artifact_id:?string, error_code:?string}
	 */
	public function submitFinalizedArtifact(array $metadata, string $payloadPath, int $payloadLength): array {
		if (!is_file($payloadPath) || !is_readable($payloadPath)) {
			throw new RuntimeException('Finalized artifact payload is not readable.');
		}

		$runtimeBinary = $this->config->getAppValue('pore', 'runtime_binary', '');
		$persistenceRoot = $this->config->getAppValue('pore', 'runtime_persistence_root', '');
		if ($runtimeBinary === '' || $persistenceRoot === '') {
			throw new RuntimeException('NC-PoRe runtime is not configured.');
		}
		if (!is_file($runtimeBinary) || !is_executable($runtimeBinary)) {
			throw new RuntimeException('NC-PoRe runtime binary is not executable.');
		}

		$metadata['protocol_version'] = self::PROTOCOL_VERSION;
		$metadata['operation'] = self::OPERATION;
		$metadata['payload_length'] = $payloadLength;

		$header = json_encode($metadata, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
		if (strlen($header) > self::MAX_HEADER_LENGTH) {
			throw new RuntimeException('Runtime request header is too large.');
		}

		$descriptors = [
			0 => ['pipe', 'r'],
			1 => ['pipe', 'w'],
			2 => ['pipe', 'w'],
		];
		$process = proc_open(
			[$runtimeBinary],
			$descriptors,
			$pipes,
			null,
			[
				'PORE_PERSISTENCE_ROOT' => $persistenceRoot,
			],
		);
		if (!is_resource($process)) {
			throw new RuntimeException('Unable to start NC-PoRE runtime.');
		}

		try {
			$this->writeAll($pipes[0], pack('N', strlen($header)) . $header);
			$payload = fopen($payloadPath, 'rb');
			if ($payload === false) {
				throw new RuntimeException('Unable to open finalized artifact payload.');
			}
			try {
				while (!feof($payload)) {
					$chunk = fread($payload, self::CHUNK_SIZE);
					if ($chunk === false) {
						throw new RuntimeException('Unable to read finalized artifact payload.');
					}
					if ($chunk !== '') {
						$this->writeAll($pipes[0], $chunk);
					}
				}
			} finally {
				fclose($payload);
			}
			fclose($pipes[0]);

			$lengthBytes = $this->readExact($pipes[1], 4);
			$responseLength = unpack('Nlength', $lengthBytes)['length'];
			if ($responseLength < 1 || $responseLength > self::MAX_RESPONSE_LENGTH) {
				throw new RuntimeException('Runtime response length is invalid.');
			}
			$responseJson = $this->readExact($pipes[1], $responseLength);
			$response = json_decode($responseJson, true, 512, JSON_THROW_ON_ERROR);
			fclose($pipes[1]);
			$stderr = stream_get_contents($pipes[2]);
			fclose($pipes[2]);
			$exitCode = proc_close($process);

			if ($exitCode !== 0) {
				$message = trim((string)$stderr);
				throw new RuntimeException($message !== '' ? $message : 'NC-PoRE runtime failed.');
			}

			if (!is_array($response)
				|| !isset($response['protocol_version'], $response['request_id'], $response['status'])) {
				throw new RuntimeException('Runtime returned an invalid response.');
			}

			return $response;
		} catch (\Throwable $exception) {
			foreach ($pipes as $pipe) {
				if (is_resource($pipe)) {
					fclose($pipe);
				}
			}
			proc_terminate($process);
			proc_close($process);
			throw $exception;
		}
	}

	/** @param resource $stream */
	private function writeAll($stream, string $data): void {
		$offset = 0;
		$length = strlen($data);
		while ($offset < $length) {
			$written = fwrite($stream, substr($data, $offset));
			if ($written === false || $written === 0) {
				throw new RuntimeException('Unable to write to NC-PoRE runtime.');
			}
			$offset += $written;
		}
	}

	/** @param resource $stream */
	private function readExact($stream, int $length): string {
		$result = '';
		while (strlen($result) < $length) {
			$chunk = fread($stream, $length - strlen($result));
			if ($chunk === false || $chunk === '') {
				throw new RuntimeException('Unexpected end of NC-PoRE runtime response.');
			}
			$result .= $chunk;
		}
		return $result;
	}
}
