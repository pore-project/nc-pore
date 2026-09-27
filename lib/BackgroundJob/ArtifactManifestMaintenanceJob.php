<?php

declare(strict_types=1);

namespace OCA\PoRe\BackgroundJob;

use OCA\PoRe\Service\ArtifactManifestStore;
use OCP\AppFramework\Utility\ITimeFactory;
use OCP\BackgroundJob\TimedJob;
use OCP\Files\File;
use OCP\Files\Folder;
use OCP\Files\IRootFolder;
use OCP\Files\NotFoundException;
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
		$this->setAllowParallelRuns(false);
	}

	#[\Override]
	protected function run(mixed $argument): void {
		$this->store->cleanupExpiredPending();

		foreach ($this->store->list() as $record) {
			if (($record['status'] ?? null) !== 'verified') continue;

			$artifactId = $record['artifact_id'] ?? null;
			$remote = is_array($record['remote'] ?? null) ? $record['remote'] : [];
			$fileId = $remote['file_id'] ?? null;
			$targetUserId = $remote['target_user_id'] ?? null;
			if (!is_string($artifactId) || trim($artifactId) === ''
				|| !is_int($fileId) || $fileId <= 0
				|| !is_string($targetUserId) || trim($targetUserId) === '') {
				$this->logger->warning('Skipping invalid PoRE artifact record during maintenance.', ['app' => 'pore']);
				continue;
			}

			try {
				$userFolder = $this->rootFolder->getUserFolder($targetUserId);
				$activeNode = $userFolder->getFirstNodeById($fileId);
				if ($activeNode instanceof File && $activeNode->getId() === $fileId) {
					continue;
				}
			} catch (\Throwable $exception) {
				$this->logger->warning('Unable to reconcile PoRE artifact record with active Nextcloud Files.', [
					'app' => 'pore',
					'artifact_id' => $artifactId,
					'file_id' => $fileId,
					'target_user_id' => $targetUserId,
					'exception' => $exception,
				]);
				continue;
			}

			try {
				$trashFolder = $userFolder->getParent()->get('files_trashbin/files');
				if (!$trashFolder instanceof Folder) {
					$this->logger->warning('Unexpected PoRE trashbin node type during artifact reconciliation.', [
						'app' => 'pore',
						'artifact_id' => $artifactId,
						'file_id' => $fileId,
					]);
					continue;
				}

				$trashNode = $trashFolder->getFirstNodeById($fileId);
				if ($trashNode instanceof File && $trashNode->getId() === $fileId) {
					continue;
				}
			} catch (NotFoundException) {
				// No trashbin node exists for this user/file. The record may now
				// be removed, subject to the conditional File-ID check below.
			} catch (\Throwable $exception) {
				$this->logger->warning('Unable to determine whether a PoRE artifact is retained in Nextcloud trash.', [
					'app' => 'pore',
					'artifact_id' => $artifactId,
					'file_id' => $fileId,
					'target_user_id' => $targetUserId,
					'exception' => $exception,
				]);
				continue;
			}

			try {
				$this->store->removeIfVerifiedRemoteFileIdMatches($artifactId, $fileId);
			} catch (\Throwable $exception) {
				$this->logger->warning('Unable to remove stale PoRE artifact record during maintenance.', [
					'app' => 'pore',
					'artifact_id' => $artifactId,
					'file_id' => $fileId,
					'exception' => $exception,
				]);
			}
		}
	}
}
