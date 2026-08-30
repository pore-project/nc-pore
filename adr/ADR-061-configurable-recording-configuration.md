# ADR-061 Configurable Recording Configuration

* Status: Accepted
* Date: 2026-08-15
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe soll auf unterschiedlichen Plattformen und mit unterschiedlicher Audio-Hardware eingesetzt werden können.

Die technische Capture-Implementierung darf daher nicht von den Eigenschaften eines einzelnen Entwicklungs- oder Testgeräts ausgehen.

Gleichzeitig benötigt NC-PoRe ein definiertes Aufnahmeprofil, damit ein Benutzer nicht zunächst alle technischen Parameter selbst bestimmen muss.

ADR-002 definiert hierfür ein bevorzugtes Audioformat und Aufnahmeprofil.

Damit müssen drei Anforderungen miteinander verbunden werden:

- NC-PoRe soll ein sinnvolles Standardprofil vorschlagen.
- Der Benutzer soll die Aufnahme grundsätzlich selbst konfigurieren können.
- Das vorgeschlagene Profil darf nicht mit den tatsächlich verfügbaren technischen Fähigkeiten der Capture-Quelle verwechselt werden.

Das vorgeschlagene Profil und die tatsächlich verfügbaren technischen Möglichkeiten eines Audio-Backends sind unterschiedliche Sachverhalte.

Für die lokale Aufnahme und Wiederherstellung ist außerdem die Länge der persistenten Audio-Chunks relevant. Sie soll die gewünschte Balance zwischen häufigem Persistieren und der Menge bereits aufgenommener Zeit bestimmen, die bei einem unerwarteten Ausfall potenziell verloren gehen kann.

Diese Chunk-Dauer ist eine Recording-Konfiguration und keine Eigenschaft des konkreten Audio-Backends.

---

# Entscheidung

Die technische Recording-Konfiguration wird als konfigurierbarer Bestandteil der Capture-Schicht behandelt.

ADR-002 definiert ein **bevorzugtes Standardprofil** für NC-PoRe.

Dieses Standardprofil wird einem Benutzer später als Voreinstellung vorgeschlagen. Es ist ein Qualitätsziel und **keine technische Mindestanforderung an jedes Audio-Gerät**.

Der Benutzer soll die Aufnahmeparameter grundsätzlich selbst konfigurieren können, sofern die gewünschte Konfiguration von der verwendeten Plattform und dem verfügbaren Audio-Backend unterstützt wird.

Damit werden drei Ebenen unterschieden:

```text
NC-PoRe Default

= bevorzugtes Aufnahmeprofil
  gemäß ADR-002


User Configuration

= vom Benutzer gewünschte Aufnahmeparameter


Audio Device / Backend

= tatsächlich verfügbare technische Möglichkeiten
```

Die konkrete Capture-Implementierung muss die gewünschte Konfiguration mit den tatsächlich verfügbaren technischen Möglichkeiten abgleichen.

### Native Source Capability

Wenn die bevorzugte oder vom Benutzer gewünschte Konfiguration von der konkreten Capture-Quelle nicht vollständig unterstützt wird, soll PoRE die **bestmögliche tatsächlich verfügbare native Quellqualität** verwenden, sofern diese mit dem Recording-Lifecycle vereinbar ist.

Das bedeutet insbesondere:

- 48 kHz / 24 bit ist ein bevorzugtes Qualitätsziel, kein Zwang.
- Unterstützt eine Quelle nur 48 kHz / 16 bit, wird diese native Qualität verwendet.
- Unterstützt eine Quelle nur 44,1 kHz / 16 bit, darf diese Quelle ebenfalls aufgezeichnet werden, sofern sie die bestmögliche verfügbare native Konfiguration darstellt.
- Eine Quelle darf nicht durch bloßes Hochskalieren als höherwertig dargestellt werden.
- Eine nachträgliche Konvertierung von 16 bit nach 24 bit erzeugt keine zusätzlichen Quellinformationen.
- Eine nachträgliche Konvertierung von 44,1 kHz nach 48 kHz erzeugt ebenfalls keine zusätzliche Quellinformation.

Die Capture-Schicht darf eine technisch nicht vorhandene Qualität daher nicht vortäuschen. Die tatsächliche Quelle ist maßgeblich.

Die genaue Priorisierung einzelner Fähigkeiten – beispielsweise Samplingrate gegenüber Kanalzahl oder Sample-Repräsentation – wird durch die konkrete Capture-Implementierung und das jeweils geltende Audio-Qualitätsprofil festgelegt. Diese ADR schreibt keine universelle Rangfolge für alle denkbaren Geräte vor.

### Chunk-Dauer

