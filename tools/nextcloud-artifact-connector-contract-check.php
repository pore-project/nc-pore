<?php

declare(strict_types=1);

namespace OCA\PoRe\AppInfo {
	final class Application { public const APP_ID = 'pore'; }
}
namespace OCA\PoRe\Service {
	class RecordingRuntimeService {
		public static string $convertedPayload = '';
		public function command(array $command, string $operation): array {
			if ($operation !== 'artifact.convert_flac_to_wav') throw new \RuntimeException('Unexpected runtime operation.');
			if (!isset($command['output_path'])) throw new \RuntimeException('Missing output path.');
			if (file_put_contents($command['output_path'], self::$convertedPayload) === false) throw new \RuntimeException('Unable to write fake runtime output.');
			return [
				'status' => 'converted',
				'sample_rate_hz' => 48000,
				'channels' => 1,
				'bits_per_sample' => 24,
				'sample_count' => intdiv(max(0, strlen(self::$convertedPayload) - 44), 3),
				'payload_length' => strlen(self::$convertedPayload),
			];
		}
	}
}
namespace OCA\PoRe\Controller {
	final class ProductionController {
		public static function ownerKey(string $productionId): string { return 'production_owner_' . $productionId; }
	}
}
namespace OCP {
	class Constants { public const PERMISSION_CREATE = 4; }
	class IConfig {}
}
namespace OCP\Files {
	class NotFoundException extends \RuntimeException {}

	class File {
		private ?Folder $parent = null;

		public function __construct(private readonly int $id, private readonly string $content, private string $name = 'Host.wav') {}

		public function getId(): int { return $this->id; }
		public function getSize(): int { return strlen($this->content); }
		public function getName(): string { return $this->name; }

		public function delete(): void {
			$this->getParent()->remove($this->name);
		}

		public function setParent(?Folder $parent, string $name): void {
			$this->parent = $parent;
			$this->name = $name;
		}

		public function getParent(): Folder {
			if ($this->parent === null) throw new \RuntimeException('Fake file has no parent.');
			return $this->parent;
		}

		public function fopen(string $mode) {
			$stream = fopen('php://temp', 'w+b');
			fwrite($stream, $this->content);
			rewind($stream);
			return $stream;
		}

		public function getPath(): string {
			return rtrim($this->getParent()->getPath(), '/') . '/' . $this->name;
		}
	}

	class Folder {
		private array $children = [];
		private array $filesById = [];
		public int $fileIdLookupCalls = 0;

		public function __construct(private string $path) {}

		public function get(string $name) {
			if (!array_key_exists($name, $this->children)) throw new NotFoundException($name);
			return $this->children[$name];
		}

		public function newFolder(string $name): Folder {
			$folder = new Folder(rtrim($this->path, '/') . '/' . $name);
			$this->children[$name] = $folder;
			return $folder;
		}

		public function newFile(string $name): File {
			$file = new WritableFile(1000 + count($this->children), $name);
			$this->add($name, $file);
			return $file;
		}

		public function add(string $name, object $node): void {
			$this->children[$name] = $node;
			if ($node instanceof File) {
				$node->setParent($this, $name);
				$this->filesById[$node->getId()] = $node;
			}
		}

		public function remove(string $name): void {
			if (!array_key_exists($name, $this->children)) throw new NotFoundException($name);
			$node = $this->children[$name];
			unset($this->children[$name]);
			if ($node instanceof File && ($this->filesById[$node->getId()] ?? null) === $node) {
				unset($this->filesById[$node->getId()]);
			}
		}

		public function getFirstNodeById(int $id): ?object {
			$this->fileIdLookupCalls += 1;
			if (isset($this->filesById[$id])) return $this->filesById[$id];
			foreach ($this->children as $child) {
				if ($child instanceof Folder) {
					$found = $child->getFirstNodeById($id);
					if ($found !== null) return $found;
				}
			}
			return null;
		}

		public function getPath(): string { return $this->path; }
	}

	class WritableFile extends File {
		private string $path;

		public function __construct(int $id, string $name) {
			parent::__construct($id, '', $name);
			$this->path = tempnam(sys_get_temp_dir(), 'pore-contract-file-');
		}

		public function getSize(): int {
			clearstatcache(true, $this->path);
			return (int)(filesize($this->path) ?: 0);
		}

