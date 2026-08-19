#!/bin/bash

set -e

echo "Checking core boundaries..."

if grep -R -E '(^|[^A-Za-z0-9_])RecordingArtifact([^A-Za-z0-9_]|$)' core/src --include="*.rs" | grep -vE ':[[:space:]]*(//|///|//!|\*)'; then
    echo
    echo "ERROR:"
    echo "Core must not depend on RecordingArtifact."
    echo "See ADR-042."
    exit 1
fi

if grep -R -E '(^|[^A-Za-z0-9_])CaptureResult([^A-Za-z0-9_]|$)' core/src --include="*.rs" | grep -vE ':[[:space:]]*(//|///|//!|\*)'; then
    echo
    echo "ERROR:"
    echo "Core must not depend on CaptureResult."
    echo "See ADR-056."
    exit 1
fi

echo "Core boundary check passed."
