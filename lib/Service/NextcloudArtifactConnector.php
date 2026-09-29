<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCA\PoRe\AppInfo\Application;
use OCP\Constants;
use OCP\Files\File;
use OCP\Files\Folder;
use OCP\Files\IRootFolder;
use OCP\Files\NotFoundException;
use OCP\IConfig;
use OCP\Security\ISecureRandom;
use OCP\Share\IManager;
use OCP\Share\IShare;
use RuntimeException;

/**
 * Nextcloud-specific transport connector.
 *
 * The connector owns all Nextcloud mechanics: target resolution, path creation,
 * temporary public upload authorization, remote verification and share cleanup.
 * The browser never receives Nextcloud Files paths as an authority; it receives
 * only the bounded upload authorization prepared here.
 */
final class NextcloudArtifactConnector {
	private const COPY_CHUNK_SIZE = 1024 * 1024;
	private const CONFIG_STORAGE_ROOT = 'storage_root';
	private const SHARE_LIFETIME_SECONDS = 3600;
	private const V1_SAMPLE_RATE = 48000;
	private const V1_FALLBACK_SAMPLE_RATE = 44100;
	private const V1_CHANNELS = 1;
	private const V1_BITS_PER_SAMPLE = 24;
	private const PAYLOAD_FORMAT_WAV = 'audio/wav';
	private const PAYLOAD_FORMAT_FLAC = 'audio/flac';

	public function __construct(
		private readonly IRootFolder $rootFolder,
		private readonly IConfig $config,
		private readonly IManager $shareManager,
		private readonly ISecureRandom $secureRandom,
		private readonly RecordingRuntimeService $runtime,
	) {
	}

