<?php

declare(strict_types=1);

namespace OCP {
	class IConfig {
		public function getSystemValue(string $key, mixed $default = null): mixed { return $default; }
	}
}

namespace OCP\App {
	class IAppManager {
		public function getAppPath(string $appId): ?string { return null; }
	}
}

namespace {
	require_once __DIR__ . '/../lib/Service/RecordingRuntimeService.php';

	use OCA\PoRe\Service\RecordingRuntimeService;

	final class FakeConfig extends \OCP\IConfig {
		public function __construct(
			private readonly string $binaryPath,
			private readonly string $sessionStore,
		) {}

		public function getSystemValue(string $key, mixed $default = null): mixed {
			return match ($key) {
				'pore_runtime_binary' => $this->binaryPath,
				'pore_runtime_session_store' => $this->sessionStore,
				default => $default,
			};
		}
	}

	final class FakeAppManager extends \OCP\App\IAppManager {}

	function check(bool $condition, string $message): void {
		if (!$condition) throw new \RuntimeException($message);
	}

	function writeExecutable(string $content): string {
		$path = tempnam(sys_get_temp_dir(), 'pore-runtime-contract-');
		if ($path === false) throw new \RuntimeException('Unable to create runtime fixture.');
		if (file_put_contents($path, $content) === false || !chmod($path, 0700)) {
			throw new \RuntimeException('Unable to prepare runtime fixture.');
		}
		return $path;
	}

	$responseScript = writeExecutable(<<<'RUNTIME'
#!/usr/bin/php
<?php
$header = fread(STDIN, 4);
$length = unpack('Nlength', $header)['length'] ?? 0;
if ($length > 0) fread(STDIN, $length);
$payload = json_encode(['status' => 'ok']);
echo pack('N', strlen($payload)), $payload;
RUNTIME
	);

	$sleepScript = writeExecutable(<<<'RUNTIME'
#!/bin/sh
exec sleep 5
RUNTIME
	);

	$sessionStore = sys_get_temp_dir() . '/pore-runtime-store-' . bin2hex(random_bytes(8));
	if (!mkdir($sessionStore, 0700, true) && !is_dir($sessionStore)) {
		throw new \RuntimeException('Unable to create runtime store.');
	}

	$service = new RecordingRuntimeService(
		new FakeConfig($responseScript, $sessionStore),
		new FakeAppManager(),
	);

	// TEST-RT-01: stateless FLAC conversion must not wait for .runtime.lock.
	$lockPath = $sessionStore . '/.runtime.lock';
	$lock = fopen($lockPath, 'c');
	check($lock !== false && flock($lock, LOCK_EX), 'Test runtime lock could not be acquired.');

	$result = $service->command(['status' => 'ok'], 'artifact.convert_flac_to_wav', 2);
	check(($result['status'] ?? null) === 'ok', 'Lock-free runtime operation did not execute.');

	flock($lock, LOCK_UN);
	fclose($lock);

	// TEST-RT-02: a timeout must terminate a hung child promptly.
	$timeoutService = new RecordingRuntimeService(
		new FakeConfig($sleepScript, $sessionStore),
		new FakeAppManager(),
	);
	$started = microtime(true);
	try {
		$timeoutService->command(['status' => 'ok'], 'artifact.convert_flac_to_wav', 1);
		throw new \RuntimeException('Expected runtime timeout.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'PoRE runtime command timed out.', 'Unexpected runtime timeout error.');
	}
	check((microtime(true) - $started) < 3.0, 'Runtime timeout did not terminate the child promptly.');

	@unlink($responseScript);
	@unlink($sleepScript);
	@unlink($lockPath);
	@rmdir($sessionStore);

	echo "Recording runtime contract checks passed.\n";
}
