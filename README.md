# NC-PoRe

## Deutsch ([English version below](#english-version))

---

# NC-PoRe

NC-PoRe ist eine selbsthostbare Open-Source-Plattform für professionelle Podcast-Aufnahmen und Produktion.

Das zentrale Prinzip:

> Meine Daten gehören mir.

NC-PoRe ermöglicht verteilte Podcast-Produktion, bei der Audio lokal aufgenommen und anschließend mit der eigenen Infrastruktur synchronisiert wird.

Die aktuelle Implementierung umfasst inzwischen die zentrale Domain- und Recorder-Grundlage, lokale Recording-Artefakte und Persistenz, Recovery, die vendor-neutrale Synchronisationsgrenze sowie einen produktiven Nextcloud-Provider für V1.

## Projektstatus

NC-PoRe befindet sich in der **V1-Härtungs- und Produktisierungsphase**.

Der vollständige reale technische Pfad von lokaler CPAL-Aufnahme über RecordingArtifact-Erzeugung, lokale Persistenz, Synchronisation und Nextcloud-Transfer bis zur Remote-Verifikation und Bereinigung wurde am 2026-08-24 erfolgreich validiert.

Die technische Grundlage für lokale Aufnahme, persistierte Recording-Artefakte und die Synchronisation nach Nextcloud ist damit nicht nur durch Tests, sondern auch durch einen realen End-to-End-Reality-Check abgesichert. Die ursprünglich als nächste Synchronisationsarbeiten dokumentierten Issues #143–#146 sind abgeschlossen.

Der nächste Entwicklungsschnitt wird daher aus einer kritischen Bestandsaufnahme des aktuellen Architektur- und Implementierungsstands sowie der verbleibenden V1-Lücken bestimmt.

Für eine öffentliche V1-Freigabe gelten insbesondere:

- Nextcloud ist der einzige Remote-Provider.
- Nextcloud-Verbindungen verwenden ausschließlich HTTPS.
- Das vollständige `RecordingArtifact` bleibt die Synchronisationseinheit.
- Provider-spezifische WebDAV-/Upload-Details bleiben außerhalb des Core.
- Remote-Zustand darf nur bei nachgewiesener Vollständigkeit und Integrität als synchronisiert gelten.

Weitere Informationen:

* `docs/project-status-2026-08-24.md` — aktueller Projektstand
* `docs/project-status.md` — historischer Statusstand vom 2026-08-21
* `docs/architecture/` — Architektur und technische Grundlagen
* `docs/implementation/` — Umsetzung und V1-Planung
* `adr/` — Architecture Decision Records

---

# Entwicklung

NC-PoRe wird als Open-Source-Projekt unter der **AGPL-3.0 Lizenz** entwickelt.

Grundprinzipien:

* Open Source First
* nachvollziehbare Entscheidungen
* kleine vollständige Entwicklungsschritte
* offene Standards
* Qualität vor Geschwindigkeit

---

# English Version ([Deutsche Version oben](#deutsch))

---

# NC-PoRe

NC-PoRe is a self-hostable open-source platform for professional podcast recording and production.

The central principle:

> My data belongs to me.

NC-PoRe enables distributed podcast production by recording audio locally and synchronizing it afterwards with the user's own infrastructure.

The current implementation includes the core domain and recorder foundation, local recording artifacts and persistence, recovery, the vendor-neutral synchronization boundary, and a productive Nextcloud provider for V1.

## Project Status

NC-PoRe is currently in the **V1 hardening and productization phase**.

As of 2026-08-24, the complete real technical path from local CPAL capture through RecordingArtifact creation, local persistence, synchronization, Nextcloud transfer, remote verification and cleanup has been successfully validated.

The technical foundation for local recording, persisted recording artifacts, and synchronization to Nextcloud is therefore validated by both automated tests and a real end-to-end reality check. The previously listed synchronization issues #143–#146 are completed.

The next development slice will be selected from a critical review of the current architecture and implementation state and the remaining V1 gaps.

For a public V1 release, the following principles apply in particular:

- Nextcloud is the only remote provider.
- Nextcloud connections use HTTPS only.
- The complete `RecordingArtifact` remains the synchronization unit.
- Provider-specific WebDAV/upload details remain outside the Core.
- Remote state is only considered synchronized after completeness and integrity have been established.

Further information:

* `docs/project-status-2026-08-24.md` — current project status
* `docs/project-status.md` — historical status snapshot from 2026-08-21
* `docs/architecture/` — architecture and technical foundations
* `docs/implementation/` — implementation and V1 planning
* `adr/` — Architecture Decision Records

---

# Development

NC-PoRe is developed as an open-source project under the **AGPL-3.0 license**.

Core principles:

* Open Source First
* traceable decisions
* small complete development steps
* open standards
* quality over speed