	/**
	 * @return array{transfer_id:string, upload_url:string, upload_username:string, upload_password:string, filename:string, size:int, sha256:string, upload_required:bool, target_user_id:string}
	 */
	public function prepare(
		string $productionId,
		string $productionLabel,
		string $recordingId,
		string $captureId,
		string $startedAt,
		string $participantLabel,
		int $size,
		string $sha256,
		string $actorUserId,
		string $payloadFormat = self::PAYLOAD_FORMAT_WAV,
		?int $preferredFileId = null,
	): array {
		$this->validateHash($sha256);
		if (trim($actorUserId) === '') throw new RuntimeException('Transport actor is required.');
		if ($size < 0) throw new RuntimeException('Transport payload size must not be negative.');
		if (!in_array($payloadFormat, [self::PAYLOAD_FORMAT_WAV, self::PAYLOAD_FORMAT_FLAC], true)) throw new RuntimeException('artifact_payload_format_invalid');
		if ($preferredFileId !== null && $preferredFileId <= 0) throw new RuntimeException('Preferred artifact file id must be positive.');

		$targetUserId = trim($this->config->getAppValue(
			Application::APP_ID,
			\OCA\PoRe\Controller\ProductionController::ownerKey($productionId),
			'',
		));
		if ($targetUserId === '') throw new RuntimeException('No storage target is registered for this production.');

		$userFolder = $this->rootFolder->getUserFolder($targetUserId);
		$allowFilenameReuse = $preferredFileId === null;

		if ($preferredFileId !== null) {
			$preferred = $this->findFileById($userFolder, $preferredFileId);
			if ($preferred !== null) {
				if ($payloadFormat === self::PAYLOAD_FORMAT_FLAC) {
					$this->inspectWav($preferred, $preferred->getSize());
					return $this->prepareExistingFileHandle($preferred, $targetUserId, $captureId, $preferred->getSize(), $this->hashFile($preferred), $actorUserId);
				}
				if ($preferred->getSize() !== $size || !hash_equals(strtolower($sha256), $this->hashFile($preferred))) {
					throw new RuntimeException('Nextcloud recorded artifact payload has changed.');
				}
				return $this->prepareExistingFileHandle($preferred, $targetUserId, $captureId, $size, $sha256, $actorUserId);
			}
		}

		$path = NextcloudArtifactPath::build(
			$this->normalizedConfiguredRoot($targetUserId),
			$productionId,
			$productionLabel,
			$captureId,
			$startedAt,
			$participantLabel,
		);

		$folder = $this->ensureConfiguredRoot($userFolder, $path['root']);
		$folder = $this->ensureFolder($folder, $path['year']);
		$folder = $this->ensureFolder($folder, $path['month']);
		$folder = $this->ensureFolder($folder, $path['leaf']);

		$canonicalFilename = $path['filename'];
		$transportFilename = $canonicalFilename;
		if ($payloadFormat === self::PAYLOAD_FORMAT_FLAC) {
			$transportFilename = pathinfo($canonicalFilename, PATHINFO_FILENAME) . '.flac';
			while ($this->findFile($folder, $canonicalFilename) !== null || $this->findFile($folder, $transportFilename) !== null) {
				$canonicalFilename = $this->nextFreeFilename($folder, $canonicalFilename);
				$transportFilename = pathinfo($canonicalFilename, PATHINFO_FILENAME) . '.flac';
			}
		} else {
			$existing = $this->findFile($folder, $transportFilename);
			if ($existing !== null) {
				if ($allowFilenameReuse && $existing->getSize() === $size && hash_equals(strtolower($sha256), $this->hashFile($existing))) {
					return $this->prepareExistingFileHandle($existing, $targetUserId, $captureId, $size, $sha256, $actorUserId);
				}
				$canonicalFilename = $this->nextFreeFilename($folder, $canonicalFilename);
				$transportFilename = $canonicalFilename;
			}
		}

		if (!$this->shareManager->shareApiAllowLinks() || !$this->shareManager->shareApiLinkAllowPublicUpload()) {
			throw new RuntimeException('Nextcloud public upload shares are disabled.');
		}

		$password = $this->secureRandom->generate(32, ISecureRandom::CHAR_ALPHANUMERIC);
		$expiration = new \DateTimeImmutable('now', new \DateTimeZone('UTC'));
		$expiration = $expiration->modify('+' . self::SHARE_LIFETIME_SECONDS . ' seconds');

		$share = $this->shareManager->newShare();
		$share->setNode($folder);
		$share->setShareType(IShare::TYPE_LINK);
		$share->setPermissions(Constants::PERMISSION_CREATE);
		$share->setPassword($password);
		$share->setExpirationDate(\DateTime::createFromImmutable($expiration));
		$share->setLabel('PoRE transport ' . $captureId);
		$share->setSharedBy($targetUserId);
		$share->setShareOwner($targetUserId);
		$share = $this->shareManager->createShare($share);

		$transferId = $this->secureRandom->generate(32, ISecureRandom::CHAR_ALPHANUMERIC);
		$state = [
			'transfer_id' => $transferId,
			'share_id' => $share->getId(),
			'target_user_id' => $targetUserId,
			'folder_path' => $this->relativeUserPath($folder, $targetUserId),
			'filename' => $transportFilename,
			'canonical_filename' => $canonicalFilename,
			'payload_format' => $payloadFormat,
			'capture_id' => $captureId,
			'file_id' => null,
			'size' => $size,
			'sha256' => strtolower($sha256),
			'actor_user_id' => $actorUserId,
			'upload_required' => true,
		];
		$handle = $this->encodeHandle($state);

		return [
			'transfer_id' => $handle,
			'upload_url' => '/public.php/dav/files/' . rawurlencode($share->getToken()),
			'upload_username' => 'anonymous',
			'upload_password' => $password,
			'filename' => $transportFilename,
			'canonical_filename' => $canonicalFilename,
			'payload_format' => $payloadFormat,
			'size' => $size,
			'sha256' => strtolower($sha256),
			'upload_required' => true,
			'target_user_id' => $targetUserId,
		];
	}

	/**
	 * @return array{artifact_id:string, target_user_id:string, file_id:int, path:string, size:int, sha256:string, filename:string, preservation:array{format:string,encoding:string,sampleRate:int,channels:int,bitsPerSample:int}}
	 */
	public function verify(string $handle, string $actorUserId): array {
		$state = $this->decodeHandle($handle);
		$this->assertHandleActor($state, $actorUserId);

		$file = null;
		if ($state['file_id'] !== null) {
			$userFolder = $this->rootFolder->getUserFolder($state['target_user_id']);
			$file = $this->findFileById($userFolder, $state['file_id']);
			if ($file === null) {
				throw new RuntimeException('Nextcloud transport artifact has not arrived.');
			}
		} else {
			$folder = $this->folderForState($state);
			$file = $this->findFile($folder, $state['filename']);
			if ($file === null) throw new RuntimeException('Nextcloud transport artifact has not arrived.');
		}

		$size = $file->getSize();
		if ($size !== $state['size']) throw new RuntimeException('Nextcloud transport artifact size does not match.');

		$hash = $this->hashFile($file);
		if (!hash_equals($state['sha256'], $hash)) throw new RuntimeException('Nextcloud transport artifact SHA-256 does not match.');

		if ($state['payload_format'] === self::PAYLOAD_FORMAT_FLAC) {
			$receipt = $this->convertFlacToCanonicalWav($file, $state);
			$receipt['transport'] = [
				'format' => self::PAYLOAD_FORMAT_FLAC,
				'size' => $size,
				'sha256' => $hash,
			];
			return $receipt;
		}

		$preservation = $this->inspectWav($file, $size);

		return [
			'artifact_id' => $state['capture_id'],
			'target_user_id' => $state['target_user_id'],
			'file_id' => $file->getId(),
			'path' => $this->relativeUserPath($file, $state['target_user_id']),
			'size' => $size,
			'sha256' => $hash,
			'filename' => $file->getName(),
			'preservation' => $preservation,
		];
	}

