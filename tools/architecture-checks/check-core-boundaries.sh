#!/bin/bash

set -e

echo "Checking core boundaries..."

if grep -R "RecordingArtifact" core/src --include="*.rs"; then
    echo
    echo "ERROR:"
    echo "Core must not depend on RecordingArtifact."
    echo "See ADR-042."
    exit 1
fi

if grep -R "CaptureResult" core/src --include="*.rs"; then
    echo
    echo "ERROR:"
    echo "Core must not depend on CaptureResult."
    echo "See ADR-056."
    exit 1
fi

echo "Core boundary check passed."
