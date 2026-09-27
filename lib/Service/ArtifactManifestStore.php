<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use OCP\IConfig;
use RuntimeException;

final class ArtifactManifestStore {
	private const SCHEMA_VERSION = 1;
	private const PENDING_RETENTION_SECONDS = 2 * 60 * 60;
	private const DIRECTORY_MODE = 0700;
	private const FILE_MODE = 0600;

	public function __construct(
		private readonly IConfig $config,
	) {
	}

	/**
	 * Store the client-reported metadata at the transport preparation boundary.
	 *
	 * The data is explicitly marked as pending verification. Nothing in this
	 * record is treated as proof about the actual remote file until verify()
	 * has succeeded against the remote representation.
	 *
	 * @param array<string, mixed> $submission
	 */
	public function stagePreparedArtifact(array $submission): array {
		$record = $this->normalizePendingRecord($submission);

		return $this->withArtifactLock($record['artifact_id'], LOCK_EX, function (string $path) use ($record): array {
			$existing = $this->readPath($path);

			if ($existing !== null) {
				$this->assertStableIdentityMatches($existing, $record);
				$this->assertExpectedPayloadMatches($existing, $record);

				if (($existing['status'] ?? null) === 'verified') {
					// A previously verified immutable Artifact record wins over a
					// repeated preparation. Do not downgrade or mutate it.
					return $existing;
				}
			}

			$record['prepared_at'] = gmdate('c');
			$this->writeLocked($path, $record);
			return $record;
		});
	}

	/**
	 * Persist a server-canonical verified artifact record.
	 *
	 * @param array<string, mixed> $receipt
	 */
	public function persistVerifiedArtifact(array $receipt): array {
		$artifactId = $this->requiredString($receipt['artifact_id'] ?? null, 'artifact_id');

		return $this->withArtifactLock($artifactId, LOCK_EX, function (string $path) use ($receipt, $artifactId): array {
			$existing = $this->readPath($path);

			if ($existing !== null && ($existing['status'] ?? null) === 'verified') {
				$remote = is_array($existing['remote'] ?? null) ? $existing['remote'] : [];
				$receiptSize = $this->requiredNonNegativeInt($receipt['size'] ?? null, 'size');
				$receiptHash = $this->requiredSha256($receipt['sha256'] ?? null);

				if (($remote['size'] ?? null) !== $receiptSize
					|| strtolower((string)($remote['sha256'] ?? '')) !== $receiptHash) {
					throw new RuntimeException('artifact_manifest_conflict');
				}

				$preservation = $this->normalizePreservation($receipt['preservation'] ?? null);
				if ($this->canonicalJson($existing['preservation'] ?? null) !== $this->canonicalJson($preservation)) {
					throw new RuntimeException('artifact_manifest_conflict');
				}

				$updated = $existing;
				$updated['remote'] = [
					'target_user_id' => $this->nullableString($receipt['target_user_id'] ?? ($remote['target_user_id'] ?? null)),
					'filename' => $this->nullableString($receipt['filename'] ?? ($remote['filename'] ?? null)),
					'file_id' => $this->requiredPositiveInt($receipt['file_id'] ?? ($remote['file_id'] ?? null), 'file_id'),
					'path' => $this->nullableString($receipt['path'] ?? ($remote['path'] ?? null)),
					'size' => $receiptSize,
					'sha256' => $receiptHash,
				];

				if ($this->canonicalJson($existing) !== $this->canonicalJson($updated)) {
					$this->writeLocked($path, $updated);
				}
				return $updated;
			}

			if ($existing === null) {
				throw new RuntimeException('artifact_manifest_context_missing');
			}

			$pending = $existing;
			$fileId = $this->requiredPositiveInt($receipt['file_id'] ?? null, 'file_id');
			$size = $this->requiredNonNegativeInt($receipt['size'] ?? null, 'size');
			$sha256 = $this->requiredSha256($receipt['sha256'] ?? null);

			$record = [
				'schema_version' => self::SCHEMA_VERSION,
				'status' => 'verified',
				'production_id' => $pending['production_id'] ?? null,
				'production_label' => $pending['production_label'] ?? null,
				'recording_id' => $pending['recording_id'] ?? null,
				'recording_session_id' => $pending['recording_session_id'] ?? null,
				'artifact_id' => $artifactId,
				'participant_label' => $pending['participant_label'] ?? null,
				'remote' => [
					'target_user_id' => $this->nullableString($receipt['target_user_id'] ?? ($pending['remote']['target_user_id'] ?? null)),
					'filename' => $this->nullableString($receipt['filename'] ?? ($pending['remote']['filename'] ?? null)),
					'file_id' => $fileId,
					'path' => $this->nullableString($receipt['path'] ?? null),
					'size' => $size,
					'sha256' => $sha256,
				],
				'preservation' => $this->normalizePreservation($receipt['preservation'] ?? null),
				'capture' => $pending['capture'] ?? null,
				'sourceSegments' => $pending['sourceSegments'] ?? [],
				'verified_at' => gmdate('c'),
			];

			$this->assertExpectedPayloadMatches($existing, $record, true);

			$record['manifest_hash'] = $this->manifestHash($record);
			$this->writeLocked($path, $record);
			return $record;
		});
	}