	public function close(string $handle, string $actorUserId): void {
		$state = $this->decodeHandle($handle);
		$this->assertHandleActor($state, $actorUserId);
		if ($state['share_id'] !== null) {
			try {
				$share = $this->shareManager->getShareById($state['share_id']);
				$this->shareManager->deleteShare($share);
			} catch (\OCP\Share\Exceptions\ShareNotFound) {
				// Idempotent close: expiration or an earlier cleanup is already success.
			}
		}
		if ($state['payload_format'] !== self::PAYLOAD_FORMAT_FLAC || !$state['upload_required']) return;
		$folder = $this->folderForState($state);
		$file = $this->findFile($folder, $state['filename']);
		if ($file === null) return;
		try { $file->delete(); } catch (NotFoundException) {}
	}

	/** @param File $file */
	private function prepareExistingFileHandle(
		File $file,
		string $targetUserId,
		string $captureId,
		int $size,
		string $sha256,
		string $actorUserId,
	): array {
		$transferId = $this->secureRandom->generate(32, ISecureRandom::CHAR_ALPHANUMERIC);
		$parent = $file->getParent();
		$state = [
			'transfer_id' => $transferId,
			'share_id' => null,
			'target_user_id' => $targetUserId,
			'folder_path' => $this->relativeUserPath($parent, $targetUserId),
			'filename' => $file->getName(),
			'capture_id' => $captureId,
			'file_id' => $file->getId(),
			'size' => $size,
			'sha256' => strtolower($sha256),
			'actor_user_id' => $actorUserId,
			'upload_required' => false,
		];
		return [
			'transfer_id' => $this->encodeHandle($state),
			'upload_url' => '',
			'upload_username' => '',
			'upload_password' => '',
			'filename' => $file->getName(),
			'size' => $size,
			'sha256' => strtolower($sha256),
			'upload_required' => false,
			'target_user_id' => $targetUserId,
		];
	}

	/** @param array<string,mixed> $state */
	private function encodeHandle(array $state): string {
		$payload = $this->base64UrlEncode(json_encode($state, JSON_THROW_ON_ERROR));
		$signature = hash_hmac('sha256', $payload, $this->config->getSystemValueString('secret'));
		return $payload . '.' . $signature;
	}

