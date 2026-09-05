<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\NextcloudArtifactStorage;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use RuntimeException;

final class RecordingTransportController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly NextcloudArtifactStorage $artifactStorage,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[NoAdminRequired]
	public function submitFinalizedArtifact(string $metadata): DataResponse {
		$requestId = '';

		try {
			$decoded = json_decode($metadata, true, 512, JSON_THROW_ON_ERROR);
			if (!is_array($decoded)) {
				throw new RuntimeException('Metadata must be a JSON object.');
			}

			$requestId = $this->requiredString($decoded, 'request_id');
			$payloadFile = $this->request->getUploadedFile('payload');
			if (!is_array($payloadFile) || !isset($payloadFile['tmp_name'], $payloadFile['error'])) {
				throw new RuntimeException('Finalized artifact payload is missing.');
			}
			if ((int)$payloadFile['error'] !== UPLOAD_ERR_OK) {
				throw new RuntimeException('Finalized artifact upload failed.');
			}

			$payloadPath = (string)$payloadFile['tmp_name'];
			$payloadLength = filesize($payloadPath);
			if ($payloadLength === false) {
				throw new RuntimeException('Unable to determine finalized artifact size.');
			}

			$expectedHash = strtolower($this->requiredString($decoded, 'payload_sha256'));
			if (!preg_match('/^[a-f0-9]{64}$/', $expectedHash)) {
				throw new RuntimeException('payload_sha256 must be a SHA-256 hex digest.');
			}
			$actualHash = hash_file('sha256', $payloadPath);
			if ($actualHash === false || !hash_equals($expectedHash, strtolower($actualHash))) {
				throw new RuntimeException('Uploaded payload does not match the browser transfer hash.');
			}

			$stored = $this->artifactStorage->storeFinalizedArtifact(
				$this->requiredString($decoded, 'production_id'),
				$this->requiredString($decoded, 'production_label'),
				$this->requiredString($decoded, 'recording_id'),
				$this->requiredString($decoded, 'capture_id'),
				$this->requiredString($decoded, 'started_at'),
				$payloadPath,
				(int)$payloadLength,
			);

			return new DataResponse([
				'protocol_version' => 1,
				'request_id' => $requestId,
				'status' => 'stored',
				'artifact_id' => $this->requiredString($decoded, 'capture_id'),
				'file_id' => $stored['file_id'],
				'path' => $stored['path'],
				'size' => $stored['size'],
				'sha256' => $stored['sha256'],
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			return new DataResponse([
				'protocol_version' => 1,
				'request_id' => $requestId,
				'status' => 'rejected',
				'artifact_id' => null,
				'file_id' => null,
				'path' => null,
				'size' => null,
				'sha256' => null,
				'error_code' => 'nextcloud_storage_failed',
			], 500);
		}
	}

	/** @param array<string, mixed> $metadata */
	private function requiredString(array $metadata, string $key): string {
		$value = $metadata[$key] ?? null;
		if (!is_string($value) || trim($value) === '') {
			throw new RuntimeException(sprintf('Metadata field "%s" is required.', $key));
		}
		return $value;
	}
}
