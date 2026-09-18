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

	public function __construct(
		private readonly IRootFolder $rootFolder,
		private readonly IConfig $config,
		private readonly IManager $shareManager,
		private readonly ISecureRandom $secureRandom,
	) {
	}

	/**
	 * @return array{transfer_id:string, upload_url:string, upload_username:string, upload_password:string, filename:string, size:int, sha256:string, upload_required:bool}
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
	): array {
		$this->validateHash($sha256);
		if ($size < 0) throw new RuntimeException('Transport payload size must not be negative.');

		$targetUserId = trim($this->config->getAppValue(
			Application::APP_ID,
			\OCA\PoRe\Controller\ProductionController::ownerKey($productionId),
			'',
		));
		if ($targetUserId === '') throw new RuntimeException('No storage target is registered for this production.');

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

		$existing = $this->findFile($folder, $path['filename']);
		if ($existing !== null) {
			if ($existing->getSize() === $size && hash_equals(strtolower($sha256), $this->hashFile($existing))) {
				$transferId = $this->secureRandom->generate(32, ISecureRandom::CHAR_ALPHANUMERIC);
				$state = [
					'transfer_id' => $transferId,
					'share_id' => null,
					'target_user_id' => $targetUserId,
					'folder_path' => $this->relativeUserPath($folder, $targetUserId),
					'filename' => $path['filename'],
					'capture_id' => $captureId,
					'size' => $size,
					'sha256' => strtolower($sha256),
					'upload_required' => false,
				];
				$handle = $this->encodeHandle($state);
				return [
					'transfer_id' => $handle,
					'upload_url' => '',
					'upload_username' => '',
					'upload_password' => '',
					'filename' => $path['filename'],
					'size' => $size,
					'sha256' => strtolower($sha256),
					'upload_required' => false,
				];
			}
			$path['filename'] = $this->nextFreeFilename($folder, $path['filename']);
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
			'filename' => $path['filename'],
			'capture_id' => $captureId,
			'size' => $size,
			'sha256' => strtolower($sha256),
			'upload_required' => true,
		];
		$handle = $this->encodeHandle($state);

		return [
			'transfer_id' => $handle,
			'upload_url' => '/public.php/dav/files/' . rawurlencode($share->getToken()),
			'upload_username' => 'anonymous',
			'upload_password' => $password,
			'filename' => $path['filename'],
			'size' => $size,
			'sha256' => strtolower($sha256),
			'upload_required' => true,
		];
	}

	/**
	 * @return array{artifact_id:string, file_id:int, path:string, size:int, sha256:string}
	 */
	public function verify(string $handle): array {
		$state = $this->decodeHandle($handle);
		$folder = $this->folderForState($state);
		$file = $this->findFile($folder, $state['filename']);
		if ($file === null) throw new RuntimeException('Nextcloud transport artifact has not arrived.');

		$size = $file->getSize();
		if ($size !== $state['size']) throw new RuntimeException('Nextcloud transport artifact size does not match.');

		$hash = $this->hashFile($file);
		if (!hash_equals($state['sha256'], $hash)) throw new RuntimeException('Nextcloud transport artifact SHA-256 does not match.');

		return [
			'artifact_id' => $state['capture_id'],
			'file_id' => $file->getId(),
			'path' => $this->relativeUserPath($file, $state['target_user_id']),
			'size' => $size,
			'sha256' => $hash,
		];
	}

	public function close(string $handle): void {
		$state = $this->decodeHandle($handle);
		if ($state['share_id'] === null) return;
		try {
			$share = $this->shareManager->getShareById($state['share_id']);
			$this->shareManager->deleteShare($share);
		} catch (\OCP\Share\Exceptions\ShareNotFound) {
			// Idempotent close: expiration or an earlier cleanup is already success.
		}
	}

	/** @param array<string,mixed> $state */
	private function encodeHandle(array $state): string {
		$payload = $this->base64UrlEncode(json_encode($state, JSON_THROW_ON_ERROR));
		$signature = hash_hmac('sha256', $payload, $this->config->getSystemValueString('secret'));
		return $payload . '.' . $signature;
	}

	/** @return array{transfer_id:string,share_id:string,target_user_id:string,folder_path:string,filename:string,capture_id:string,size:int,sha256:string} */
	private function decodeHandle(string $handle): array {
		$parts = explode('.', $handle, 2);
		if (count($parts) !== 2) throw new RuntimeException('Invalid transport handle.');
		[$payload, $signature] = $parts;
		$expected = hash_hmac('sha256', $payload, $this->config->getSystemValueString('secret'));
		if (!hash_equals($expected, $signature)) throw new RuntimeException('Invalid transport handle signature.');
		$decoded = json_decode($this->base64UrlDecode($payload), true, 512, JSON_THROW_ON_ERROR);
		if (!is_array($decoded)) throw new RuntimeException('Invalid transport handle payload.');
		foreach (['transfer_id','share_id','target_user_id','folder_path','filename','capture_id','size','sha256'] as $key) {
			if (!array_key_exists($key, $decoded)) throw new RuntimeException('Incomplete transport handle.');
		}
		return [
			'transfer_id' => (string)$decoded['transfer_id'],
			'share_id' => $decoded['share_id'] === null ? null : (string)$decoded['share_id'],
			'target_user_id' => (string)$decoded['target_user_id'],
			'folder_path' => (string)$decoded['folder_path'],
			'filename' => (string)$decoded['filename'],
			'capture_id' => (string)$decoded['capture_id'],
			'size' => (int)$decoded['size'],
			'sha256' => strtolower((string)$decoded['sha256']),
			'upload_required' => !array_key_exists('upload_required', $decoded) || (bool)$decoded['upload_required'],
		];
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