	/** @return array{transfer_id:string,share_id:string|null,target_user_id:string,folder_path:string,filename:string,canonical_filename:string,payload_format:string,capture_id:string,file_id:int|null,size:int,sha256:string,actor_user_id:string,upload_required:bool} */
	private function decodeHandle(string $handle): array {
		$parts = explode('.', $handle, 2);
		if (count($parts) !== 2) throw new RuntimeException('Invalid transport handle.');
		[$payload, $signature] = $parts;
		$expected = hash_hmac('sha256', $payload, $this->config->getSystemValueString('secret'));
		if (!hash_equals($expected, $signature)) throw new RuntimeException('Invalid transport handle signature.');

		$decoded = json_decode($this->base64UrlDecode($payload), true, 512, JSON_THROW_ON_ERROR);
		if (!is_array($decoded)) throw new RuntimeException('Invalid transport handle payload.');
		foreach (['transfer_id', 'share_id', 'target_user_id', 'folder_path', 'filename', 'capture_id', 'size', 'sha256', 'actor_user_id'] as $key) {
			if (!array_key_exists($key, $decoded)) throw new RuntimeException('Incomplete transport handle.');
		}
		$fileId = $decoded['file_id'] ?? null;
		$canonicalFilename = $decoded['canonical_filename'] ?? $decoded['filename'];
		$payloadFormat = $decoded['payload_format'] ?? self::PAYLOAD_FORMAT_WAV;
		$uploadRequired = $decoded['upload_required'] ?? true;

		$size = $decoded['size'];
		if ((!is_int($size) && !is_float($size) && !is_string($size)) || (int)$size < 0) {
			throw new RuntimeException('Invalid transport handle payload.');
		}
		if ($fileId !== null && (!is_int($fileId) || $fileId <= 0)) {
			throw new RuntimeException('Invalid transport handle payload.');
		}
		if (!is_bool($uploadRequired)) throw new RuntimeException('Invalid transport handle payload.');
		if (!is_string($canonicalFilename) || trim($canonicalFilename) === '') throw new RuntimeException('Invalid transport handle payload.');
		if (!is_string($payloadFormat) || !in_array($payloadFormat, [self::PAYLOAD_FORMAT_FLAC, self::PAYLOAD_FORMAT_WAV], true)) throw new RuntimeException('Invalid transport handle payload.');

		return [
			'transfer_id' => (string)$decoded['transfer_id'],
			'share_id' => $decoded['share_id'] === null ? null : (string)$decoded['share_id'],
			'target_user_id' => (string)$decoded['target_user_id'],
			'folder_path' => (string)$decoded['folder_path'],
			'filename' => (string)$decoded['filename'],
			'canonical_filename' => (string)$canonicalFilename,
			'payload_format' => (string)$payloadFormat,
			'capture_id' => (string)$decoded['capture_id'],
			'file_id' => $fileId,
			'size' => (int)$size,
			'sha256' => strtolower((string)$decoded['sha256']),
			'actor_user_id' => (string)$decoded['actor_user_id'],
			'upload_required' => $uploadRequired,
		];
	}

	/** @param array{actor_user_id:string} $state */
	private function assertHandleActor(array $state, string $actorUserId): void {
		if (trim($actorUserId) === '' || !hash_equals($state['actor_user_id'], $actorUserId)) {
			throw new RuntimeException('Transport handle is not authorized for this user.');
		}
	}

	private function nextFreeFilename(Folder $folder, string $filename): string {
		$extension = pathinfo($filename, PATHINFO_EXTENSION);
		$stem = pathinfo($filename, PATHINFO_FILENAME);
		for ($suffix = 2; ; $suffix++) {
			$candidate = $stem . ' (' . $suffix . ')' . ($extension !== '' ? '.' . $extension : '');
			if ($this->findFile($folder, $candidate) === null) return $candidate;
		}
	}

	/** @param array{target_user_id:string,folder_path:string} $state */
	private function folderForState(array $state): Folder {
		$folder = $this->rootFolder->getUserFolder($state['target_user_id']);
		foreach (array_filter(explode('/', trim($state['folder_path'], '/')), static fn (string $segment): bool => $segment !== '') as $segment) {
			$node = $folder->get($segment);
			if (!$node instanceof Folder) throw new RuntimeException('Nextcloud transport destination is no longer a folder.');
			$folder = $node;
		}
		return $folder;
	}

	private function relativeUserPath($node, string $userId): string {
		$prefix = '/files/' . $userId . '/';
		$path = $node->getPath();
		$position = strpos($path, $prefix);
		return $position === false ? ltrim($path, '/') : substr($path, $position + strlen($prefix));
	}

	private function normalizedConfiguredRoot(string $userId): string {
		return NextcloudArtifactPath::normalizeRoot($this->config->getUserValue($userId, Application::APP_ID, self::CONFIG_STORAGE_ROOT, ''));
	}

	private function ensureConfiguredRoot(Folder $userFolder, string $configured): Folder {
		$folder = $userFolder;
		foreach (explode('/', $configured) as $segment) $folder = $this->ensureFolder($folder, $segment);
		return $folder;
	}

	private function ensureFolder(Folder $parent, string $name): Folder {
		try {
			$node = $parent->get($name);
			if (!$node instanceof Folder) throw new RuntimeException(sprintf('Nextcloud path component "%s" is not a folder.', $name));
			return $node;
		} catch (NotFoundException) {
			return $parent->newFolder($name);
		}
	}

	private function findFile(Folder $folder, string $filename): ?File {
		try {
			$node = $folder->get($filename);
			return $node instanceof File ? $node : null;
		} catch (NotFoundException) {
			return null;
		}
	}

