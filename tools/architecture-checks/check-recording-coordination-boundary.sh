#!/bin/bash

set -euo pipefail

echo "Checking recording coordination boundaries..."

# TEST-COORD-01: the neutral browser coordination channel must exist and use EventSource.
grep -q "class PoRERecordingCoordinationChannel" js/pore-recording-coordination.js
grep -q "new window.EventSource" js/pore-recording-coordination.js
grep -q "/v1/recordings/coordination/publish" js/pore-recording-coordination.js

# TEST-COORD-02: Talk may not carry PoRE recording lifecycle events.
grep -q "const url = path => window.OC?.generateUrl ? window.OC.generateUrl(path) : path" js/pore-talk-recording-host.js
if grep -nE 'sendToAll|SimpleWebRTC|signalingConnection|signalingMessageHandler|connection\.on\(.message' js/pore-talk-recording-host.js; then
    echo "ERROR: Talk signaling is still used by the PoRE recording host adapter."
    exit 1
fi

# TEST-COORD-03: the PoRE coordination service/controller may not depend on Talk.
if grep -R -nE 'apps/spreed|SimpleWebRTC|sendToAll' \
    lib/Service/RecordingCoordinationService.php \
    lib/Controller/RecordingCoordinationController.php \
    lib/Http/CoordinationEventStreamResponse.php; then
    echo "ERROR: Recording coordination server code contains a Talk dependency."
    exit 1
fi

# TEST-COORD-04: the new PHP transport surface must pass syntax validation.
php -l lib/Service/RecordingCoordinationService.php >/dev/null
php -l lib/Controller/RecordingCoordinationController.php >/dev/null
php -l lib/Controller/RecordingCoordinationEventController.php >/dev/null
php -l lib/Service/TalkSessionAccessService.php >/dev/null
php -l lib/Http/CoordinationEventStreamResponse.php >/dev/null

# TEST-COORD-05: the app must load the neutral channel before the Talk host adapter.
grep -q "pore-recording-coordination" lib/AppInfo/Application.php
grep -q "RecordingCoordinationEvent#events" appinfo/routes.php
grep -q "final class RecordingCoordinationEventController extends Controller" lib/Controller/RecordingCoordinationEventController.php
grep -q "authorizeRecordingAccess" lib/Service/RecordingCoordinationService.php
grep -q "'Snapshot' => null" lib/Service/RecordingCoordinationService.php
grep -q "TalkSessionAccessService" lib/Controller/RecordingCoordinationEventController.php
grep -q "isParticipant" lib/Service/TalkSessionAccessService.php
grep -q 'public function events' lib/Controller/RecordingCoordinationEventController.php
grep -q "sessionId = ''" lib/Controller/RecordingCoordinationEventController.php
grep -q "recordingId = ''" lib/Controller/RecordingCoordinationEventController.php
grep -q "RecordingCoordination#publish" appinfo/routes.php

coord_line=$(grep -n "pore-recording-coordination" lib/AppInfo/Application.php | cut -d: -f1)
host_line=$(grep -n "pore-talk-recording-host" lib/AppInfo/Application.php | cut -d: -f1)
if [ "$coord_line" -ge "$host_line" ]; then
    echo "ERROR: Neutral coordination channel loads after the Talk host adapter."
    exit 1
fi

# TEST-COORD-06: lifecycle state remains authoritative in Core; browser signals trigger a snapshot.
grep -q "const handleRecordingSignal = async event" js/init.js
grep -q "await synchronizeCoordinatorState()" js/init.js

# TEST-COORD-07: the production lifecycle must remain free of polling timers.
if grep -nE "setInterval\(|pollCoordination|startCoordinationPolling|coordinationPollTimer" js/init.js js/pore-talk-recording-host.js; then
    echo "ERROR: recording coordination polling was reintroduced."
    exit 1
fi

# TEST-COORD-08: pre-production coordination may open before a recording exists,
# but the Talk-specific participant check must remain at the adapter boundary.
grep -q "coordination_recording_not_found" lib/Controller/RecordingCoordinationEventController.php
grep -q "TalkSessionAccessService" lib/Controller/RecordingCoordinationEventController.php
grep -q 'isParticipant($sessionId, $user->getUID())' lib/Controller/RecordingCoordinationEventController.php

echo "Recording coordination boundary checks passed."
