<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IConfig;
use OCP\IRequest;
use OCP\IUserSession;
use RuntimeException;

final class SettingsController extends OCSController {
	private const CONFIG_STORAGE_ROOT = 'storage_root';
	private const DEFAULT_STORAGE_ROOT = 'audio';

	public function __construct(
		IRequest $request,
		private readonly IConfig $config,
		private readonly IUserSession $userSession,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[NoAdminRequired]
	public function getSettings(): DataResponse {
		$user = $this->userSession->getUser();
		if ($user === null) {
			throw new RuntimeException('No authenticated Nextcloud user is available.');
		}

		return new DataResponse([
			'storage_root' => $this->config->getUserValue($user->getUID(), Application::APP_ID, self::CONFIG_STORAGE_ROOT, ''),
			'default_storage_root' => self::DEFAULT_STORAGE_ROOT,
		]);
	}

	#[NoAdminRequired]
	public function setSettings(string $storageRoot = ''): DataResponse {
		$user = $this->userSession->getUser();
		if ($user === null) {
			throw new RuntimeException('No authenticated Nextcloud user is available.');
		}

		$storageRoot = trim($storageRoot);
		$storageRoot = preg_replace('#[\\/]+#', '/', trim($storageRoot, "\\/")) ?? '';
		if ($storageRoot === '.') {
			$storageRoot = '';
		}
		if ($storageRoot === '..' || str_starts_with($storageRoot, '../') || str_contains($storageRoot, '/../')) {
			throw new RuntimeException('Storage root may not escape the user Files root.');
		}

		foreach (preg_split('#/#', $storageRoot, -1, PREG_SPLIT_NO_EMPTY) ?: [] as $segment) {
			if ($segment === '.' || $segment === '..' || str_contains($segment, "\0")) {
				throw new RuntimeException('Storage root contains an invalid path component.');
			}
		}

		$this->config->setUserValue($user->getUID(), Application::APP_ID, self::CONFIG_STORAGE_ROOT, $storageRoot);

		return new DataResponse([
			'storage_root' => $storageRoot,
			'default_storage_root' => self::DEFAULT_STORAGE_ROOT,
		]);
	}
}
