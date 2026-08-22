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

Die technische Grundlage für lokale Aufnahme, persistierte Recording-Artefakte und die Synchronisation nach Nextcloud ist implementiert. Der nächste Schwerpunkt liegt auf Integritäts- und End-to-End-Härtung, Bereinigung von Legacy-Bestandteilen, konsistenter Projektdokumentation und der Fertigstellung des ersten nutzbaren Clients.

Für eine öffentliche V1-Freigabe gelten insbesondere:

- Nextcloud ist der einzige Remote-Provider.
- Nextcloud-Verbindungen verwenden ausschließlich HTTPS.
- Das vollständige `RecordingArtifact` bleibt die Synchronisationseinheit.
- Provider-spezifische WebDAV-/Upload-Details bleiben außerhalb des Core.
- Remote-Zustand darf nur bei nachgewiesener Vollständigkeit und Integrität als synchronisiert gelten.

Weitere Informationen:

* `docs/project-status.md` — aktueller Projektstand
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

The technical foundation for local recording, persisted recording artifacts, and synchronization to Nextcloud is implemented. The next focus is integrity and end-to-end hardening, cleanup of legacy components, documentation consolidation, and completion of the first usable client.

For a public V1 release, the following principles apply in particular:

- Nextcloud is the only remote provider.
- Nextcloud connections use HTTPS only.
- The complete `RecordingArtifact` remains the synchronization unit.
- Provider-specific WebDAV/upload details remain outside the Core.
- Remote state is only considered synchronized after completeness and integrity have been established.

Further information:

* `docs/project-status.md` — current project status
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
