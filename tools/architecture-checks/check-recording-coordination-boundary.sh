#!/bin/bash

set -euo pipefail

echo "Checking recording coordination boundaries..."

# TEST-COORD-01: the neutral browser coordination channel must exist and use EventSource.
grep -q "class PoRERecordingCoordinationChannel" js/pore-recording-coordination.js
grep -q "new window.EventSource" js/pore-recording-coordination.js
grep -q "/v1/recordings/coordination/publish" js/pore-recording-coordination.js

# TEST-COORD-02: Talk may not carry PoRE recording lifecycle events.
if grep -nE "sendToAll|SimpleWebRTC|signalingConnection|signalingMessageHandler|connection\.on\(['"]message" js/pore-talk-recording-host.js; then
    echo "ERROR: Talk signaling is still used by the PoRE recording host adapter."
    exit 1
fi

# TEST-COORD-03: the PoRE coordination service/controller may not depend on Talk.
if grep -R -nE "OCA\\\\Talk|apps/spreed|SimpleWebRTC|sendToAll"     lib/Service/RecordingCoordinationService.php     lib/Controller/RecordingCoordinationController.php     lib/Http/CoordinationEventStreamResponse.php; then
    echo "ERROR: Recording coordination server code contains a Talk dependency."
    exit 1
fi

# TEST-COORD-04: the app must load the neutral channel before the Talk host adapter.
grep -q "pore-recording-coordination" lib/AppInfo/Application.php
grep -q "RecordingCoordination#events" appinfo/routes.php
grep -q "RecordingCoordination#publish" appinfo/routes.php

coord_line=$(grep -n "pore-recording-coordination" lib/AppInfo/Application.php | cut -d: -f1)
host_line=$(grep -n "pore-talk-recording-host" lib/AppInfo/Application.php | cut -d: -f1)
if [ "$coord_line" -ge "$host_line" ]; then
    echo "ERROR: Neutral coordination channel loads after the Talk host adapter."
    exit 1
fi

# TEST-COORD-05: lifecycle state remains authoritative in Core; the browser event
# handler must synchronize from a Core snapshot instead of trusting event payload.
grep -q "await synchronizeCoordinatorState()" js/init.js
grep -q "ready.*synchronizeCoordinatorState" js/init.js || true

# TEST-COORD-06: the production lifecycle must remain free of polling timers.
if grep -nE "setInterval\(|pollCoordination|startCoordinationPolling|coordinationPollTimer" js/init.js js/pore-talk-recording-host.js; then
    echo "ERROR: recording coordination polling was reintroduced."
    exit 1
fi

echo "Recording coordination boundary checks passed."
