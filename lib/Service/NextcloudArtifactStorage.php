<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCA\PoRe\AppInfo\Application;
use OCP\Files\File;
use OCP\Files\Folder;
use OCP\Files\IRootFolder;
use OCP\Files\NotFoundException;
use OCP\IConfig;
use RuntimeException;

final class NextcloudArtifactStorage {
	private const COPY_CHUNK_SIZE = 1024 * 1024;
	private const CONFIG_STORAGE_ROOT = 'storage_root';

	public function __construct(
		private readonly IRootFolder $rootFolder,
		private readonly IConfig $config,
	) {
	}

	/**
	 * Store the finalized artifact strictly below the production owner's Files root.
	 *
	 * The configured storage root is a Nextcloud Files-relative path, never a server
	 * filesystem path. A configured root such as "Büro/interviews" is the complete
	 * PoRe root; "audio" is used only when no root has been configured.
	 *
	 * @return array{file_id:int, path:string, size:int, sha256:string}
	 */
	public function storeFinalizedArtifact(
		string $productionId,
		string $productionLabel,
		string $recordingId,
		string $captureId,
		string $startedAt,
		string $participantLabel,
		string $targetUserId,
		string $payloadPath,
		int $payloadLength,
	): array {
		if (trim($targetUserId) === '') {
			throw new RuntimeException('A storage target user is required.');
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

		$path = NextcloudArtifactPath::build(
			$this->normalizedConfiguredRoot($targetUserId),
			$productionId,
			$productionLabel,
			$captureId,
			$startedAt,
			$participantLabel,
		);

		$userFolder = $this->rootFolder->getUserFolder($targetUserId);
		$folder = $this->ensureConfiguredRoot($userFolder, $path['root']);
		$folder = $this->ensureFolder($folder, $path['year']);
		$folder = $this->ensureFolder($folder, $path['month']);
		$folder = $this->ensureFolder($folder, $path['leaf']);

		$file = $this->getAvailableFile($folder, $path['filename'], $inputHash);
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
			'path' => str_replace($path['filename'], $file->getName(), $path['relative_path']),
			'size' => $storedSize,
			'sha256' => $storedHash,
		];
	}

	private function ensureConfiguredRoot(Folder $userFolder, string $configured): Folder {
		$folder = $userFolder;
		foreach (explode('/', $configured) as $segment) {
			$folder = $this->ensureFolder($folder, $segment);
		}
		return $folder;
	}

	private function normalizedConfiguredRoot(string $userId): string {
		return NextcloudArtifactPath::normalizeRoot($this->config->getUserValue(
			$userId,
			Application::APP_ID,
			self::CONFIG_STORAGE_ROOT,
			'',
		));
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

	private function getAvailableFile(Folder $folder, string $filename, string $expectedHash): File {
		$extension = '.wav';
		$stem = str_ends_with($filename, $extension) ? substr($filename, 0, -strlen($extension)) : $filename;
		$candidate = $filename;
		$suffix = 1;
		while (true) {
			try {
				$node = $folder->get($candidate);
				if (!$node instanceof File) {
					throw new RuntimeException(sprintf('Nextcloud destination "%s" is not a file.', $candidate));
				}
				$existingHash = $this->hashFile($node);
				if (hash_equals($expectedHash, $existingHash)) return $node;
				$suffix += 1;
				$candidate = sprintf('%s (%d)%s', $stem, $suffix, $extension);
			} catch (NotFoundException) {
				return $folder->newFile($candidate);
			}
		}
	}

	private function hashFile(File $file): string {
		$input = $file->fopen('r');
		if ($input === false) throw new RuntimeException('Unable to read existing Nextcloud artifact.');
		$context = hash_init('sha256');
		try {
			while (!feof($input)) {
				$chunk = fread($input, self::COPY_CHUNK_SIZE);
				if ($chunk === false) throw new RuntimeException('Unable to read existing Nextcloud artifact.');
				if ($chunk !== '') hash_update($context, $chunk);
			}
		} finally {
			fclose($input);
		}
		return hash_final($context);
	}
}