	/**
	 * @return array<string, mixed>|null
	 */
	public function get(string $artifactId): ?array {
		return $this->withArtifactLock($artifactId, LOCK_SH, function (string $path): ?array {
			return $this->readPath($path);
		});
	}

	/**
	 * @return list<array<string, mixed>>
	 */
	public function list(): array {
		$records = [];
		foreach (glob($this->directory() . '/*.json') ?: [] as $path) {
			$record = $this->withPathLock($path, LOCK_SH, function (string $lockedPath): ?array {
				if (!is_file($lockedPath)) {
					return null;
				}
				return $this->readPath($lockedPath);
			});
			if ($record !== null) {
				$records[] = $record;
			}
		}
		return $records;
	}

	public function remove(string $artifactId): void {
		$this->withArtifactLock($artifactId, LOCK_EX, function (string $path): void {
			if (is_file($path) && !unlink($path)) {
				throw new RuntimeException('artifact_manifest_storage_unavailable');
			}
		});
	}

	public function removeIfVerifiedRemoteFileIdMatches(string $artifactId, int $expectedFileId): bool {
		return $this->withArtifactLock($artifactId, LOCK_EX, function (string $path) use ($expectedFileId): bool {
			$record = $this->readPath($path);
			if ($record === null || ($record['status'] ?? null) !== 'verified') {
				return false;
			}

			$remote = is_array($record['remote'] ?? null) ? $record['remote'] : [];
			if (($remote['file_id'] ?? null) !== $expectedFileId) {
				return false;
			}

			if (!unlink($path)) {
				throw new RuntimeException('artifact_manifest_storage_unavailable');
			}
			return true;
		});
	}

	private function removeIfPendingExpired(string $artifactId, int $threshold): bool {
		return $this->withArtifactLock($artifactId, LOCK_EX, function (string $path) use ($threshold): bool {
			$record = $this->readPath($path);
			if ($record === null || ($record['status'] ?? null) !== 'pending_verification') {
				return false;
			}

			$preparedAt = $record['prepared_at'] ?? null;
			if (!is_string($preparedAt) || trim($preparedAt) === '') {
				return false;
			}

			try {
				$timestamp = new \DateTimeImmutable($preparedAt);
			} catch (\Exception) {
				return false;
			}

			if ($timestamp->getTimestamp() > $threshold) {
				return false;
			}

			if (!unlink($path)) {
				throw new RuntimeException('artifact_manifest_storage_unavailable');
			}
			return true;
		});
	}

	public function cleanupExpiredPending(?\DateTimeImmutable $now = null): int {
		$now ??= new \DateTimeImmutable('now', new \DateTimeZone('UTC'));
		$threshold = $now->getTimestamp() - self::PENDING_RETENTION_SECONDS;
		$removed = 0;

		foreach ($this->list() as $record) {
			if (($record['status'] ?? null) !== 'pending_verification') {
				continue;
			}
			$artifactId = $record['artifact_id'] ?? null;
			if (!is_string($artifactId) || trim($artifactId) === '') {
				continue;
			}
			if ($this->removeIfPendingExpired($artifactId, $threshold)) {
				$removed++;
			}
		}

		return $removed;
	}

