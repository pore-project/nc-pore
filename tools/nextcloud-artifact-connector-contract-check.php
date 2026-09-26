<?php

declare(strict_types=1);

namespace OCA\PoRe\AppInfo {
	final class Application { public const APP_ID = 'pore'; }
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
		public function __construct(private readonly int $id, private readonly string $content) {}
		public function getId(): int { return $this->id; }
		public function getSize(): int { return strlen($this->content); }
		public function fopen(string $mode) {
			$stream = fopen('php://temp', 'w+b');
			fwrite($stream, $this->content);
			rewind($stream);
			return $stream;
		}
		public function getPath(): string { return '/files/owner/audio/2026/09/05 - 15:42 Interview - prod-1'; }
	}
	class Folder {
		private array $children = [];
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
		public function add(string $name, object $node): void { $this->children[$name] = $node; }
		public function getPath(): string { return $this->path; }
	}
	class IRootFolder {}
}
namespace OCP\Security {
	class ISecureRandom { public const CHAR_ALPHANUMERIC = 'alnum'; }
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
		public function getAppValue(string $app,string $key,string $default=''): string { return $key==='production_owner_prod-1' ? 'owner' : $default; }
		public function getUserValue(string $userId,string $app,string $key,string $default=''): string { return $default; }
		public function getSystemValueString(string $key): string { return 'test-secret'; }
	}
	final class FakeRootFolder extends IRootFolder {
		private Folder $userFolder;
		public function __construct() { $this->userFolder=new Folder('/files/owner'); }
		public function getUserFolder(string $userId): Folder { return $this->userFolder; }
		public function targetFolder(): Folder {
			$a=$this->ensure($this->userFolder,'audio'); $y=$this->ensure($a,'2026'); $m=$this->ensure($y,'09');
			return $this->ensure($m,'05 - 15:42 Interview - prod-1');
		}
		private function ensure(Folder $p,string $n): Folder {
			try { $x=$p->get($n); if(!$x instanceof Folder) throw new \RuntimeException('Expected folder'); return $x; }
			catch (\OCP\Files\NotFoundException) { return $p->newFolder($n); }
		}
	}
	final class FakeShare {
		private int $id=0;
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
		public function assignId(int $id): void { $this->id=$id; }
	}
	final class FakeShareManager extends IManager {
		public int $created=0; public int $deleted=0;
		public function shareApiAllowLinks(): bool { return true; }
		public function shareApiLinkAllowPublicUpload(): bool { return true; }
		public function newShare(): FakeShare { return new FakeShare(); }
		public function createShare(FakeShare $share): FakeShare { $this->created++; $share->assignId($this->created); return $share; }
		public function getShareById(string $id): FakeShare { throw new \OCP\Share\Exceptions\ShareNotFound(); }
		public function deleteShare(FakeShare $share): void { $this->deleted++; }
	}
	final class FakeRandom extends ISecureRandom {
		private int $counter=0;
		public function generate(int $length,string $characterSet): string { return 'token-'.(++$this->counter); }
	}
	function connector(FakeRootFolder $root, FakeShareManager $shares): NextcloudArtifactConnector {
		return new NextcloudArtifactConnector($root,new FakeConfig(),$shares,new FakeRandom());
	}


	function wav(string $pcm): string {
		$sampleRate = 48000; $channels = 1; $bits = 24;
		$header = pack(
			'a4Va4a4VvvVVvv a4V',
			'RIFF',
			36 + strlen($pcm),
			'WAVE',
			'fmt ',
			16,
			1,
			$channels,
			$sampleRate,
			$sampleRate * $channels * 3,
			$channels * 3,
			$bits,
			'data',
			strlen($pcm),
		);
		return $header . $pcm;
	}

	$root=new FakeRootFolder(); $leaf=$root->targetFolder(); $wav=wav("\x00\x00\x00"); $leaf->add('Host.wav',new File(17,$wav));
	$shares=new FakeShareManager(); $c=connector($root,$shares);
	$prepared=$c->prepare('prod-1','Interview','recording-1','capture-1','2026-09-05T15:42:31+02:00','Host',strlen($wav),hash('sha256',$wav),'actor-1');
	check($prepared['filename']==='Host.wav','Identical artifact must retain the original filename.');
	check($prepared['upload_required']===false,'Identical artifact must not require an upload.');
	check($shares->created===0,'Identical artifact must not create a temporary upload share.');
	try { $c->verify($prepared['transfer_id'],'actor-2'); throw new \RuntimeException('Transport handle actor binding must reject another user.'); } catch (\RuntimeException $error) { check($error->getMessage()==='Transport handle is not authorized for this user.','Unexpected actor-binding error.'); }
	$receipt=$c->verify($prepared['transfer_id'],'actor-1');
	check($receipt['file_id']===17,'Identical artifact verification must resolve the existing file.');
	check($receipt['sha256']===hash('sha256',$wav),'Identical artifact verification must preserve the exact hash.');
	check($receipt['preservation']['sampleRate']===48000,'Server must verify the actual WAV sample rate.');
	check($receipt['preservation']['channels']===1,'Server must verify the actual WAV channel count.');
	check($receipt['preservation']['bitsPerSample']===24,'Server must verify the actual WAV bit depth.');
	check($receipt['preservation']['encoding']==='pcm_s24le','Server must derive the PCM encoding from the WAV container.');
	$c->close($prepared['transfer_id'],'actor-1');

	$leaf->add('Host (2).wav',new File(18,'occupied'));
	$prepared=$c->prepare('prod-1','Interview','recording-2','capture-2','2026-09-05T15:42:31+02:00','Host',strlen(wav("\x01\x02\x03")),hash('sha256',wav("\x01\x02\x03")),'actor-1');
	check($prepared['filename']==='Host (3).wav','Differing content must select the first free numeric suffix.');
	check($prepared['upload_required']===true,'Differing content must require an upload.');
	check($shares->created===1,'Differing content must create exactly one upload share.');

	echo "Nextcloud artifact collision contract checks passed.\n";
}
