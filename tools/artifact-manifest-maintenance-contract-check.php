<?php

declare(strict_types=1);

namespace OCP {
	class IConfig {
		public function getSystemValueString(string $key): string { return ''; }
		public function getSystemValue(string $key, mixed $default = null): mixed { return $default; }
	}

	class \OCP\AppFramework\Utility\ITimeFactory {}

	class \OCP\BackgroundJob\TimedJob {
		private int $interval = 0;
		private bool $allowParallelRuns = true;

		public function __construct(object $time) {}
		public function setInterval(int $seconds): void { $this->interval = $seconds; }
		public function setAllowParallelRuns(bool $allow): void { $this->allowParallelRuns = $allow; }
		public function getIntervalForTest(): int { return $this->interval; }
		public function getAllowParallelRunsForTest(): bool { return $this->allowParallelRuns; }
	}

	namespace Files {
		class NotFoundException extends \RuntimeException {}

		class File {
			public function __construct(private readonly int $id, private readonly string $name) {}
			public function getId(): int { return $this->id; }
			public function getName(): string { return $this->name; }
		}

		class Folder {
			private array $children = [];

			public function __construct(private readonly string $path, private readonly ?Folder $parent = null) {}

			public function get(string $name): object {
				if (!array_key_exists($name, $this->children)) throw new NotFoundException($name);
				return $this->children[$name];
			}

			public function add(string $name, object $node): void { $this->children[$name] = $node; }

			public function getParent(): Folder {
				if ($this->parent === null) throw new \RuntimeException('Fake folder has no parent.');
				return $this->parent;
			}

			public function getFirstNodeById(int $id): ?object {
				foreach ($this->children as $child) {
					if ($child instanceof File && $child->getId() === $id) return $child;
					if ($child instanceof Folder) {
						$found = $child->getFirstNodeById($id);
						if ($found !== null) return $found;
					}
				}
				return null;
			}
		}

		class IRootFolder {}
	}
}

namespace Psr\Log {
	interface LoggerInterface {
		public function warning(string|\Stringable $message, array $context = []): void;
	}
}

namespace {
	require_once __DIR__ . '/../lib/Service/ArtifactManifestStore.php';
	require_once __DIR__ . '/../lib/BackgroundJob/ArtifactManifestMaintenanceJob.php';

	use OCA\PoRe\BackgroundJob\ArtifactManifestMaintenanceJob;
	use OCA\PoRe\Service\ArtifactManifestStore;
	use OCP\Files\File;
	use OCP\Files\Folder;
	use OCP\Files\IRootFolder;
	use OCP\IConfig;
	use Psr\Log\LoggerInterface;

	final class FakeConfig extends IConfig {
		public function __construct(private readonly string $dataDirectory, private readonly string $instanceId) {}

		public function getSystemValue(string $key, mixed $default = null): mixed {
			return match ($key) {
				'datadirectory' => $this->dataDirectory,
				'instanceid' => $this->instanceId,
				default => $default,
			};
		}
	}

	final class FakeLogger implements LoggerInterface {
		public array $warnings = [];
		public function warning(string|\Stringable $message, array $context = []): void {
			$this->warnings[] = [$message, $context];
		}
	}

	final class FakeRootFolder extends IRootFolder {
		public function __construct(private readonly Folder $userFolder) {}
		public function getUserFolder(string $userId): Folder { return $this->userFolder; }
	}

	function check(bool $condition, string $message): void {
		if (!$condition) throw new \RuntimeException($message);
	}

	function runJob(ArtifactManifestMaintenanceJob $job): void {
		$method = new \ReflectionMethod($job, 'run');
		$method->setAccessible(true);
		$method->invoke($job, []);
	}

	function verifiedRecord(ArtifactManifestStore $store, string $artifactId, int $fileId): void {
		$provenance = [
			'schemaVersion' => 1,
			'capture' => [
				'startedAt' => '2026-09-27T09:00:00.000Z',
				'sampleRate' => 48000,
				'sampleSize' => 24,
				'channelCount' => 1,
				'processing' => [
					'echoCancellation' => false,
					'noiseSuppression' => false,
					'autoGainControl' => false,
				],
			],
			'sourceSegments' => [],
		];

		$store->stagePreparedArtifact([
			'artifact_id' => $artifactId,
			'production_id' => 'production-1',
			'recording_id' => 'recording-' . $artifactId,
			'recording_session_id' => 'session-' . $artifactId,
			'participant_label' => 'Host',
			'target_user_id' => 'owner',
			'filename' => $artifactId . '.wav',
			'size' => 47,
			'payload_sha256' => hash('sha256', 'payload'),
			'capture_provenance' => $provenance,
		]);

		$store->persistVerifiedArtifact([
			'artifact_id' => $artifactId,
			'target_user_id' => 'owner',
			'file_id' => $fileId,
			'path' => 'audio/' . $artifactId . '.wav',
			'filename' => $artifactId . '.wav',
			'size' => 47,
			'sha256' => hash('sha256', 'payload'),
			'preservation' => [
				'format' => 'audio/wav',
				'encoding' => 'pcm_s24le',
				'sampleRate' => 48000,
				'channels' => 1,
				'bitsPerSample' => 24,
			],
		]);
	}

	$directory = sys_get_temp_dir() . '/nc-pore-artifact-maintenance-' . bin2hex(random_bytes(6));
	$store = new ArtifactManifestStore(new FakeConfig($directory, 'instance-1'));

	$filesRoot = new Folder('/files');
	$userFolder = new Folder('/files/owner', $filesRoot);
	$trashRoot = new Folder('/files/files_trashbin', $filesRoot);
	$trashFiles = new Folder('/files/files_trashbin/files', $trashRoot);
	$filesRoot->add('files_trashbin', $trashRoot);
	$trashRoot->add('files', $trashFiles);

	$userFolder->add('active.wav', new File(42, 'active.wav'));
	$trashFiles->add('deleted.wav', new File(43, 'deleted.wav'));

	$root = new FakeRootFolder($userFolder);
	$logger = new FakeLogger();
	$job = new ArtifactManifestMaintenanceJob(
		new \OCP\AppFramework\Utility\ITimeFactory(),
		$store,
		$root,
		$logger,
	);

	check($job->getIntervalForTest() === 3600, 'Artifact maintenance interval must remain one hour.');
	check($job->getAllowParallelRunsForTest() === false, 'Artifact maintenance must not allow parallel runs.');

	// TEST-01: An active artifact remains associated with its verified record.
	verifiedRecord($store, 'active', 42);

	// TEST-02: A file retained in Nextcloud trash must not be treated as
	// permanently deleted.
	verifiedRecord($store, 'trash', 43);

	// TEST-03: A file absent from both active Files and trash is eligible for
	// conditional record removal.
	verifiedRecord($store, 'missing', 44);

	runJob($job);

	check($store->get('active') !== null, 'Active artifact record must remain.');
	check($store->get('trash') !== null, 'Trashed artifact record must remain.');
	check($store->get('missing') === null, 'Permanently absent artifact record must be removed.');

	$glob = glob($directory . '/appdata_instance-1/pore/artifacts/*.json') ?: [];
	foreach ($glob as $path) @unlink($path);
	foreach (glob($directory . '/appdata_instance-1/pore/artifacts/*.lock') ?: [] as $path) @unlink($path);
	@rmdir($directory . '/appdata_instance-1/pore/artifacts');
	@rmdir($directory . '/appdata_instance-1/pore');
	@rmdir($directory . '/appdata_instance-1');
	@rmdir($directory);

	echo "Artifact maintenance contract checks passed.\n";
}
