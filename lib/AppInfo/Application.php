<?php

declare(strict_types=1);

namespace OCA\PoRe\AppInfo;

use OCP\AppFramework\App;
use OCP\AppFramework\Bootstrap\IBootContext;
use OCP\AppFramework\Bootstrap\IBootstrap;
use OCP\AppFramework\Bootstrap\IRegistrationContext;
use OCP\Util;

class Application extends App implements IBootstrap {
	public const APP_ID = 'pore';

	public function __construct(array $urlParams = []) {
		parent::__construct(self::APP_ID, $urlParams);
	}

	public function register(IRegistrationContext $context): void {
	}

	public function boot(IBootContext $context): void {
		Util::addStyle(self::APP_ID, 'pore-talk-recording');
		Util::addInitScript(self::APP_ID, 'pore-browser-pcm-persistence');
		Util::addInitScript(self::APP_ID, 'pore-browser-completion-job');
		Util::addInitScript(self::APP_ID, 'pore-recording-controller');
		Util::addInitScript(self::APP_ID, 'pore-browser-pcm-recorder');
		Util::addInitScript(self::APP_ID, 'pore-talk-audio-connector');
		Util::addInitScript(self::APP_ID, 'pore-talk-recording-state-bridge');
		Util::addInitScript(self::APP_ID, 'pore-talk-recording-ui');
		Util::addInitScript(self::APP_ID, 'init');
	}
}
