<?php

declare(strict_types=1);

namespace OCA\PoRe\Service;

use RuntimeException;

final class NextcloudArtifactPath {
	public const DEFAULT_STORAGE_ROOT = 'audio';

	/**
	 * @return array{relative_path:string, root:string, year:string, month:string, leaf:string, filename:string}
	 */
	public static function build(
		string $configuredRoot,
		string $productionId,
		string $productionLabel,
		string $captureId,
		string $startedAt,
	): array {
		$root = self::normalizeRoot($configuredRoot);
		if ($root === '') {
			$root = self::DEFAULT_STORAGE_ROOT;
		}

		self::assertSegment($productionId);
		self::assertSegment($productionLabel);
		self::assertSegment($captureId);

		try {
			$timestamp = new \DateTimeImmutable($startedAt);
		} catch (\Exception) {
			throw new RuntimeException('Invalid recording start timestamp.');
		}

		$year = $timestamp->format('Y');
		$month = $timestamp->format('m');
		$leaf = $timestamp->format('d - H:i') . ' ' . $productionLabel . ' - ' . $productionId;
		$filename = $captureId . '.wav';

		return [
			'relative_path' => implode('/', [$root, $year, $month, $leaf, $filename]),
			'root' => $root,
			'year' => $year,
			'month' => $month,
			'leaf' => $leaf,
			'filename' => $filename,
		];
	}

	public static function normalizeRoot(string $configuredRoot): string {
		$raw = trim($configuredRoot);
		if ($raw === '' || $raw === '.') {
			return '';
		}
		if (str_starts_with($raw, '/') || str_starts_with($raw, '\\') || preg_match('/^[A-Za-z]:[\\\\\/]/', $raw) === 1) {
			throw new RuntimeException('Configured PoRe storage root must be a relative Files path.');
		}

		$root = preg_replace('#[\\/]+#', '/', trim($raw, "\\/")) ?? '';
		if ($root === '' || $root === '.') {
			return '';
		}
		if ($root === '..' || str_starts_with($root, '../') || str_contains($root, '/../')) {
			throw new RuntimeException('Configured PoRe storage root may not escape the user Files root.');
		}

		foreach (explode('/', $root) as $segment) {
			self::assertSegment($segment);
		}

		return $root;
	}

	private static function assertSegment(string $value): void {
		if ($value === '' || $value === '.' || $value === '..' || strpbrk($value, '/\\') !== false || str_contains($value, "\0")) {
			throw new RuntimeException('Invalid path segment in finalized artifact identity.');
		}
	}
}
