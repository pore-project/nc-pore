<?php

declare(strict_types=1);

return [
	'ocs' => [
		[
			'name' => 'RecordingTransport#submitFinalizedArtifact',
			'url' => '/v1/recordings/finalized-artifact',
			'verb' => 'POST',
		],
		[
			'name' => 'Recording#command',
			'url' => '/v1/recordings/command',
			'verb' => 'POST',
		],
		[
			'name' => 'Production#command',
			'url' => '/v1/productions/command',
			'verb' => 'POST',
		],
		[
			'name' => 'Settings#getSettings',
			'url' => '/v1/settings',
			'verb' => 'GET',
		],
		[
			'name' => 'Settings#setSettings',
			'url' => '/v1/settings',
			'verb' => 'PUT',
		],
	],
];
