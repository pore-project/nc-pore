# NC-PoRe

## Deutsch ([English version below](#english-version))

---

# NC-PoRe

NC-PoRe ist eine selbsthostbare Open-Source-Plattform für professionelle Podcast-Aufnahmen und Produktion.

Das zentrale Prinzip:

> Meine Daten gehören mir.

NC-PoRe ermöglicht professionelle lokale Audioaufnahme und die anschließende, verifizierte Übergabe fertiger Aufnahmen an die eigene Nextcloud-Infrastruktur.

## Aktueller V1-Stand

NC-PoRe befindet sich in der **V1-Härtungs- und Produktisierungsphase**.

Für V1 ist Nextcloud der Host und die autoritative Speicherinstanz. Der aktuelle technische Pfad ist bewusst einfach:

```text
Nextcloud Talk
    → NC-PoRe Recorder
    → dauerhafter Browser-Artefaktstand
    → authentifizierter Nextcloud-OCS-Endpunkt
    → Nextcloud Files API
    → Größenprüfung + SHA-256 Read-back
    → bestätigtes Artefakt in Nextcloud Files
```

Der Browser hält die fertiggestellte WAV-Datei bereits dauerhaft vor, bevor die Übergabe an Nextcloud beginnt. Dadurch benötigt V1 keine zweite serverseitige Artefaktablage und keinen separaten PoRe-Transferdienst.

### Speicherpfad

Der Speicherort wird als **logischer, relativ zur aktuellen Benutzer-Files-Root angegebener PoRe-Stamm** konfiguriert.

Beispiel:

`Büro/interviews`

Daraus entsteht direkt:

`Büro/interviews/YYYY/MM/DD - HH:MIN <production label> - <core.ProductionId>/<captureId>.wav`

Wenn kein eigener Stamm konfiguriert ist, verwendet PoRe `audio`:

`audio/YYYY/MM/DD - HH:MIN <production label> - <core.ProductionId>/<captureId>.wav`

Ein physischer Nextcloud-Datenpfad wie `/var/www/nextcloud/data/...` ist **niemals Bestandteil der Schnittstelle** und wird von NC-PoRe nicht verwendet.

### Integritätsbestätigung

Eine erfolgreiche HTTP-Antwort allein gilt nicht als abgeschlossene Übergabe. V1 prüft:

1. SHA-256 des hochgeladenen temporären Artefakts gegen den vom Browser gelieferten Hash;
2. erfolgreiche Speicherung über die Nextcloud Files API;
3. gespeicherte Dateigröße gegen die fertige Payload;
4. erneutes Lesen der gespeicherten Datei über die Files API;
5. erneute SHA-256-Berechnung und Vergleich.

Erst wenn alle Prüfungen erfolgreich sind, meldet NC-PoRe das Artefakt als gespeichert. Der Browser darf erst danach seine dauerhafte Completion-Markierung setzen.

## Nextcloud Talk

NC-PoRe integriert seine V1-Konfiguration in den bestehenden Talk-Einstellungsdialog. Der Speicherstamm wird als relativer Files-Pfad eingegeben; `audio` erscheint als grauer Standard-Platzhalter, wenn kein eigener Wert gespeichert ist.

V1 verwendet die normale Nextcloud-App-Architektur. Es gibt keine Abhängigkeit von AppAPI oder ExApp und keinen separaten PoRe-HTTP-Server.

## Projektstatus

Die technische Grundlage für lokale Aufnahme, dauerhafte Browser-Recovery, Artefaktidentität und verifizierte Nextcloud-Speicherung ist implementiert. Der nächste wesentliche Schritt ist die reale Nextcloud/Talk-Integration in einer dafür bereitgestellten Testinstanz.

Für eine öffentliche V1-Freigabe gelten insbesondere:

- Nextcloud ist die V1-Host- und Speicherinstanz.
- Nextcloud-Verbindungen verwenden ausschließlich HTTPS.
- Die fertige WAV-Datei ist vor der Host-Übergabe dauerhaft im Browser gesichert.
- Nextcloud Files ist die autoritative Speicherung des V1-Artefakts.
- Remote-Abschluss wird nur nach Größen- und SHA-256-Verifikation bestätigt.
- Provider- und Host-spezifische Details bleiben außerhalb des host-neutralen PoRe-Kerns.

Weitere Informationen:

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

# English Version

---

# NC-PoRe

NC-PoRe is a self-hostable open-source platform for professional podcast recording and production.

The central principle:

> My data belongs to me.

NC-PoRe provides professional local audio recording and verified delivery of finalized recordings into the user's own Nextcloud infrastructure.

## Current V1 status

NC-PoRe is currently in the **V1 hardening and productization phase**.

For V1, Nextcloud is the host and authoritative storage system. The technical path is intentionally simple:

```text
Nextcloud Talk
    → NC-PoRe Recorder
    → durable browser artifact
    → authenticated Nextcloud OCS endpoint
    → Nextcloud Files API
    → size check + SHA-256 read-back
    → confirmed artifact in Nextcloud Files
```

The browser already keeps the finalized WAV durably before host handoff. V1 therefore needs neither a second server-side artifact store nor a separate PoRe transfer service.

### Storage path

The storage location is configured as a **logical path relative to the current user's Nextcloud Files root**.

Example:

`Büro/interviews`

The resulting path is:

`Büro/interviews/YYYY/MM/DD - HH:MIN <production label> - <core.ProductionId>/<captureId>.wav`

If no custom root is configured, PoRe uses `audio`:

`audio/YYYY/MM/DD - HH:MIN <production label> - <core.ProductionId>/<captureId>.wav`

A physical Nextcloud data path such as `/var/www/nextcloud/data/...` is **never part of the interface** and is never used by NC-PoRe.

### Integrity confirmation

A successful HTTP response alone is not considered completion. V1 verifies:

1. SHA-256 of the uploaded temporary artifact against the browser-provided hash;
2. successful storage through the Nextcloud Files API;
3. stored file size against the finalized payload;
4. a read-back of the stored file through the Files API;
5. a second SHA-256 calculation and comparison.

Only after all checks succeed does NC-PoRe report the artifact as stored. The browser marks durable completion only afterwards.

## Nextcloud Talk

NC-PoRe integrates its V1 configuration into the existing Talk settings dialog. The storage root is entered as a relative Files path; `audio` is shown as the grey default placeholder when no custom value is saved.

V1 uses the normal Nextcloud app architecture. There is no AppAPI or ExApp dependency and no separate PoRe HTTP server.

## Project status

The technical foundation for local recording, durable browser recovery, artifact identity and verified Nextcloud storage is implemented. The next major step is real Nextcloud/Talk integration testing in a provisioned test instance.

For a public V1 release, the following principles apply in particular:

- Nextcloud is the V1 host and storage system.
- Nextcloud connections use HTTPS only.
- The finalized WAV is durably preserved in the browser before host handoff.
- Nextcloud Files is the authoritative V1 artifact store.
- Remote completion is confirmed only after size and SHA-256 verification.
- Provider- and host-specific details remain outside the host-neutral PoRe core.

Further information:

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