Die Recording-Konfiguration enthält zusätzlich eine explizite Chunk-Dauer.

Die Chunk-Dauer wird als kontrollierte Auswahl definierter Zeitwerte modelliert und nicht als beliebiger numerischer Wert. Dadurch kann die spätere Benutzeroberfläche sinnvolle und technisch beherrschbare Auswahlmöglichkeiten anbieten, ohne das zugrunde liegende Modell auf diese Auswahl zu begrenzen.

Die zunächst unterstützten Werte sind:

- 10 Sekunden
- 30 Sekunden
- 1 Minute
- 2 Minuten
- 5 Minuten
- 10 Minuten

Der Standardwert beträgt 1 Minute.

Die Auswahl ist bewusst erweiterbar. Die konkrete Liste ist keine unveränderliche Architekturgrenze.

Die konfigurierte Chunk-Dauer beschreibt die beabsichtigte maximale Länge eines regulären persistenten Chunks. Sie dient zugleich als Richtwert für die potenziell bei einem unerwarteten Ausfall verlorene bereits aufgenommene Zeit. Die tatsächliche Verlustspanne hängt vom konkreten Persistierungszeitpunkt und vom Zeitpunkt des Ausfalls ab.

Der letzte Chunk einer regulär beendeten Aufnahme darf kürzer als die konfigurierte Dauer sein.

Die Chunk-Dauer ist unabhängig von der technischen Positionierung eines Chunks im Audiostrom. Ein `sample_offset` beschreibt die Position des Chunks im Audiostrom und ist nicht von der gewählten Chunk-Dauer abgeleitet.

Die konkrete Chunk-Erzeugung, Persistierung und Wiederherstellung wird durch diese ADR noch nicht implementiert. Die Konfiguration schafft jedoch die architektonische Grenze, an der diese später ansetzen kann.

---

# Konsequenzen

## Positive Konsequenzen

- Die Implementierung wird nicht auf die Eigenschaften eines einzelnen Entwicklungsgeräts zugeschnitten.
- Ein sinnvoller Standard kann Benutzern vorgeschlagen werden.
- Benutzer können die technischen Aufnahmeparameter an ihre eigene Umgebung anpassen.
- Ein Gerät muss nicht die bevorzugte Konfiguration vollständig unterstützen, damit eine Aufnahme möglich ist.
- Die bestmögliche native Qualität einer schwächeren Quelle kann erhalten werden.
- Künstliche Qualitätssteigerungen durch reine Formatkonvertierung werden vermieden.
- Plattform- und Backend-Unterschiede bleiben Teil der technischen Capture-Schicht.
- Die in ADR-002 definierte Vorgabe bleibt als Referenzprofil erhalten.
- Die gewünschte Persistierungs- und Ausfallbalance kann später über die Recording-Konfiguration beeinflusst werden.
- Die Chunk-Dauer ist nicht im Capture-Code als fester Wert verdrahtet.
- Das Modell kann später um zusätzliche sinnvolle Zeitwerte erweitert werden, ohne die Konfigurationsgrenze neu zu entwerfen.

## Negative Konsequenzen

- Die Capture-Schicht benötigt eine explizite Konfigurationsrepräsentation.
- Die technische Verfügbarkeit gewünschter Parameter muss geprüft werden.
- Die Auswahl der bestmöglichen nativen Konfiguration kann je nach Backend unterschiedliche Implementierungslogik erfordern.
- Die Behandlung nicht unterstützter Konfigurationen muss technisch definiert werden.
- Die konkrete Persistierungsstrategie muss die konfigurierte Chunk-Dauer berücksichtigen.

---

# Abgrenzung

Diese ADR entscheidet nicht über:

- konkrete Benutzeroberflächen für die Konfiguration
- konkrete Konfigurationsdateien
- Persistenz der Benutzerpräferenzen
- konkrete Audio-Backends
- eine universelle Rangfolge von Samplingrate, Kanalzahl und Sample-Repräsentation für alle Geräte
- konkrete Fallback-Strategien innerhalb eines einzelnen Backends
- Audio-Konvertierung als Produktionsschritt
- konkrete Implementierung der Chunk-Erzeugung
- konkrete Implementierung der Chunk-Persistierung
- konkrete Implementierung der Wiederherstellung
- Synchronisierung zwischen Aufnahmespuren

Diese Aspekte werden in späteren technischen Entscheidungen festgelegt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe is intended to run on different platforms and with different audio hardware.

The technical Capture implementation must therefore not be based on the properties of a single development or test device.

At the same time, NC-PoRe needs a defined recording profile so that users do not have to determine all technical parameters themselves before making a recording.

ADR-002 defines a preferred audio format and recording profile for this purpose.

