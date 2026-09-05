<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\RuntimeClient;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use RuntimeException;

final class RecordingTransportController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly RuntimeClient $runtimeClient,
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

			$runtimeResponse = $this->runtimeClient->submitFinalizedArtifact(
				[
					'request_id' => $requestId,
					'capture_id' => $this->requiredString($decoded, 'capture_id'),
					'recording_session_id' => $this->requiredString($decoded, 'recording_session_id'),
					'production_id' => $this->requiredString($decoded, 'production_id'),
					'recording_id' => $this->requiredString($decoded, 'recording_id'),
					'track_id' => $this->requiredString($decoded, 'track_id'),
					'sample_rate_hz' => $this->positiveInt($decoded, 'sample_rate_hz'),
					'channels' => $this->positiveInt($decoded, 'channels'),
				],
				$payloadPath,
				(int)$payloadLength,
			);

			return new DataResponse([
				'protocol_version' => $runtimeResponse['protocol_version'],
				'request_id' => $runtimeResponse['request_id'],
				'status' => $runtimeResponse['status'],
				'artifact_id' => $runtimeResponse['artifact_id'] ?? null,
				'error_code' => $runtimeResponse['error_code'] ?? null,
			]);
		} catch (\Throwable $exception) {
			return new DataResponse([
				'protocol_version' => 1,
				'request_id' => $requestId,
				'status' => 'rejected',
				'artifact_id' => null,
				'error_code' => 'runtime_handoff_failed',
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

	/** @param array<string, mixed> $metadata */
	private function positiveInt(array $metadata, string $key): int {
		$value = $metadata[$key] ?? null;
		if (!is_int($value) && !is_string($value)) {
			throw new RuntimeException(sprintf('Metadata field "%s" is required.', $key));
		}
		$value = filter_var($value, FILTER_VALIDATE_INT);
		if ($value === false || $value < 1) {
			throw new RuntimeException(sprintf('Metadata field "%s" must be a positive integer.', $key));
		}
		return $value;
	}
}
