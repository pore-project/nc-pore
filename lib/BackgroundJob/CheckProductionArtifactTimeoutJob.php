<?php

declare(strict_types=1);

namespace OCA\PoRe\BackgroundJob;

use OCA\PoRe\Service\RecordingRuntimeService;
use OCP\AppFramework\Utility\ITimeFactory;
use OCP\BackgroundJob\QueuedJob;
use RuntimeException;

final class CheckProductionArtifactTimeoutJob extends QueuedJob {
	public function __construct(
		ITimeFactory $time,
		private readonly RecordingRuntimeService $runtime,
	) {
		parent::__construct($time);
	}

	#[\Override]
	protected function run(mixed $argument): void {
		if (!is_array($argument) || !is_string($argument['production_id'] ?? null) || trim($argument['production_id']) === '') {
			throw new RuntimeException('Production artifact timeout job requires a production_id argument.');
		}

		$this->runtime->command([
			'request_id' => bin2hex(random_bytes(16)),
			'session_id' => $argument['production_id'],
			'actor_id' => '',
			'owner_id' => '',
			'participants' => [],
			'command' => ['CheckTimeout' => null],
		], 'production.command');
	}
}
