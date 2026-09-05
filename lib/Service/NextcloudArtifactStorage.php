<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCP\App\IAppManager;
use OCP\Files\File;
use OCP\Files\Folder;
use OCP\Files\IRootFolder;
use OCP\Files\NotFoundException;
use OCP\IConfig;
use OCP\IUserSession;
use RuntimeException;

final class NextcloudArtifactStorage {
	private const DEFAULT_STORAGE_ROOT = 'audio';
	private const AUDIO_FOLDER = 'audio';
	private const COPY_CHUNK_SIZE = 1024 * 1024;
	private const CONFIG_STORAGE_ROOT = 'storage_root';

	public function __construct(
		private readonly IRootFolder $rootFolder,
		private readonly IUserSession $userSession,
		private readonly IConfig $config,
	) {
	}

	/**
	 * Store the finalized artifact strictly below the authenticated user's Files root.
	 *
	 * The configured storage root is a Nextcloud Files-relative path, never a server
	 * filesystem path. For example: "Büro/interviews" becomes
	 * "Büro/interviews/audio/YYYY/MM/..." inside the current user's Files tree.
	 *
	 * @return array{file_id:int, path:string, size:int, sha256:string}
	 */
	public function storeFinalizedArtifact(
		string $productionId,
		string $productionLabel,
		string $recordingId,
		string $captureId,
		string $startedAt,
		string $payloadPath,
		int $payloadLength,
	): array {
		$user = $this->userSession->getUser();
		if ($user === null) {
			throw new RuntimeException('No authenticated Nextcloud user is available.');
		}
		if (!is_file($payloadPath) || !is_readable($payloadPath)) {
			throw new RuntimeException('Finalized artifact payload is not readable.');
		}

		$actualLength = filesize($payloadPath);
		if ($actualLength === false || (int)$actualLength !== $payloadLength) {
			throw new RuntimeException('Finalized artifact size changed before storage.');
		}

		$inputHash = hash_file('sha256', $payloadPath);
		if ($inputHash === false) {
			throw new RuntimeException('Unable to calculate finalized artifact hash.');
		}

		$this->assertPathSegment($productionId);
		$this->assertPathSegment($productionLabel);
		$this->assertPathSegment($recordingId);
		$this->assertPathSegment($captureId);

		$timestamp = $this->parseStartedAt($startedAt);
		$year = $timestamp->format('Y');
		$month = $timestamp->format('m');
		$dayAndTime = $timestamp->format('d - H:i');
		$leaf = $dayAndTime . ' ' . $productionLabel . ' - ' . $productionId;

		$userFolder = $this->rootFolder->getUserFolder($user->getUID());
		$folder = $this->ensureConfiguredRoot($userFolder);
		$folder = $this->ensureFolder($folder, self::AUDIO_FOLDER);
		$folder = $this->ensureFolder($folder, $year);
		$folder = $this->ensureFolder($folder, $month);
		$folder = $this->ensureFolder($folder, $leaf);

		$filename = $captureId . '.wav';
		$file = $this->getOrCreateFile($folder, $filename);
		$output = $file->fopen('w');
		if ($output === false) {
			throw new RuntimeException('Unable to open Nextcloud destination for writing.');
		}

		$input = fopen($payloadPath, 'rb');
		if ($input === false) {
			fclose($output);
			throw new RuntimeException('Unable to open finalized artifact payload.');
		}

		try {
			while (!feof($input)) {
				$chunk = fread($input, self::COPY_CHUNK_SIZE);
				if ($chunk === false) {
					throw new RuntimeException('Unable to read finalized artifact payload.');
				}
				if ($chunk !== '' && fwrite($output, $chunk) !== strlen($chunk)) {
					throw new RuntimeException('Unable to write finalized artifact to Nextcloud.');
				}
			}
		} finally {
			fclose($input);
			fclose($output);
		}

		$storedSize = $file->getSize();
		if ($storedSize !== $payloadLength) {
			throw new RuntimeException('Nextcloud stored size does not match the finalized artifact.');
		}

		$storedInput = $file->fopen('r');
		if ($storedInput === false) {
			throw new RuntimeException('Unable to reopen stored Nextcloud artifact.');
		}
		$hashContext = hash_init('sha256');
		try {
			while (!feof($storedInput)) {
				$chunk = fread($storedInput, self::COPY_CHUNK_SIZE);
				if ($chunk === false) {
					throw new RuntimeException('Unable to read stored Nextcloud artifact.');
				}
				if ($chunk !== '') {
					hash_update($hashContext, $chunk);
				}
			}
		} finally {
			fclose($storedInput);
		}
		$storedHash = hash_final($hashContext);

		if (!hash_equals($inputHash, $storedHash)) {
			throw new RuntimeException('Nextcloud stored artifact hash does not match the finalized artifact.');
		}

		return [
			'file_id' => $file->getId(),
			'path' => $folder->getPath() . '/' . $filename,
			'size' => $storedSize,
			'sha256' => $storedHash,
		];
	}

	private function ensureConfiguredRoot(Folder $userFolder): Folder {
		$configured = trim($this->config->getAppValue('pore', self::CONFIG_STORAGE_ROOT, self::DEFAULT_STORAGE_ROOT));
		if ($configured === '') {
			$configured = self::DEFAULT_STORAGE_ROOT;
		}

		$segments = preg_split('#[\\/]+#', trim($configured, "\\/"), -1, PREG_SPLIT_NO_EMPTY);
		if ($segments === false || $segments === []) {
			throw new RuntimeException('Configured PoRE storage root is invalid.');
		}

		$folder = $userFolder;
		foreach ($segments as $segment) {
			$this->assertPathSegment($segment);
			$folder = $this->ensureFolder($folder, $segment);
		}
		return $folder;
	}

	private function ensureFolder(Folder $parent, string $name): Folder {
		try {
			$node = $parent->get($name);
			if (!$node instanceof Folder) {
				throw new RuntimeException(sprintf('Nextcloud path component "%s" is not a folder.', $name));
			}
			return $node;
		} catch (NotFoundException) {
			return $parent->newFolder($name);
		}
	}

	private function getOrCreateFile(Folder $folder, string $filename): File {
		try {
			$node = $folder->get($filename);
			if (!$node instanceof File) {
				throw new RuntimeException(sprintf('Nextcloud destination "%s" is not a file.', $filename));
			}
			return $node;
		} catch (NotFoundException) {
			return $folder->newFile($filename);
		}
	}

	private function assertPathSegment(string $value): void {
		if ($value === '' || $value === '.' || $value === '..' || strpbrk($value, '/\\') !== false || str_contains($value, "\0")) {
			throw new RuntimeException('Invalid path segment in finalized artifact identity.');
		}
	}

	private function parseStartedAt(string $startedAt): \DateTimeImmutable {
		try {
			return new \DateTimeImmutable($startedAt);
		} catch (\Exception) {
			throw new RuntimeException('Invalid recording start timestamp.');
		}
	}
}