		public function fopen(string $mode) {
			$normalized = str_starts_with($mode, 'w') ? 'w+b' : 'rb';
			$stream = fopen($this->path, $normalized);
			if ($stream === false) throw new RuntimeException('Unable to open fake writable file.');
			return $stream;
		}

		public function __destruct() {
			@unlink($this->path);
		}
	}

	class IRootFolder {}
}
namespace OCP\Security {
	class ISecureRandom {
		public const CHAR_ALPHANUMERIC = 'alnum';
	}
}
namespace OCP\Share {
	class IShare { public const TYPE_LINK = 3; }
	class IManager {}
}
namespace OCP\Share\Exceptions {
	class ShareNotFound extends \RuntimeException {}
}
namespace {
	require_once __DIR__ . '/../lib/Service/NextcloudArtifactPath.php';
	require_once __DIR__ . '/../lib/Service/NextcloudArtifactConnector.php';

	use OCA\PoRe\Service\NextcloudArtifactConnector;
	use OCP\Files\File;
	use OCP\Files\Folder;
	use OCP\Files\IRootFolder;
	use OCP\IConfig;
	use OCP\Security\ISecureRandom;
	use OCP\Share\IManager;

	function check(bool $condition, string $message): void {
		if (!$condition) throw new \RuntimeException($message);
	}

	final class FakeConfig extends IConfig {
		public function getAppValue(string $app, string $key, string $default = ''): string {
			return $key === 'production_owner_prod-1' ? 'owner' : $default;
		}
		public function getUserValue(string $userId, string $app, string $key, string $default = ''): string {
			return $default;
		}
		public function getSystemValueString(string $key): string { return 'test-secret'; }
	}

	final class FakeRootFolder extends IRootFolder {
		private Folder $userFolder;

		public function __construct() {
			$this->userFolder = new Folder('/files/owner');
		}

		public function getUserFolder(string $userId): Folder { return $this->userFolder; }

		public function targetFolder(): Folder {
			$a = $this->ensure($this->userFolder, 'audio');
			$y = $this->ensure($a, '2026');
			$m = $this->ensure($y, '09');
			return $this->ensure($m, '05 - 15:42 Interview - prod-1');
		}

		public function movedFolder(): Folder {
			$a = $this->ensure($this->userFolder, 'audio');
			$y = $this->ensure($a, '2026');
			return $this->ensure($y, 'archive');
		}

		private function ensure(Folder $parent, string $name): Folder {
			try {
				$x = $parent->get($name);
				if (!$x instanceof Folder) throw new \RuntimeException('Expected folder');
				return $x;
			} catch (\OCP\Files\NotFoundException) {
				return $parent->newFolder($name);
			}
		}
	}

	final class FakeShare {
		private int $id = 0;
		public function setNode(object $node): void {}
		public function setShareType(int $type): void {}
		public function setPermissions(int $permissions): void {}
		public function setPassword(string $password): void {}
		public function setExpirationDate(object $date): void {}
		public function setLabel(string $label): void {}
		public function setSharedBy(string $userId): void {}
		public function setShareOwner(string $userId): void {}
		public function getId(): int { return $this->id; }
		public function getToken(): string { return 'share-token'; }
		public function assignId(int $id): void { $this->id = $id; }
	}

	final class FakeShareManager extends IManager {
		public int $created = 0;
		public int $deleted = 0;
		public function shareApiAllowLinks(): bool { return true; }
		public function shareApiLinkAllowPublicUpload(): bool { return true; }
		public function newShare(): FakeShare { return new FakeShare(); }
		public function createShare(FakeShare $share): FakeShare {
			$this->created++;
			$share->assignId($this->created);
			return $share;
		}
		public function getShareById(string $id): FakeShare { throw new \OCP\Share\Exceptions\ShareNotFound(); }
		public function deleteShare(FakeShare $share): void { $this->deleted++; }
	}

	final class FakeRandom extends ISecureRandom {
		private int $counter = 0;
		public function generate(int $length, string $characterSet): string { return 'token-' . (++$this->counter); }
	}

	function connector(FakeRootFolder $root, FakeShareManager $shares): NextcloudArtifactConnector {
		return new NextcloudArtifactConnector($root, new FakeConfig(), $shares, new FakeRandom(), new \OCA\PoRe\Service\RecordingRuntimeService());
	}

