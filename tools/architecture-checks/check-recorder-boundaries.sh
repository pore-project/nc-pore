#!/bin/bash

set -e

echo "Checking recorder boundaries..."

if grep -R "ProductionSession" recorder/src --include="*.rs"; then
    echo
    echo "ERROR:"
    echo "Recorder must not depend on ProductionSession."
    echo "See ADR-041 and ADR-042."
    exit 1
fi

if grep -R "ParticipantRole" recorder/src --include="*.rs"; then
    echo
    echo "ERROR:"
    echo "Recorder must not depend on domain roles."
    echo "See ADR-042."
    exit 1
fi

echo "Recorder boundary check passed."