	private function findFileById(Folder $folder, int $fileId): ?File {
		try {
			$node = $folder->getFirstNodeById($fileId);
			return $node instanceof File ? $node : null;
		} catch (NotFoundException) {
			return null;
		}
	}

	private function convertFlacToCanonicalWav(File $flacFile, array $state): array {
		$inputPath = tempnam(sys_get_temp_dir(), 'pore-flac-in-');
		$outputPath = tempnam(sys_get_temp_dir(), 'pore-flac-out-');
		if ($inputPath === false || $outputPath === false) {
			if ($inputPath !== false) @unlink($inputPath);
			if ($outputPath !== false) @unlink($outputPath);
			throw new RuntimeException('artifact_preservation_invalid');
		}
		$canonicalFile = null;
		try {
			$input = $flacFile->fopen('r');
			$local = fopen($inputPath, 'wb');
			if ($input === false || $local === false) throw new RuntimeException('artifact_preservation_invalid');
			stream_copy_to_stream($input, $local);
			fclose($input); fclose($local);

			$response = $this->runtime->command([
				'input_path' => $inputPath,
				'output_path' => $outputPath,
				'expected_sample_rate_hz' => 0,
				'expected_channels' => self::V1_CHANNELS,
				'expected_bits_per_sample' => self::V1_BITS_PER_SAMPLE,
			], 'artifact.convert_flac_to_wav');

			$rate = $response['sample_rate_hz'] ?? null;
			$channels = $response['channels'] ?? null;
			$bits = $response['bits_per_sample'] ?? null;
			$count = $response['sample_count'] ?? null;
			$length = $response['payload_length'] ?? null;
			if (($response['status'] ?? null) !== 'converted'
				|| !is_int($rate) || !in_array($rate, [self::V1_SAMPLE_RATE, self::V1_FALLBACK_SAMPLE_RATE], true)
				|| $channels !== self::V1_CHANNELS || $bits !== self::V1_BITS_PER_SAMPLE
				|| !is_int($count) || $count <= 0 || !is_int($length) || $length !== 44 + ($count * 3)) {
				throw new RuntimeException('artifact_preservation_invalid');
			}
			$localSize = filesize($outputPath);
			$localHash = hash_file('sha256', $outputPath);
			if ($localSize === false || $localSize !== $length || $localHash === false) throw new RuntimeException('artifact_preservation_invalid');

			$folder = $this->folderForState($state);
			$name = $state['canonical_filename'];
			$existing = $this->findFile($folder, $name);
			if ($existing !== null) {
				if ($existing->getSize() === $localSize && hash_equals($localHash, $this->hashFile($existing))) {
					$preservation = $this->inspectWav($existing, $existing->getSize());
					return [
						'artifact_id' => $state['capture_id'], 'target_user_id' => $state['target_user_id'],
						'file_id' => $existing->getId(), 'path' => $this->relativeUserPath($existing, $state['target_user_id']),
						'size' => $existing->getSize(), 'sha256' => $localHash, 'filename' => $existing->getName(),
						'preservation' => $preservation,
					];
				}
				throw new RuntimeException('artifact_manifest_conflict');
			}
			$canonicalFile = $folder->newFile($name);
			$source = fopen($outputPath, 'rb');
			$destination = $canonicalFile->fopen('w');
			if ($source === false || $destination === false) throw new RuntimeException('artifact_preservation_invalid');
			if (stream_copy_to_stream($source, $destination) === false) throw new RuntimeException('artifact_preservation_invalid');
			fclose($source); fclose($destination);

			$size = $canonicalFile->getSize();
			$hash = $this->hashFile($canonicalFile);
			$preservation = $this->inspectWav($canonicalFile, $size);
			if ($size !== $localSize || !hash_equals($localHash, $hash)) throw new RuntimeException('artifact_preservation_invalid');

			return [
				'artifact_id' => $state['capture_id'], 'target_user_id' => $state['target_user_id'],
				'file_id' => $canonicalFile->getId(), 'path' => $this->relativeUserPath($canonicalFile, $state['target_user_id']),
				'size' => $size, 'sha256' => $hash, 'filename' => $canonicalFile->getName(), 'preservation' => $preservation,
			];
		} catch (RuntimeException $error) {
			if ($canonicalFile !== null) { try { $canonicalFile->delete(); } catch (\Throwable) {} }
			throw $error;
		} catch (\Throwable $error) {
			if ($canonicalFile !== null) { try { $canonicalFile->delete(); } catch (\Throwable) {} }
			throw new RuntimeException('artifact_preservation_invalid', 0, $error);
		} finally {
			@unlink($inputPath); @unlink($outputPath);
		}
	}