	function wav(string $pcm, int $sampleRate = 48000, int $channels = 1, int $bits = 24): string {
		$bytesPerSample = intdiv($bits + 7, 8);
		$header = pack(
			'a4Va4a4VvvVVvva4V',
			'RIFF',
			36 + strlen($pcm),
			'WAVE',
			'fmt ',
			16,
			1,
			$channels,
			$sampleRate,
			$sampleRate * $channels * $bytesPerSample,
			$channels * $bytesPerSample,
			$bits,
			'data',
			strlen($pcm),
		);
		return $header . $pcm;
	}

	$root = new FakeRootFolder();
	$leaf = $root->targetFolder();

	$wav = wav("\x00\x00\x00");
	$host = new File(17, $wav, 'Host.wav');
	$leaf->add('Host.wav', $host);

	$shares = new FakeShareManager();
	$c = connector($root, $shares);

	$prepared = $c->prepare('prod-1', 'Interview', 'recording-1', 'capture-1', '2026-09-05T15:42:31+02:00', 'Host', strlen($wav), hash('sha256', $wav), 'actor-1');
	check($prepared['filename'] === 'Host.wav', 'Identical artifact must retain the original filename.');
	check($prepared['upload_required'] === false, 'Identical artifact must not require an upload.');
	check($shares->created === 0, 'Identical artifact must not create a temporary upload share.');
	try {
		$c->verify($prepared['transfer_id'], 'actor-2');
		throw new \RuntimeException('Transport handle actor binding must reject another user.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'Transport handle is not authorized for this user.', 'Unexpected actor-binding error.');
	}
	$receipt = $c->verify($prepared['transfer_id'], 'actor-1');
	check($receipt['file_id'] === 17, 'Identical artifact verification must resolve the existing file.');
	check($receipt['sha256'] === hash('sha256', $wav), 'Identical artifact verification must preserve the exact hash.');
	check($receipt['target_user_id'] === 'owner', 'Verification must retain the storage owner.');
	check($receipt['preservation']['sampleRate'] === 48000, 'Server must verify the actual WAV sample rate.');
	check($receipt['preservation']['channels'] === 1, 'Server must verify the actual WAV channel count.');
	check($receipt['preservation']['bitsPerSample'] === 24, 'Server must verify the actual WAV bit depth.');
	check($receipt['preservation']['encoding'] === 'pcm_s24le', 'Server must derive the PCM encoding from the WAV container.');
	$c->close($prepared['transfer_id'], 'actor-1');

	// TEST-01: A previously verified file may be renamed/moved and should still
	// be reusable by its stable File-ID without creating a new upload.
	$leaf->remove('Host.wav');
	$moved = $root->movedFolder();
	$moved->add('Host-renamed.wav', $host);
	$prepared = $c->prepare('prod-1', 'Interview', 'recording-1', 'capture-1', '2026-09-05T15:42:31+02:00', 'Host', strlen($wav), hash('sha256', $wav), 'actor-1', 17);
	check($prepared['filename'] === 'Host-renamed.wav', 'Preferred File-ID reuse must follow a renamed/moved file.');
	check($prepared['upload_required'] === false, 'A moved identical artifact must not require a new upload.');
	$receipt = $c->verify($prepared['transfer_id'], 'actor-1');
	check($receipt['file_id'] === 17, 'Preferred File-ID verification must resolve the original file after relocation.');
	check($receipt['filename'] === 'Host-renamed.wav', 'Verification must report the current file name.');

	// TEST-02: A file referenced by an existing verified record must not be
	// silently reused after its payload was changed.
	$changed = wav("\x09\x09\x09");
	$changedRoot = new FakeRootFolder();
	$changedUserFolder = $changedRoot->getUserFolder('owner');
	$changedUserFolder->fileIdLookupCalls = 0;
	$changedUserFolder->add('Host.wav', new File(17, $changed, 'Host.wav'));
	check($changedUserFolder->getFirstNodeById(17) instanceof File, 'Preferred File-ID fixture must be visible from the user folder.');
	$changedUserFolder->fileIdLookupCalls = 0;
	check(hash('sha256', $changed) !== hash('sha256', $wav), 'Preferred File-ID fixture payload must differ from the expected payload.');
	$changedShares = new FakeShareManager();
	$changedConnector = connector($changedRoot, $changedShares);
	$rejected = false;
	try {
		$changedConnector->prepare('prod-1', 'Interview', 'recording-1', 'capture-1', '2026-09-05T15:42:31+02:00', 'Host', strlen($wav), hash('sha256', $wav), 'actor-1', 17);
	} catch (\RuntimeException $error) {
		$rejected = true;
		check($error->getMessage() === 'Nextcloud recorded artifact payload has changed.', 'Unexpected preferred File-ID conflict error: ' . $error->getMessage());
	}
	check($rejected, 'Preferred File-ID payload change must be rejected. lookupCalls=' . $changedUserFolder->fileIdLookupCalls);

	// TEST-02b: A missing preferred File-ID must never fall back to an unrelated
	// same-name/same-payload file.
	$moved->remove('Host-renamed.wav');
	$leaf->add('Host.wav', new File(22, $wav, 'Host.wav'));
	$sharesBeforePreferredMissing = $shares->created;
	$prepared = $c->prepare('prod-1', 'Interview', 'recording-1', 'capture-1', '2026-09-05T15:42:31+02:00', 'Host', strlen($wav), hash('sha256', $wav), 'actor-1', 17);
	check($prepared['upload_required'] === true, 'A missing preferred File-ID must force a fresh upload instead of reusing a different file.');
	check($prepared['filename'] === 'Host (2).wav', 'A stale preferred File-ID must use a fresh collision-free filename.');
	check($shares->created === $sharesBeforePreferredMissing + 1, 'A stale preferred File-ID must create exactly one replacement upload authorization.');
	$c->close($prepared['transfer_id'], 'actor-1');

	$leaf->add('Host-renamed.wav', new File(19, wav("\x01\x02\x03", 44100), 'Host-renamed.wav'));
	$shares = new FakeShareManager();
	$c = connector($root, $shares);

	// TEST-03: The explicitly supported 44.1 kHz fallback remains valid.
	$wav441 = wav("\x01\x02\x03", 44100);
	$prepared = $c->prepare('prod-1', 'Interview', 'recording-3', 'capture-3', '2026-09-05T15:42:31+02:00', 'Host-renamed', strlen($wav441), hash('sha256', $wav441), 'actor-1');
	$receipt = $c->verify($prepared['transfer_id'], 'actor-1');
	check($receipt['preservation']['sampleRate'] === 44100, '44.1 kHz preservation must remain explicitly represented.');

	$leaf->add('Invalid.wav', new File(20, wav("\x00\x01\x02", 32000), 'Invalid.wav'));
	try {
		$prepared = $c->prepare('prod-1', 'Interview', 'recording-4', 'capture-4', '2026-09-05T15:42:31+02:00', 'Invalid', strlen(wav("\x00\x01\x02", 32000)), hash('sha256', wav("\x00\x01\x02", 32000)), 'actor-1');
		$c->verify($prepared['transfer_id'], 'actor-1');
		throw new \RuntimeException('Unsupported sample rate must be rejected.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'Nextcloud transport artifact is not a supported PoRE PCM WAV.', 'Unsupported WAV sample rate must be rejected server-side.');
	}

	$leaf->add('Host.wav', new File(21, 'occupied', 'Host.wav'));

	// TEST-04: FLAC is transport-only; the uploaded FLAC is verified by its
	// transport hash and then converted into the canonical WAV Artifact.
	$canonicalWav = wav("\x00\x00\x00");
\OCA\PoRe\Service\RecordingRuntimeService::$convertedPayload = $canonicalWav;
	$flacPayload = base64_decode('ZkxhQwAAACIQABAAAAATAAATC7gBcAAAAAMhBpBwZIs2WggRQqoY7t/kAwAAEgAAAAAAAAAAAAAAAAAAAAAAA4QAACggAAAAcmVmZXJlbmNlIGxpYkZMQUMgMS41LjAgMjAyNTAyMTEAAAAA//hqDAACggIAAAB///+AAAC3fA==', true);
	check($flacPayload !== false, 'FLAC fixture must decode from base64.');
	$prepared = $c->prepare(
		'prod-1',
		'Interview',
		'recording-5',
		'capture-5',
		'2026-09-05T15:42:31+00:00',
		'Host',
		strlen($flacPayload),
		hash('sha256', $flacPayload),
		'actor-1',
		null,
		'audio/flac',
	);
	check($prepared['payload_format'] === 'audio/flac', 'FLAC prepare must expose the transport payload format.');
	check($prepared['filename'] === 'Host (2).flac', 'FLAC transport must use a distinct .flac filename.');
	check($prepared['canonical_filename'] === 'Host (2).wav', 'FLAC transport must retain the canonical .wav filename.');
	check($prepared['upload_required'] === true, 'Fresh FLAC transport must require an upload.');

	$uploadedFlac = new File(30, $flacPayload, 'Host (2).flac');
	$leaf->add('Host (2).flac', $uploadedFlac);
	$receipt = $c->verify($prepared['transfer_id'], 'actor-1');
	check($receipt['filename'] === 'Host (2).wav', 'FLAC verification must return the canonical WAV artifact.');
	check($receipt['size'] === strlen($canonicalWav), 'Canonical WAV size must come from the converted artifact.');
	check($receipt['sha256'] === hash('sha256', $canonicalWav), 'Canonical WAV hash must be distinct from the FLAC transport hash.');
	check($receipt['preservation']['encoding'] === 'pcm_s24le', 'Converted canonical artifact must satisfy the V1 PCM preservation contract.');
	$canonicalId = $receipt['file_id'];
	$c->close($prepared['transfer_id'], 'actor-1');

	// TEST-FLAC-02: A canonical collision must reject the transport and remove
	// the temporary uploaded FLAC rather than leaving an orphaned remote file.
	$conflictPrepared = $c->prepare(
		'prod-1',
		'Interview',
		'recording-6',
		'capture-6',
		'2026-09-05T15:42:31+00:00',
		'Guest',
		strlen($flacPayload),
		hash('sha256', $flacPayload),
		'actor-1',
		null,
		'audio/flac',
	);
	$leaf->add('Guest.flac', new File(31, $flacPayload, 'Guest.flac'));
	$leaf->add('Guest.wav', new File(32, wav("\x09\x09\x09"), 'Guest.wav'));
	try {
		$c->verify($conflictPrepared['transfer_id'], 'actor-1');
		throw new \RuntimeException('Canonical collision must be rejected.');
	} catch (\RuntimeException $error) {
		check($error->getMessage() === 'artifact_manifest_conflict', 'Unexpected canonical collision error: ' . $error->getMessage());
	}
	try {
		$leaf->get('Guest.flac');
		throw new \RuntimeException('Failed FLAC canonicalization must not leave the uploaded transport file behind.');
	} catch (\OCP\Files\NotFoundException) {
	}

	$prepared = $c->prepare(
		'prod-1',
		'Interview',
		'recording-5',
		'capture-5',
		'2026-09-05T15:42:31+00:00',
		'Host',
		strlen($flacPayload) + 7,
		hash('sha256', $flacPayload . 'changed'),
		'actor-1',
		$canonicalId,
		'audio/flac',
	);
	check($prepared['upload_required'] === false, 'An existing canonical WAV must be reusable without re-upload for FLAC transport.');
	check($prepared['filename'] === 'Host (2).wav', 'Canonical File-ID reuse must retain the current WAV filename.');
	check($prepared['canonical_filename'] === 'Host (2).wav', 'Existing canonical reuse must expose the WAV canonical name.');
	check($prepared['payload_format'] === 'audio/flac', 'Canonical reuse must retain the requested FLAC transport format.');

	$sharesCreatedBeforeCollision = $shares->created;
	$leaf->add('Host (2).wav', new File(18, 'occupied', 'Host (2).wav'));
	$payload = wav("\x01\x02\x03");
	$prepared = $c->prepare('prod-1', 'Interview', 'recording-2', 'capture-2', '2026-09-05T15:42:31+02:00', 'Host', strlen($payload), hash('sha256', $payload), 'actor-1');
	check($prepared['filename'] === 'Host (3).wav', 'Differing content must select the first free numeric suffix.');
	check($prepared['upload_required'] === true, 'Differing content must require an upload.');
	check($shares->created === $sharesCreatedBeforeCollision + 1, 'Differing content must create exactly one additional upload share.');

	echo "Nextcloud artifact collision contract checks passed.\n";
}