	/**
	 * @param array<string, mixed> $submission
	 * @return array<string, mixed>
	 */
	private function normalizePendingRecord(array $submission): array {
		$artifactId = $this->requiredString($submission['artifact_id'] ?? null, 'artifact_id');
		$productionId = $this->requiredString($submission['production_id'] ?? null, 'production_id');
		$recordingId = $this->requiredString($submission['recording_id'] ?? null, 'recording_id');
		$recordingSessionId = $this->requiredString($submission['recording_session_id'] ?? null, 'recording_session_id');

		$provenance = $this->normalizeCaptureProvenance($submission['capture_provenance'] ?? null);
		$size = $this->requiredNonNegativeInt($submission['size'] ?? null, 'size');

		return [
			'schema_version' => self::SCHEMA_VERSION,
			'status' => 'pending_verification',
			'production_id' => $productionId,
			'production_label' => $this->nullableString($submission['production_label'] ?? null),
			'recording_id' => $recordingId,
			'recording_session_id' => $recordingSessionId,
			'artifact_id' => $artifactId,
			'participant_label' => $this->nullableString($submission['participant_label'] ?? null),
			'remote' => [
				'target_user_id' => $this->nullableString($submission['target_user_id'] ?? null),
				'filename' => $this->nullableString($submission['filename'] ?? null),
				'file_id' => null,
				'path' => null,
				'size' => $size,
				'sha256' => $this->requiredSha256($submission['payload_sha256'] ?? null),
			],
			'capture' => $provenance['capture'] ?? null,
			'sourceSegments' => $provenance['sourceSegments'] ?? [],
		];
	}

	/**
	 * @param mixed $value
	 * @return array<string, mixed>|null
	 */
	private function normalizeCaptureProvenance(mixed $value): ?array {
		if ($value === null) return null;
		if (!is_array($value)) throw new RuntimeException('artifact_provenance_invalid');

		$capture = $value['capture'] ?? null;
		if ($capture !== null && !is_array($capture)) throw new RuntimeException('artifact_provenance_invalid');

		$segments = $value['sourceSegments'] ?? [];
		if (!is_array($segments)) throw new RuntimeException('artifact_provenance_invalid');

		$normalizedSegments = [];
		foreach ($segments as $segment) {
			if (!is_array($segment)) throw new RuntimeException('artifact_provenance_invalid');
			$normalizedSegments[] = $this->normalizeCaptureSegment($segment);
		}

		return [
			'schemaVersion' => $this->nullableInt($value['schemaVersion'] ?? self::SCHEMA_VERSION),
			'capture' => $capture === null ? null : $this->normalizeCaptureSegment($capture),
			'sourceSegments' => $normalizedSegments,
		];
	}

	/**
	 * @param array<string, mixed> $segment
	 * @return array<string, mixed>
	 */
	private function normalizeCaptureSegment(array $segment): array {
		$processing = $segment['processing'] ?? [];
		if (!is_array($processing)) throw new RuntimeException('artifact_provenance_invalid');

		return [
			'startedAt' => $this->nullableString($segment['startedAt'] ?? null),
			'sampleRate' => $this->nullableInt($segment['sampleRate'] ?? null),
			'sampleSize' => $this->nullableInt($segment['sampleSize'] ?? null),
			'channelCount' => $this->nullableInt($segment['channelCount'] ?? null),
			'processing' => [
				'echoCancellation' => $this->nullableBool($processing['echoCancellation'] ?? null),
				'noiseSuppression' => $this->nullableBool($processing['noiseSuppression'] ?? null),
				'autoGainControl' => $this->nullableBool($processing['autoGainControl'] ?? null),
			],
		];
	}

	/**
	 * @param mixed $value
	 * @return array<string, mixed>
	 */
	private function normalizePreservation(mixed $value): array {
		if (!is_array($value)) throw new RuntimeException('artifact_preservation_invalid');

		return [
			'format' => $this->requiredString($value['format'] ?? null, 'preservation.format'),
			'encoding' => $this->requiredString($value['encoding'] ?? null, 'preservation.encoding'),
			'sampleRate' => $this->requiredPositiveInt($value['sampleRate'] ?? null, 'preservation.sampleRate'),
			'channels' => $this->requiredPositiveInt($value['channels'] ?? null, 'preservation.channels'),
			'bitsPerSample' => $this->requiredPositiveInt($value['bitsPerSample'] ?? null, 'preservation.bitsPerSample'),
		];
	}

