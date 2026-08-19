#!/bin/bash

set -e

echo "Checking application boundaries..."

if grep -R -E 'std::fs|std::path::|serde_json|FileProductionSessionRepository' application/src --include="*.rs"; then
    echo
    echo "ERROR:"
    echo "Application must not contain concrete filesystem persistence implementations."
    echo "See ADR-036 and ADR-052."
    exit 1
fi

echo "Application boundary check passed."
