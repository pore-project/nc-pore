# Deutsch ([English version below](#english-version))

# ADR-036: Browser-First Guest Participation

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Kontext

NC-PoRe muss zuverlässige lokale Aufnahmen ermöglichen und gleichzeitig die Teilnahme externer Gäste so einfach wie möglich halten.

Separate Clients für Windows, macOS, Linux, iOS und Android würden insbesondere für gelegentliche Teilnehmer unnötige Installations- und Wartungsbarrieren schaffen.

Die Architektur von Ennuicastr zeigt, dass browserbasierte Teilnahme einen wesentlichen Teil des Remote-Recording-Anwendungsfalls abdecken kann, ohne einen plattformspezifischen Gast-Client vorauszusetzen.

Gleichzeitig können professionelle Aufnahmeszenarien Fähigkeiten erfordern, die Browser nicht auf allen Plattformen zuverlässig bereitstellen können.

---

# Entscheidung

NC-PoRe verfolgt ein **Browser-First-Modell für die Teilnahme**.

Externe Gäste sollen über einen unterstützten modernen Browser an einer Session teilnehmen und aufnehmen können, ohne eine dedizierte NC-PoRe-Anwendung installieren zu müssen.

Ein nativer oder spezialisierter Recorder kann für professionelle Workflows vorgesehen werden, wenn Browser-Fähigkeiten nicht ausreichen. Er ist jedoch keine Voraussetzung für die gewöhnliche Gastteilnahme.

Der Browser-Client ist für Capture und Session-Interaktion verantwortlich. Er definiert weder das kanonische Produktionsformat noch das Storage-Format.

---

# Architekturprinzip

> Teilnahme soll überall dort, wo es technisch sinnvoll möglich ist, einen Browser und keinen plattformspezifischen NC-PoRe-Client voraussetzen.

Server und Domain-Modell müssen unabhängig von einer konkreten Browser-Implementierung bleiben.

---

# Abgrenzung

Browser-First bedeutet nicht Browser-Only.

Professionelle Clients können später zusätzliche Fähigkeiten bereitstellen, beispielsweise:

* erweiterte Hardwarekontrolle
* spezialisiertes Monitoring
* Offline-Workflows
* professionelle Audio-Interfaces

Solche Clients sind optionale Erweiterungen und dürfen nicht zur Voraussetzung für die Gastteilnahme werden.

---

# Konsequenzen

## Positive Auswirkungen

* sehr geringe Einstiegshürde für Gäste
* keine plattformspezifische Verteilung eines Gast-Clients erforderlich
* einfacheres Onboarding und einfachere Aktualisierung
* breite Geräteabdeckung durch etablierte Browser-Plattformen
* klare Trennung zwischen Capture-Client und Produktionsmodell

---

## Negative Auswirkungen

* Browserverhalten, Berechtigungen und Scheduling müssen als explizite technische Randbedingungen behandelt werden
* Hintergrundausführung, Gerätewechsel, Buffer-Verhalten und Browser-Lifecycle benötigen gezielte Tests
* einzelne professionelle Aufnahmefunktionen können einen spezialisierten Client erfordern

---

# Betrachtete Alternativen

## Plattformspezifische Gast-Clients

Als Standard verworfen, da Installation und Wartung gelegentliche Teilnehmer unnötig belasten würden.

---

## Browser-Only für alle Aufnahmeszenarien

Als absolute Vorgabe verworfen, da professionelle Audio-Workflows Fähigkeiten benötigen können, die Browser nicht zuverlässig garantieren können.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung **konkretisiert ADR-008**. Die dort festgelegte Unterscheidung zwischen professioneller Aufnahme und einfacher Gastteilnahme bleibt bestehen. Der Gastpfad wird nun jedoch ausdrücklich als Browser-First definiert und nicht mehr lediglich als mögliche zukünftige Option.

ADR-026 bleibt maßgeblich für die Trennung zwischen Domain-Modell und Storage-Providern.

---

# Zukünftige Betrachtungen

Die unterstützten Browser-Fähigkeiten und Mindestanforderungen müssen vor dem produktiven Einsatz definiert werden.

Das Browser-Capture-Modell muss gemeinsam mit den Entscheidungen zu aktiver Synchronisation und rekonstruierbaren Capture-Artefakten bewertet werden.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-036: Browser-First Guest Participation

## Status

Proposed

## Date

2026-08-17

## Decision Type

Architecture

---

# Context

NC-PoRe must provide reliable local recording while keeping external guest participation as simple as possible.

Separate clients for Windows, macOS, Linux, iOS and Android would create unnecessary installation and maintenance barriers, especially for occasional participants.

The Ennuicastr architecture demonstrates that browser-based participation can cover a substantial part of the remote recording use case without requiring a platform-specific guest client.

At the same time, professional recording scenarios may require capabilities that browsers cannot reliably provide on every platform.

---

# Decision

NC-PoRe adopts a **browser-first participation model**.

External guests should be able to join and record through a supported modern browser without installing a dedicated NC-PoRe application.

A native or specialized recorder may be provided for professional workflows where browser capabilities are insufficient, but it is not a prerequisite for ordinary guest participation.

The browser client is responsible for capture and session interaction. It does not define the canonical production format or storage format.

---

# Architectural Principle

> Participation should require a browser, not a platform-specific NC-PoRe client, wherever technically feasible.

The server and domain model must remain independent of a specific browser implementation.

---

# Scope Boundary

Browser-first does not mean browser-only.

Professional clients may later provide additional capabilities such as:

* advanced hardware control
* specialist monitoring
* offline workflows
* professional audio interfaces

Such clients are optional extensions and must not become a dependency for guest participation.

---

# Consequences

## Positive Effects

* very low entry barrier for guests
* no platform-specific guest-client distribution requirement
* simpler onboarding and updates
* broad device coverage through established browser platforms
* clear separation between capture client and production model

---

## Negative Effects

* browser behavior, permissions and scheduling must be treated as explicit technical constraints
* background execution, device changes, buffering and browser lifecycle require dedicated testing
* some professional recording capabilities may require a specialized client

---

# Alternatives Considered

## Platform-Specific Guest Clients

Rejected as the default approach because installation and maintenance would unnecessarily burden occasional participants.

---

## Browser-Only for All Recording Scenarios

Rejected as an absolute requirement because professional audio workflows may require capabilities that browsers cannot reliably guarantee.

---

# Relationship to Existing Architecture

This decision **refines ADR-008**. The existing distinction between professional recording and simple guest participation remains valid, but the guest path is now explicitly browser-first rather than merely a possible future option.

ADR-026 remains authoritative for the separation between the domain model and storage providers.

---

# Future Considerations

The supported browser capabilities and minimum requirements must be defined before production use.

The browser capture model must be evaluated together with the decisions on active synchronization and reconstructable capture artifacts.
