<?php

declare(strict_types=1);

namespace OCA\PoRe\BackgroundJob;

use OCA\PoRe\Service\ArtifactManifestStore;
use OCP\AppFramework\Utility\ITimeFactory;
use OCP\BackgroundJob\TimedJob;
use OCP\Files\File;
use OCP\Files\IRootFolder;
use Psr\Log\LoggerInterface;

final class ArtifactManifestMaintenanceJob extends TimedJob {
	private const INTERVAL_SECONDS = 60 * 60;

	public function __construct(
		ITimeFactory $time,
		private readonly ArtifactManifestStore $store,
		private readonly IRootFolder $rootFolder,
		private readonly LoggerInterface $logger,
	) {
		parent::__construct($time);
		$this->setInterval(self::INTERVAL_SECONDS);
	}

	#[\Override]
	protected function run(mixed $argument): void {
		$this->store->cleanupExpiredPending();

		foreach ($this->store->list() as $record) {
			if (($record['status'] ?? null) !== 'verified') continue;

			$artifactId = $record['artifact_id'] ?? null;
			$fileId = $record['remote']['file_id'] ?? null;
			if (!is_string($artifactId) || $artifactId === '' || !is_int($fileId) || $fileId <= 0) {
				$this->logger->warning('Skipping invalid PoRE artifact record during maintenance.', ['app' => 'pore']);
				continue;
			}

			try {
				$nodes = $this->rootFolder->getById($fileId);
			} catch (\Throwable $exception) {
				$this->logger->warning('Unable to reconcile PoRE artifact record with Nextcloud Files.', [
					'app' => 'pore',
					'artifact_id' => $artifactId,
					'file_id' => $fileId,
					'exception' => $exception,
				]);
				continue;
			}

			$exists = false;
			foreach ($nodes as $node) {
				if ($node instanceof File && $node->getId() === $fileId) {
					$exists = true;
					break;
				}
			}

			if (!$exists) {
				$this->store->remove($artifactId);
			}
		}
	}
}
