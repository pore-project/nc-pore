# ADR-061 Configurable Recording Configuration

* Status: Accepted
* Date: 2026-08-15
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

---

# Kontext

NC-PoRe soll auf unterschiedlichen Plattformen und mit
unterschiedlicher Audio-Hardware eingesetzt werden können.

Die technische Capture-Implementierung darf daher nicht
von den Eigenschaften eines einzelnen Entwicklungs- oder
Testgeräts ausgehen.

Gleichzeitig benötigt NC-PoRe ein definiertes
Aufnahmeprofil, damit ein Benutzer nicht zunächst alle
technischen Parameter selbst bestimmen muss.

ADR-002 definiert hierfür ein bevorzugtes Audioformat und
Aufnahmeprofil.

Damit müssen zwei Anforderungen miteinander verbunden
werden:

- NC-PoRe soll ein sinnvolles Standardprofil vorschlagen.
- Der Benutzer soll die Aufnahme grundsätzlich selbst
  konfigurieren können.

Das vorgeschlagene Profil und die tatsächlich verfügbaren
technischen Möglichkeiten eines Audio-Backends sind dabei
unterschiedliche Sachverhalte.

---

# Entscheidung

Die technische Recording-Konfiguration wird als
konfigurierbarer Bestandteil der Capture-Schicht behandelt.

ADR-002 definiert ein bevorzugtes Standardprofil für
NC-PoRe.

Dieses Standardprofil wird einem Benutzer später als
Voreinstellung vorgeschlagen, ist jedoch kein technischer
Zwang.

Der Benutzer soll die Aufnahmeparameter grundsätzlich
selbst konfigurieren können, sofern die gewünschte
Konfiguration von der verwendeten Plattform und dem
verfügbaren Audio-Backend unterstützt wird.

Damit werden drei Ebenen unterschieden:

```text
NC-PoRe Default

= vorgeschlagenes Aufnahmeprofil
  gemäß ADR-002


User Configuration

= vom Benutzer gewünschte Aufnahmeparameter


Audio Device / Backend

= tatsächlich verfügbare technische Möglichkeiten
```

Die konkrete Capture-Implementierung muss die gewünschte
Konfiguration mit den tatsächlich verfügbaren technischen
Möglichkeiten abgleichen.

Diese ADR legt noch nicht fest, wie ein nicht unterstütztes
Profil behandelt wird. Insbesondere wird hier nicht
entschieden, ob die Aufnahme abgelehnt, eine alternative
Konfiguration angeboten oder eine technische Konvertierung
verwendet wird.

Diese Entscheidungen gehören zur späteren konkreten
Recording-Implementierung.

---

# Konsequenzen

## Positive Konsequenzen

- Die Implementierung wird nicht auf die Eigenschaften
  eines einzelnen Entwicklungsgeräts zugeschnitten.
- Ein sinnvoller Standard kann Benutzern vorgeschlagen
  werden.
- Benutzer können die technischen Aufnahmeparameter an
  ihre eigene Umgebung anpassen.
- Plattform- und Backend-Unterschiede bleiben Teil der
  technischen Capture-Schicht.
- Die in ADR-002 definierte Vorgabe bleibt als
  Referenzprofil erhalten.

## Negative Konsequenzen

- Die Capture-Schicht benötigt eine explizite
  Konfigurationsrepräsentation.
- Die technische Verfügbarkeit gewünschter Parameter muss
  geprüft werden.
- Die Behandlung nicht unterstützter Konfigurationen muss
  später definiert werden.

---

# Abgrenzung

Diese ADR entscheidet nicht über:

- konkrete Benutzeroberflächen für die Konfiguration
- konkrete Konfigurationsdateien
- Persistenz der Benutzerpräferenzen
- konkrete Audio-Backends
- konkrete Fallback-Strategien
- Audio-Konvertierung
- konkrete Default-Werte außerhalb der Vorgaben aus ADR-002

Diese Aspekte werden in späteren technischen Entscheidungen
festgelegt.

---

# English Version ([Deutsche Version oben](#deutsch))

---

# Context

NC-PoRe is intended to run on different platforms and with
different audio hardware.

The technical Capture implementation must therefore not be
based on the properties of a single development or test
device.

At the same time, NC-PoRe needs a defined recording profile
so that users do not have to determine all technical
parameters themselves before making a recording.

ADR-002 defines a preferred audio format and recording
profile for this purpose.

Two requirements therefore have to be combined:

- NC-PoRe should suggest a sensible default profile.
- Users should in principle be able to configure the
  recording themselves.

The suggested profile and the actual technical capabilities
of an audio backend are different concerns.

---

# Decision

The technical Recording Configuration is treated as a
configurable part of the Capture layer.

ADR-002 defines a preferred default profile for NC-PoRe.

This default profile will later be suggested to users as a
preset, but it is not a technical requirement.

Users shall in principle be able to configure the recording
parameters themselves, provided that the requested
configuration is supported by the selected platform and
available audio backend.

This distinguishes three levels:

```text
NC-PoRe Default

= suggested recording profile
  according to ADR-002


User Configuration

= recording parameters requested by the user


Audio Device / Backend

= actual technical capabilities
```

The concrete Capture implementation must match the requested
configuration against the actual technical capabilities
available to it.

This ADR does not yet define how an unsupported profile is
handled. In particular, it does not decide whether recording
is rejected, an alternative configuration is offered, or
technical conversion is used.

These decisions belong to the later concrete Recording
implementation.

---

# Consequences

## Positive Consequences

- The implementation is not tailored to the properties of a
  single development device.
- A sensible default can be suggested to users.
- Users can adapt technical recording parameters to their own
  environment.
- Platform and backend differences remain within the
  technical Capture layer.
- The profile defined by ADR-002 remains available as a
  reference configuration.

## Negative Consequences

- The Capture layer requires an explicit configuration
  representation.
- The technical availability of requested parameters must be
  checked.
- Handling unsupported configurations must be defined later.

---

# Scope

This ADR does not decide:

- concrete user interfaces for configuration
- concrete configuration files
- persistence of user preferences
- concrete audio backends
- concrete fallback strategies
- audio conversion
- concrete default values beyond the profile defined by
  ADR-002

These aspects will be defined by later technical decisions.
