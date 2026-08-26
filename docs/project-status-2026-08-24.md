# NC-PoRe Project Status

- Version: 4.0
- Date: 2026-08-24
- Baseline: `develop` / `afa008a`

---

# Deutsch

## Projektphase

NC-PoRe befindet sich in der **V1-Härtungs- und Produktisierungsphase**.

Die wesentlichen lokalen Recording-, Persistenz- und Synchronisationsgrenzen sind implementiert. Der technische Stand ist inzwischen nicht mehr nur durch Unit- und Integrationstests abgesichert: Der vollständige reale Pfad von lokaler CPAL-Aufnahme bis zur Verifikation eines synchronisierten RecordingArtifacts in Nextcloud wurde auf dem Arbeitsrechner erfolgreich ausgeführt.

## Aktueller technischer Stand

### Core / Domain

Implementiert und validiert sind insbesondere:

- ProductionSession und Recording-Lifecycle
- Participant-/Participation-Semantik und sessionbezogene Rollen
- Activity History
- fachliche Lifecycle- und Berechtigungsinvarianten
- stabile Application/API-Grenzen für Production Management
- RecordingArtifact als technische Synchronisationseinheit
- provider-neutrale technische Grenzen zwischen Core, Recorder, Persistence und Remote Transfer

### Recorder

Implementiert und validiert sind insbesondere:

- CPAL-basierter konkreter CaptureProvider
- realer lokaler Audio-Capture
- CaptureResult mit Tracks und Chunks
- RecordingArtifact-Erzeugung
- Recording-Konfiguration entlang der Capture-to-Artifact-Grenze
- Chunk-Sample-Offsets
- lokaler RecordingArtifact-Persistenzpfad
- Recovery und Konsistenzbewertung
- Lifecycle- und Fehlerbehandlung

### Persistence

Implementiert und validiert sind insbesondere:

- Filesystem Persistence Provider
- vollständige Persistenz von Recording-Payloads
- Persistenz der Track-Konfiguration
- Persistenz der Chunk-Sample-Offsets
- Idempotenz äquivalenter Artifacts
- Konflikt- und Integritätsverhalten
- Schutz unvollständiger persistierter Artifacts

### Synchronisation / Nextcloud

Implementiert und validiert sind inzwischen:

- persistente Synchronisationsarbeit
- vendor-neutrale Artifact-Transfer-Grenze
- idempotente und integritätsgesicherte Transfersemantik
- Synchronisations-Recovery und Retry-/Offline-first-Orchestrierung
- produktiver Nextcloud-Provider für V1
- reale Nextcloud-Synchronisationsprüfung

## Reality Check – 2026-08-24

Der entscheidende technische End-to-End-Reality-Check wurde erfolgreich ausgeführt:

1. echtes Audio wurde über CPAL vom lokalen Eingabegerät aufgenommen,
2. daraus wurde ein RecordingArtifact erzeugt,
3. das Artifact wurde lokal persistiert,
4. die Synchronisation wurde ausgeführt,
5. das Artifact wurde nach Nextcloud übertragen,
6. der Remote-Zustand wurde verifiziert,
7. anschließend wurde der Testzustand wieder bereinigt.

Der Test `nextcloud_real_recording_reality_check` meldete:

> real CPAL capture, artifact persistence, synchronization, remote verification and cleanup succeeded

Ergebnis: **1 passed, 0 failed**; Laufzeit ca. 65 Sekunden.

Damit ist der komplette technische Pfad **CPAL → RecordingArtifact → lokale Persistenz → Synchronisation → Nextcloud → Remote-Verifikation → Cleanup** real nachgewiesen.

## Tests

Der vollständige Recorder-Testlauf auf dem Arbeitsrechner ergab:

- **129 Recorder-Tests: 129 passed, 0 failed**
- Filesystem-Persistence-Boundary-Test: **passed**
- gezielter Test für persistierte Recording-Konfiguration: **passed**
- `cargo fmt -- --check`: **passed**
- realer Nextcloud-Recording-Reality-Check: **passed**

