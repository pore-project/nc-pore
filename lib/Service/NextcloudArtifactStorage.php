<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCP\Files\File;
use OCP\Files\Folder;
use OCP\Files\IRootFolder;
use OCP\Files\NotFoundException;
use OCP\IUserSession;
use RuntimeException;

final class NextcloudArtifactStorage {
	private const ROOT_FOLDER = 'PoRE';
	private const COPY_CHUNK_SIZE = 1024 * 1024;

	public function __construct(
		private readonly IRootFolder $rootFolder,
		private readonly IUserSession $userSession,
	) {
	}

	/**
	 * @return array{file_id:int, path:string, size:int, sha256:string}
	 */
	public function storeFinalizedArtifact(
		string $productionId,
		string $recordingId,
		string $captureId,
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
		$this->assertPathSegment($recordingId);
		$this->assertPathSegment($captureId);

		$userFolder = $this->rootFolder->getUserFolder($user->getUID());
		$folder = $this->ensureFolder($userFolder, self::ROOT_FOLDER);
		$folder = $this->ensureFolder($folder, 'Productions');
		$folder = $this->ensureFolder($folder, $productionId);
		$folder = $this->ensureFolder($folder, 'Recordings');
		$folder = $this->ensureFolder($folder, $recordingId);

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

		$storedHash = hash_final(hash_init('sha256'));
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
			'path' => self::ROOT_FOLDER . '/Productions/' . $productionId . '/Recordings/' . $recordingId . '/' . $filename,
			'size' => $storedSize,
			'sha256' => $storedHash,
		];
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
		if ($value === '' || $value === '.' || $value === '..' || strpbrk($value, '/\\') !== false) {
			throw new RuntimeException('Invalid path segment in finalized artifact identity.');
		}
	}
}