	private function assertStableIdentityMatches(?array $existing, array $candidate): void {
		if ($existing === null) return;

		foreach (['production_id', 'recording_id', 'recording_session_id', 'participant_label'] as $field) {
			$existingValue = $existing[$field] ?? null;
			$candidateValue = $candidate[$field] ?? null;
			if ($existingValue !== null && $candidateValue !== null && $existingValue !== $candidateValue) {
				throw new RuntimeException('artifact_manifest_conflict');
			}
		}

		$existingRemote = is_array($existing['remote'] ?? null) ? $existing['remote'] : [];
		$candidateRemote = is_array($candidate['remote'] ?? null) ? $candidate['remote'] : [];
		$existingTargetUserId = $existingRemote['target_user_id'] ?? null;
		$candidateTargetUserId = $candidateRemote['target_user_id'] ?? null;
		if ($existingTargetUserId !== null && $candidateTargetUserId !== null && $existingTargetUserId !== $candidateTargetUserId) {
			throw new RuntimeException('artifact_manifest_conflict');
		}

		if (($existing['capture'] ?? null) !== null && ($candidate['capture'] ?? null) !== null
			&& $this->canonicalJson($existing['capture']) !== $this->canonicalJson($candidate['capture'])) {
			throw new RuntimeException('artifact_manifest_conflict');
		}

		if ($this->canonicalJson($existing['sourceSegments'] ?? []) !== $this->canonicalJson($candidate['sourceSegments'] ?? [])) {
			throw new RuntimeException('artifact_manifest_conflict');
		}
	}

	/**
	 * @param array<string, mixed>|null $existing
	 * @param array<string, mixed> $candidate
	 */
	private function assertExpectedPayloadMatches(?array $existing, array $candidate, bool $final = false): void {
		if ($existing === null) return;

		$existingRemote = is_array($existing['remote'] ?? null) ? $existing['remote'] : [];
		$candidateRemote = is_array($candidate['remote'] ?? null) ? $candidate['remote'] : [];

		$existingHash = $existingRemote['sha256'] ?? null;
		$candidateHash = $candidateRemote['sha256'] ?? null;
		$existingSize = $existingRemote['size'] ?? null;
		$candidateSize = $candidateRemote['size'] ?? null;

		if ($existingHash !== null && $candidateHash !== null && strtolower((string)$existingHash) !== strtolower((string)$candidateHash)) {
			throw new RuntimeException('artifact_manifest_conflict');
		}
		if ($existingSize !== null && $candidateSize !== null && (int)$existingSize !== (int)$candidateSize) {
			throw new RuntimeException('artifact_manifest_conflict');
		}

		if ($final && ($existing['status'] ?? null) === 'verified') {
			$existingHash = $existing['manifest_hash'] ?? null;
			$candidateHash = $this->manifestHash($candidate);
			if ($existingHash !== null && $existingHash !== $candidateHash) {
				throw new RuntimeException('artifact_manifest_conflict');
			}
		}
	}

	/**
	 * Build the immutable artifact/provenance fingerprint. Mutable remote
	 * locator data such as filename, file id and path is deliberately excluded.
	 *
	 * @param array<string, mixed> $record
	 */
	private function manifestHash(array $record): string {
		$material = [
			'schema_version' => self::SCHEMA_VERSION,
			'production_id' => $record['production_id'] ?? null,
			'recording_id' => $record['recording_id'] ?? null,
			'recording_session_id' => $record['recording_session_id'] ?? null,
			'artifact_id' => $record['artifact_id'],
			'participant_label' => $record['participant_label'] ?? null,
			'remote' => [
				'size' => $record['remote']['size'] ?? null,
				'sha256' => $record['remote']['sha256'] ?? null,
			],
			'preservation' => $record['preservation'] ?? [],
			'capture' => $record['capture'] ?? null,
			'sourceSegments' => $record['sourceSegments'] ?? [],
		];

		return hash('sha256', $this->canonicalJson($material));
	}

	/**
	 * @param mixed $value
	 */
	private function canonicalJson(mixed $value): string {
		return json_encode(
			$this->sortMap($value),
			JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES | JSON_PRESERVE_ZERO_FRACTION | JSON_THROW_ON_ERROR
		);
	}

	/**
	 * @param mixed $value
	 * @return mixed
	 */
	private function sortMap(mixed $value): mixed {
		if (!is_array($value)) return $value;
		if ($this->isList($value)) {
			return array_map(fn ($item) => $this->sortMap($item), $value);
		}
		ksort($value);
		foreach ($value as $key => $item) $value[$key] = $this->sortMap($item);
		return $value;
	}

	/**
	 * @param array<mixed> $value
	 */
	private function isList(array $value): bool {
		$expected = range(0, count($value) - 1);
		return $value === [] || array_keys($value) === $expected;
	}

	private function requiredString(mixed $value, string $field): string {
		$value = $this->nullableString($value);
		if ($value === null || trim($value) === '') throw new RuntimeException('artifact_manifest_invalid');
		if (strlen($value) > 1024) throw new RuntimeException('artifact_manifest_invalid');
		return $value;
	}