Es bestehen derzeit einige Compiler-Warnungen zu ungenutzten Imports bzw. `unused_mut`; sie blockieren den erfolgreichen Testlauf nicht und sind als eigener Cleanup-Punkt zu betrachten.

## Architekturstand

Die aktuelle Architektur folgt weiterhin insbesondere diesen Grenzen:

- Core bleibt Autorität für fachliche Regeln und Lifecycle-Semantik.
- RecordingArtifact bleibt von fachlichen Domainobjekten getrennt.
- Capture, Persistence und Remote Transfer bleiben technische Grenzen.
- Provider-spezifische WebDAV-/Nextcloud-Details gelangen nicht in Core.
- Lokale Aufnahme bleibt unabhängig von Netzwerkverfügbarkeit.
- Remote-Zustand gilt erst nach erfolgreicher Vollständigkeits- und Integritätsprüfung als synchronisiert.
- Das vollständige RecordingArtifact bleibt die Synchronisationseinheit.

ADR-068 ist akzeptiert und bildet die Grundlage für den gemeinsamen Recording-Start sowie Opening-/Closing-Sync-Signet.

## Abgeschlossene Synchronisationsarbeit

Die ursprünglich im Projektstatus als nächste Schritte genannten Issues

- #143 – persistent synchronization queue / pending-transfer boundary
- #144 – vendor-neutral artifact transfer boundary
- #145 – resumable, idempotent artifact transfer semantics
- #146 – synchronization recovery, retry, and offline-first orchestration

sind inzwischen **alle abgeschlossen**. Der bestehende Projektstatus war an dieser Stelle veraltet.

## Repository-Zustand

Der aktuelle Repository-Stand ist konsolidiert:

- `origin/develop` → `afa008a`
- `origin/main` → `a5b2672`
- `main` und `develop` besitzen denselben Tree
- `main` und `develop` sind inhaltlich vollständig synchronisiert
- temporäre Arbeitsbranches aus dem abgeschlossenen PR-/Merge-Prozess sind entfernt
- aktuell bestehen keine offenen Issues oder Pull Requests

Die unterschiedliche Commit-ID von `main` und `develop` ist aufgrund des Merge-Commits auf `main` erwartbar; beide Branches zeigen auf denselben Projekt-Tree.

## Nächster Entwicklungsschnitt

Es wird **kein bereits erledigtes Synchronisations-Issue erneut geöffnet oder künstlich fortgeschrieben**.

Da der komplette reale Recording-/Nextcloud-Pfad inzwischen nachgewiesen ist und aktuell keine offenen Issues bestehen, wird der nächste Entwicklungsschnitt aus dem tatsächlichen Architektur- und Produktstand neu bestimmt.

Vor der nächsten Implementierung erfolgt deshalb eine kritische Bestandsaufnahme von ADRs, Implementierung und verbleibenden V1-Lücken. Erst danach wird der nächste konkrete technische Schnitt als Issue/PR festgelegt.

---

# English

NC-PoRe is in the **V1 hardening and productization phase**.

As of 2026-08-24, the complete real technical path from local CPAL capture through RecordingArtifact creation, local persistence, synchronization, Nextcloud transfer, remote verification and cleanup has been successfully validated on the development workstation.

The full recorder suite passed with **129/129 tests**, formatting passed, the targeted persistence test passed, and the real Nextcloud recording reality check passed.

The previously listed synchronization issues #143–#146 are all completed. The old project-status wording was therefore outdated.

The repository is consolidated with `origin/develop` at `afa008a` and `origin/main` at `a5b2672`; both branches have the same tree. There are currently no open issues or pull requests.

The next development slice will therefore be selected from a fresh critical review of the current ADRs, implementation state and remaining V1 gaps rather than from already completed synchronization work.
