<?php

declare(strict_types=1);

require_once __DIR__ . '/../lib/Service/NextcloudArtifactPath.php';

use OCA\PoRe\Service\NextcloudArtifactPath;

function check(bool $condition, string $message): void {
	if (!$condition) {
		throw new RuntimeException($message);
	}
}

$default = NextcloudArtifactPath::build('', 'prod-123', 'Interview mit Max Muster', 'capture-456', '2026-09-05T15:42:31+02:00');
check($default['root'] === 'audio', 'Empty root must use audio as the default.');
check($default['relative_path'] === 'audio/2026/09/05 - 15:42 Interview mit Max Muster - prod-123/capture-456.wav', 'Default path contract is incorrect.');

$custom = NextcloudArtifactPath::build('Büro\\interviews/', 'prod-123', 'Interview mit Max Muster', 'capture-456', '2026-09-05T15:42:31+02:00');
check($custom['root'] === 'Büro/interviews', 'Configured root must be normalized as a Files-relative path.');
check($custom['relative_path'] === 'Büro/interviews/2026/09/05 - 15:42 Interview mit Max Muster - prod-123/capture-456.wav', 'Custom root must be the complete PoRe root.');

$audioRoot = NextcloudArtifactPath::build('audio', 'prod-123', 'Interview', 'capture-456', '2026-09-05T15:42:31+02:00');
check($audioRoot['relative_path'] === 'audio/2026/09/05 - 15:42 Interview - prod-123/capture-456.wav', 'A custom audio root must not become audio/audio.');

foreach (['../escape', 'foo/../escape', '/absolute/path', '\\absolute\\path', 'C:/absolute/path', "foo\0bar"] as $invalidRoot) {
	try {
		NextcloudArtifactPath::normalizeRoot($invalidRoot);
		throw new RuntimeException('Invalid root was accepted: ' . $invalidRoot);
	} catch (RuntimeException $exception) {
		check(
			str_contains($exception->getMessage(), 'escape')
				|| str_contains($exception->getMessage(), 'Invalid path')
				|| str_contains($exception->getMessage(), 'relative Files path'),
			'Invalid root failed for an unexpected reason.',
		);
	}
}

foreach ([['prod/evil', 'Production'], ['prod', 'Label/evil'], ['prod', 'Label', 'capture/evil']] as $case) {
	$productionId = $case[0];
	$productionLabel = $case[1];
	$captureId = $case[2] ?? 'capture';
	try {
		NextcloudArtifactPath::build('', $productionId, $productionLabel, $captureId, '2026-09-05T15:42:31+02:00');
		throw new RuntimeException('Invalid artifact identity was accepted.');
	} catch (RuntimeException $exception) {
		check(str_contains($exception->getMessage(), 'Invalid path segment'), 'Invalid artifact identity failed for an unexpected reason.');
	}
}

try {
	NextcloudArtifactPath::build('', 'prod', 'Interview', 'capture', 'not-a-timestamp');
	throw new RuntimeException('Invalid timestamp was accepted.');
} catch (RuntimeException $exception) {
	check(str_contains($exception->getMessage(), 'Invalid recording start timestamp'), 'Invalid timestamp failed for an unexpected reason.');
}

echo "Nextcloud V1 storage path contract checks passed.\n";
