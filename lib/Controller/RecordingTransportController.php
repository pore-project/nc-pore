<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCA\PoRe\AppInfo\Application;
use OCA\PoRe\Service\NextcloudArtifactConnector;
use OCP\AppFramework\Http\Attribute\NoAdminRequired;
use OCP\AppFramework\Http\DataResponse;
use OCP\AppFramework\OCSController;
use OCP\IRequest;
use RuntimeException;

final class RecordingTransportController extends OCSController {
	public function __construct(
		IRequest $request,
		private readonly NextcloudArtifactConnector $connector,
	) {
		parent::__construct(Application::APP_ID, $request);
	}

	#[NoAdminRequired]
	public function prepareFinalizedArtifact(
		string $production_id,
		string $production_label,
		string $recording_id,
		string $capture_id,
		string $started_at,
		string $participant_label,
		int $size,
		string $payload_sha256,
	): DataResponse {
		try {
			$prepared = $this->connector->prepare(
				$this->required($production_id, 'production_id'),
				$this->required($production_label, 'production_label'),
				$this->required($recording_id, 'recording_id'),
				$this->required($capture_id, 'capture_id'),
				$this->required($started_at, 'started_at'),
				$participant_label,
				$size,
				$this->required($payload_sha256, 'payload_sha256'),
			);
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'prepared',
				...$prepared,
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			return $this->rejected($exception);
		}
	}

	#[NoAdminRequired]
	public function verifyFinalizedArtifact(string $transfer_id): DataResponse {
		try {
			$receipt = $this->connector->verify($this->required($transfer_id, 'transfer_id'));
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'verified',
				...$receipt,
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			return $this->rejected($exception);
		}
	}

	#[NoAdminRequired]
	public function closeFinalizedArtifactTransfer(string $transfer_id): DataResponse {
		try {
			$this->connector->close($this->required($transfer_id, 'transfer_id'));
			return new DataResponse([
				'protocol_version' => 2,
				'status' => 'closed',
				'error_code' => null,
			]);
		} catch (\Throwable $exception) {
			return $this->rejected($exception);
		}
	}

	private function required(string $value, string $name): string {
		if (trim($value) === '') throw new RuntimeException(sprintf('%s is required.', $name));
		return $value;
	}

	private function rejected(\Throwable $exception): DataResponse {
		return new DataResponse([
			'protocol_version' => 2,
			'status' => 'rejected',
			'error_code' => $exception->getMessage(),
		], 500);
	}
}