Three requirements therefore have to be combined:

- NC-PoRe should suggest a sensible default profile.
- Users should in principle be able to configure the recording themselves.
- The suggested profile must not be confused with the actual technical capabilities of every capture source.

The suggested profile and the actual technical capabilities of an audio backend are different concerns.

For local recording and recovery, the duration of persistent audio chunks is also relevant. It should determine the desired balance between frequent persistence and the amount of already recorded time that may potentially be lost after an unexpected failure.

This chunk duration is a recording configuration and not a property of a concrete audio backend.

---

# Decision

The technical Recording Configuration is treated as a configurable part of the Capture layer.

ADR-002 defines a **preferred default profile** for NC-PoRe.

This default profile will later be suggested to users as a preset. It is a quality target and **not a technical minimum requirement for every audio device**.

Users shall in principle be able to configure the recording parameters themselves, provided that the requested configuration is supported by the selected platform and available audio backend.

This distinguishes three levels:

```text
NC-PoRe Default

= preferred recording profile
  according to ADR-002


User Configuration

= recording parameters requested by the user


Audio Device / Backend

= actual technical capabilities
```

The concrete Capture implementation must match the requested configuration against the actual technical capabilities available to it.

### Native Source Capability

If the preferred or user-requested configuration is not fully supported by the concrete capture source, PoRE shall use the **best actual native source quality available**, provided that it is compatible with the recording lifecycle.

In particular:

- 48 kHz / 24 bit is a preferred quality target, not a requirement.
- If a source only supports 48 kHz / 16 bit, that native quality is used.
- If a source only supports 44.1 kHz / 16 bit, that source may also be recorded when it represents the best available native configuration.
- A source must not be represented as higher quality merely through upscaling.
- Converting 16-bit samples to 24-bit does not create additional source information.
- Converting 44.1 kHz to 48 kHz likewise does not create additional source information.

The Capture layer must therefore not imply technical quality that the source does not actually provide. The real source is authoritative.

The precise prioritization of individual capabilities – for example sample rate versus channel count or sample representation – is defined by the concrete Capture implementation and the applicable audio quality profile. This ADR does not prescribe one universal ranking for every possible device.

### Chunk Duration

The Recording Configuration additionally contains an explicit chunk duration.

Chunk duration is modeled as a controlled selection of defined time values rather than an arbitrary numeric value. This allows a later user interface to offer sensible and technically manageable choices without making the underlying model depend on that particular list.

The initially supported values are:

- 10 seconds
- 30 seconds
- 1 minute
- 2 minutes
- 5 minutes
- 10 minutes

The default value is 1 minute.

The selection is intentionally extensible. The concrete list is not an immutable architectural boundary.

The configured chunk duration describes the intended maximum length of a regular persistent chunk. It also serves as a practical reference for the amount of already recorded time that may potentially be lost after an unexpected failure. The actual loss interval depends on the concrete persistence timing and the point at which the failure occurs.

The final chunk of a normally stopped recording may be shorter than the configured duration.

Chunk duration is independent of the technical position of a chunk in the audio stream. A `sample_offset` describes the chunk's position in the audio stream and is not derived from the selected chunk duration.

This ADR does not yet implement chunk creation, persistence, or recovery. The configuration establishes the architectural boundary at which those later components can attach.

---

# Consequences

## Positive Consequences

- The implementation is not tailored to the properties of a single development device.
- A sensible default can be suggested to users.
- Users can adapt technical recording parameters to their own environment.
- A device does not have to fully support the preferred configuration for recording to be possible.
- The best native quality of a lower-capability source can be preserved.
- Artificial quality increases through pure format conversion are avoided.
- Platform and backend differences remain within the technical Capture layer.
- The profile defined by ADR-002 remains available as a reference configuration.
- The desired persistence and failure-recovery balance can later be influenced through Recording Configuration.
- Chunk duration is not hard-coded in the Capture implementation.
- Additional sensible time values can be added later without redesigning the configuration boundary.

## Negative Consequences

- The Capture layer requires an explicit configuration representation.
- The technical availability of requested parameters must be checked.
- Selecting the best native configuration may require backend-specific implementation logic.
- Handling unsupported configurations must be defined technically.
- The concrete persistence strategy must honor the configured chunk duration.

---

# Scope

This ADR does not decide:

- concrete user interfaces for configuration
- concrete configuration files
- persistence of user preferences
- concrete audio backends
- a universal ranking of sample rate, channel count, and sample representation for every device
- concrete fallback strategies within an individual backend
- audio conversion as a production step
- concrete chunk creation implementation
- concrete chunk persistence implementation
- concrete recovery implementation
- synchronization between recording tracks

These aspects will be defined by later technical decisions.
