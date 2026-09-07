<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCA\PoRe\AppInfo\Application;
use OCP\Files\File;
use OCP\Files\Folder;
use OCP\Files\IRootFolder;
use OCP\Files\NotFoundException;
use OCP\IConfig;
use OCP\IUserSession;
use RuntimeException;

final class NextcloudArtifactStorage {
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

		$path = NextcloudArtifactPath::build(
			$this->normalizedConfiguredRoot($user->getUID()),
			$productionId,
			$productionLabel,
			$captureId,
			$startedAt,
		);

		$userFolder = $this->rootFolder->getUserFolder($user->getUID());
		$folder = $this->ensureConfiguredRoot($userFolder, $path['root']);
		$folder = $this->ensureFolder($folder, $path['year']);
		$folder = $this->ensureFolder($folder, $path['month']);
		$folder = $this->ensureFolder($folder, $path['leaf']);

		$file = $this->getOrCreateFile($folder, $path['filename']);
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
			'path' => $path['relative_path'],
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
}
