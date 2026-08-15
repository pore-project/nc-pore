# NC-PoRe Milestone — CPAL Capture Integration

- Date: 2026-08-15
- Phase: Technical Implementation
- Related commit: 7d34d8c `feat: connect CPAL capture to recorder application`

---

## Deutsch

### Ergebnis

Der Recorder besitzt erstmals eine konkrete CPAL-basierte CaptureProvider-Implementierung. Ein lokales Standard-Eingabegerät kann über CPAL geöffnet und ein Input-Stream erfolgreich gestartet werden.

Der technische Capture-Pfad wurde bis zum bestehenden Recorder Application Flow geführt:

CPAL Input Device → CpalCaptureProvider → CaptureResult → RecordingArtifact → Persistence

### Nachgewiesene Integration

Die technische Integration wurde auf dem Entwicklungsrechner erfolgreich ausgeführt:

- Standard-Eingabegerät erkannt
- Default Input Configuration: 2 Kanäle, 48000 Hz, F32
- Input-Stream erfolgreich gestartet
- 95232 Samples innerhalb des Testintervalls empfangen
- 380928 Payload-Bytes in einem CaptureChunk erzeugt
- CaptureTrack mit einem Chunk erzeugt
- CaptureResult mit einem Track erzeugt
- vollständiger Recorder Application Flow mit CpalCaptureProvider ausgeführt
- RecordingArtifact mit einem Track erzeugt und durch den bestehenden Persistenzpfad verarbeitet

### Architekturgrenze

CpalCaptureProvider implementiert die bestehende CaptureProvider-Schnittstelle. Die konkrete Capture-Technologie bleibt damit hinter der technischen Capture-Grenze verborgen.

CaptureResult und RecordingArtifact bleiben getrennte Datenmodelle. Die Umwandlung erfolgt weiterhin explizit über RecordingArtifactFactory und wird durch den bestehenden Artifact Processing Flow koordiniert.

### Bewusste Begrenzung

Die aktuelle CPAL-Implementierung ist noch keine vollständige produktionsgeeignete Recording-Implementierung. Sie dient als technische Integrationsstufe und verwendet derzeit:

- Default Input Configuration
- F32 als Capture-Sampletyp
- einen gemeinsamen Sample-Puffer
- einen CaptureChunk beim Stop der Aufnahme
- rohe Little-Endian-F32-Sample-Bytes als Payload

Noch nicht festgelegt oder umgesetzt sind insbesondere:

- verbindliche Recording-Konfiguration
- tatsächliches Zielformat der Audio-Payload
- kontrollierte Chunk-Erzeugung während längerer Aufnahmen
- produktionsgeeignete Stream-Fehlerbehandlung
- vollständige Lifecycle- und Ressourcen-Semantik
- Umgang mit unterschiedlichen CPAL-Sample-Formaten

Diese Punkte werden als nächste technische Ausbaustufe behandelt und nicht stillschweigend Teil der CaptureResult- oder Artifact-Grenzen gemacht.

### Bedeutung für den Projektstand

Der bisherige CaptureResult-/RecordingArtifact-Datenpfad ist nicht mehr nur durch Testdaten validiert. Er kann nun mit tatsächlich von einem lokalen Audiogerät gelieferten Samples durchlaufen werden.

Damit ist die technische Integration von Audio-Capture in die bestehende Recorder-Architektur nachgewiesen. Die nächste Aufgabe besteht darin, daraus eine definierte Recording-Implementierung zu entwickeln, ohne die bestehenden Architekturgrenzen aufzuweichen.

---

## English Version

### Result

The recorder now has a concrete CPAL-based CaptureProvider implementation. A local default input device can be opened through CPAL and an input stream can be started successfully.

The technical capture path has been connected to the existing Recorder Application Flow:

CPAL Input Device → CpalCaptureProvider → CaptureResult → RecordingArtifact → Persistence

### Demonstrated Integration

The technical integration was successfully executed on the development machine:

- default input device detected
- default input configuration: 2 channels, 48000 Hz, F32
- input stream started successfully
- 95232 samples received during the test interval
- 380928 payload bytes produced in one CaptureChunk
- CaptureTrack created with one chunk
- CaptureResult created with one track
- complete Recorder Application Flow executed with CpalCaptureProvider
- RecordingArtifact with one track created and processed through the existing persistence path

### Architecture Boundary

CpalCaptureProvider implements the existing CaptureProvider interface. The concrete capture technology therefore remains behind the technical capture boundary.

CaptureResult and RecordingArtifact remain separate data models. Conversion continues to happen explicitly through RecordingArtifactFactory and is coordinated by the existing Artifact Processing Flow.

### Intentional Limitation

The current CPAL implementation is not yet a production-ready recording implementation. It is a technical integration stage and currently uses:

- default input configuration
- F32 as capture sample type
- one shared sample buffer
- one CaptureChunk when capture stops
- raw little-endian F32 sample bytes as payload

The following are not yet defined or implemented in particular:

- mandatory recording configuration
- actual target format of the audio payload
- controlled chunk creation during longer recordings
- production-ready stream error handling
- complete lifecycle and resource semantics
- handling of different CPAL sample formats

These topics will be addressed as the next technical implementation stage and will not be implicitly pushed into the CaptureResult or Artifact boundaries.

### Significance for Project Status

The existing CaptureResult/RecordingArtifact data path is no longer validated only with synthetic test data. It can now be traversed using samples actually delivered by a local audio device.

This demonstrates the technical integration of audio capture into the existing recorder architecture. The next task is to turn this into a defined recording implementation without weakening the existing architecture boundaries.
