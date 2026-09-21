<?php

declare(strict_types=1);

namespace OCA\PoRe\Http;

use Closure;
use OCP\AppFramework\Http\ICallbackResponse;
use OCP\AppFramework\Http\IOutput;
use OCP\AppFramework\Http\Response;

final class CoordinationEventStreamResponse extends Response implements ICallbackResponse {
	public function __construct(
		private readonly Closure $callback,
	) {
		parent::__construct();
		$this->cacheFor(0);
		$this->addHeader('Content-Type', 'text/event-stream; charset=UTF-8');
		$this->addHeader('Cache-Control', 'no-cache, no-transform');
		$this->addHeader('X-Accel-Buffering', 'no');
		$this->addHeader('X-Content-Type-Options', 'nosniff');
	}

	public function callback(IOutput $output): void {
		($this->callback)(
			static function (string $chunk): void {
				echo $chunk;
				if (function_exists('ob_flush')) @ob_flush();
				flush();
			},
		);
	}
}
