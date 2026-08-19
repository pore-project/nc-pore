#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Running architecture checks..."
echo

"$SCRIPT_DIR/check-core-boundaries.sh"
echo

"$SCRIPT_DIR/check-recorder-boundaries.sh"
echo

bash "$SCRIPT_DIR/check-application-boundaries.sh"
echo
echo "All architecture checks passed."