	private function nullableString(mixed $value): ?string {
		if ($value === null || $value === '') return null;
		if (!is_string($value)) throw new RuntimeException('artifact_manifest_invalid');
		return $value;
	}

	private function requiredInt(mixed $value, string $field): int {
		$value = $this->nullableInt($value);
		if ($value === null) throw new RuntimeException('artifact_manifest_invalid');
		return $value;
	}

	private function requiredNonNegativeInt(mixed $value, string $field): int {
		$value = $this->requiredInt($value, $field);
		if ($value < 0) throw new RuntimeException('artifact_manifest_invalid');
		return $value;
	}

	private function requiredPositiveInt(mixed $value, string $field): int {
		$value = $this->requiredInt($value, $field);
		if ($value <= 0) throw new RuntimeException('artifact_manifest_invalid');
		return $value;
	}

	private function nullableInt(mixed $value): ?int {
		if ($value === null || $value === '') return null;
		if (is_int($value)) return $value;
		if (is_float($value) && is_finite($value) && (int)$value === $value) return (int)$value;
		if (is_string($value) && preg_match('/^\d+$/', $value) === 1) return (int)$value;
		throw new RuntimeException('artifact_manifest_invalid');
	}

	private function nullableBool(mixed $value): ?bool {
		if ($value === null || $value === '') return null;
		if (!is_bool($value)) throw new RuntimeException('artifact_provenance_invalid');
		return $value;
	}

	private function requiredSha256(mixed $value): string {
		if (!is_string($value) || preg_match('/^[a-f0-9]{64}$/i', $value) !== 1) throw new RuntimeException('artifact_manifest_invalid');
		return strtolower($value);
	}

	private function directory(): string {
		$dataDirectory = trim((string)$this->config->getSystemValue('datadirectory', ''));
		$instanceId = trim((string)$this->config->getSystemValue('instanceid', ''));
		if ($dataDirectory === '' || $instanceId === '') throw new RuntimeException('artifact_manifest_storage_unavailable');

		$directory = rtrim($dataDirectory, '/') . '/appdata_' . $instanceId . '/pore/artifacts';
		if (!is_dir($directory) && !mkdir($directory, self::DIRECTORY_MODE, true) && !is_dir($directory)) {
			throw new RuntimeException('artifact_manifest_storage_unavailable');
		}
		return $directory;
	}

	private function path(string $artifactId): string {
		return $this->directory() . '/' . hash('sha256', $artifactId) . '.json';
	}

	/**
	 * @return array<string, mixed>|null
	 */
	private function readPath(string $path): ?array {
		if (!is_file($path)) return null;
		$content = file_get_contents($path);
		if ($content === false) throw new RuntimeException('artifact_manifest_storage_unavailable');

		$decoded = json_decode($content, true, 512, JSON_THROW_ON_ERROR);
		if (!is_array($decoded)) throw new RuntimeException('artifact_manifest_storage_invalid');
		return $decoded;
	}

	/**
	 * @param callable(string): mixed $callback
	 */
	private function withArtifactLock(string $artifactId, int $lockMode, callable $callback): mixed {
		$path = $this->path($this->requiredString($artifactId, 'artifact_id'));
		return $this->withPathLock($path, $lockMode, $callback);
	}

	/**
	 * @param callable(string): mixed $callback
	 */
	private function withPathLock(string $path, int $lockMode, callable $callback): mixed {
		$lockPath = $path . '.lock';
		$lock = fopen($lockPath, 'c');
		if ($lock === false || !flock($lock, $lockMode)) {
			if (is_resource($lock)) fclose($lock);
			throw new RuntimeException('artifact_manifest_storage_unavailable');
		}

		try {
			return $callback($path);
		} finally {
			flock($lock, LOCK_UN);
			fclose($lock);
		}
	}

	/**
	 * @param array<string, mixed> $record
	 */
	private function writeLocked(string $path, array $record): void {
		$json = $this->canonicalJson($record);
		$tmp = $path . '.tmp-' . bin2hex(random_bytes(8));

		if (file_put_contents($tmp, $json . "\n", LOCK_EX) === false) {
			@unlink($tmp);
			throw new RuntimeException('artifact_manifest_storage_unavailable');
		}

		@chmod($tmp, self::FILE_MODE);
		if (!rename($tmp, $path)) {
			@unlink($tmp);
			throw new RuntimeException('artifact_manifest_storage_unavailable');
		}
		@chmod($path, self::FILE_MODE);
	}
}