	private function hashFile(File $file): string {
		$input = $file->fopen('r');
		if ($input === false) throw new RuntimeException('Unable to read stored Nextcloud artifact.');
		$context = hash_init('sha256');
		try {
			while (!feof($input)) {
				$chunk = fread($input, self::COPY_CHUNK_SIZE);
				if ($chunk === false) throw new RuntimeException('Unable to read stored Nextcloud artifact.');
				if ($chunk !== '') hash_update($context, $chunk);
			}
		} finally {
			fclose($input);
		}
		return hash_final($context);
	}

	/**
	 * V1 transport artifacts are canonical RIFF/WAVE PCM files with the
	 * 44-byte header produced by the browser completion job.
	 *
	 * @return array{format:string,encoding:string,sampleRate:int,channels:int,bitsPerSample:int}
	 */
	private function inspectWav(File $file, int $size): array {
		if ($size < 44) throw new RuntimeException('Nextcloud transport artifact is not a valid WAV container.');
		$input = $file->fopen('r');
		if ($input === false) throw new RuntimeException('Unable to read stored Nextcloud artifact.');
		try {
			$header = fread($input, 44);
		} finally {
			fclose($input);
		}

		if ($header === false || strlen($header) !== 44
			|| substr($header, 0, 4) !== 'RIFF'
			|| substr($header, 8, 4) !== 'WAVE'
			|| substr($header, 12, 4) !== 'fmt ') {
			throw new RuntimeException('Nextcloud transport artifact is not a valid WAV container.');
		}

		$riffLength = unpack('V', substr($header, 4, 4))[1] ?? 0;
		$fmtSize = unpack('V', substr($header, 16, 4))[1] ?? 0;
		$audioFormat = unpack('v', substr($header, 20, 2))[1] ?? 0;
		$channels = unpack('v', substr($header, 22, 2))[1] ?? 0;
		$sampleRate = unpack('V', substr($header, 24, 4))[1] ?? 0;
		$byteRate = unpack('V', substr($header, 28, 4))[1] ?? 0;
		$blockAlign = unpack('v', substr($header, 32, 2))[1] ?? 0;
		$bitsPerSample = unpack('v', substr($header, 34, 2))[1] ?? 0;
		$dataLength = unpack('V', substr($header, 40, 4))[1] ?? 0;

		$validSampleRate = in_array($sampleRate, [self::V1_SAMPLE_RATE, self::V1_FALLBACK_SAMPLE_RATE], true);
		if ($riffLength !== $size - 8
			|| $fmtSize !== 16
			|| $audioFormat !== 1
			|| $channels !== self::V1_CHANNELS
			|| !$validSampleRate
			|| $bitsPerSample !== self::V1_BITS_PER_SAMPLE
			|| $blockAlign !== self::V1_CHANNELS * 3
			|| $byteRate !== $sampleRate * $blockAlign
			|| substr($header, 36, 4) !== 'data'
			|| $dataLength !== $size - 44
			|| $dataLength < 0
			|| $dataLength % $blockAlign !== 0) {
			throw new RuntimeException('Nextcloud transport artifact is not a supported PoRE PCM WAV.');
		}

		return [
			'format' => 'audio/wav',
			'encoding' => 'pcm_s24le',
			'sampleRate' => $sampleRate,
			'channels' => self::V1_CHANNELS,
			'bitsPerSample' => self::V1_BITS_PER_SAMPLE,
		];
	}

	private function validateHash(string $hash): void {
		if (!preg_match('/^[a-f0-9]{64}$/i', $hash)) throw new RuntimeException('Transport SHA-256 must be a SHA-256 hex digest.');
	}

	private function base64UrlEncode(string $value): string {
		return rtrim(strtr(base64_encode($value), '+/', '-_'), '=');
	}

	private function base64UrlDecode(string $value): string {
		$decoded = base64_decode(strtr($value, '-_', '+/') . str_repeat('=', (4 - strlen($value) % 4) % 4), true);
		if ($decoded === false) throw new RuntimeException('Invalid transport handle encoding.');
		return $decoded;
	}
}
