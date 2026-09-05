<?php

declare(strict_types=1);

namespace OCA\PoRe\Controller;

use OCP\AppFramework\Http\JSONResponse;
use OCP\AppFramework\Http\Response;
use OCP\AppFramework\OCS\OCSController;
use OCP\IRequest;

/**
 * HTTP adapter for browser-finalized recording artifacts.
 *
 * This controller is deliberately transport-only. It validates the request
 * envelope and is the future composition point for the application boundary;
 * it does not write recording payloads or implement WebDAV synchronization.
 */
class BrowserRecordingArtifactController extends OCSController {
	public function __construct(string $appName, IRequest $request) {
		parent::__construct($appName, $request);
	}

	/**
	 * Accepts the browser artifact handoff.
	 *
	 * V1 currently exposes the transport boundary without persisting the
	 * payload until the application composition root is wired to the Rust
	 * application service. Returning 501 makes that state explicit instead of
	 * pretending the artifact was accepted.
	 */
	public function submit(): Response {
		return new JSONResponse(
			[
				'error' => 'browser artifact persistence is not wired',
			],
			501,
		);
	}
}
