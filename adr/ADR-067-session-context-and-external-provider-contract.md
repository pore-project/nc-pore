# ADR-067: Session Context and External Provider Contract

* Status: Proposed
* Date: 2026-08-20
* Decision Type: Architecture

---

# Deutsch ([English version below](#english-version))

## Kontext

NC-PoRe benötigt für eine vollständige Aufnahme einen definierten technischen Kontext: Die Session muss verfügbar sein, Teilnehmer müssen identifizierbar und für die erforderlichen Operationen autorisierbar sein, und die für die Aufnahme benötigten Session- und Teilnahmeinformationen müssen zuverlässig verfügbar sein.

Nextcloud Talk liefert bereits einen großen Teil dieses Kontexts. Talk besitzt eigene Semantik für Conversations, Identitäten, Gäste, registrierte Benutzer, Federation, Zugriffsregeln und den Lebenszyklus von Conversations. NC-PoRe soll diese Semantik nicht nachbauen oder in sein Domain-Modell kopieren.

Gleichzeitig darf die Architektur nicht voraussetzen, dass jede Integrationsumgebung dieselben Fähigkeiten besitzt.

---

# Entscheidung

NC-PoRe definiert einen **Session Context Contract** als klaren Ein-/Ausstiegspunkt zwischen der PoRE-Anwendungslogik und einer externen Integrationsumgebung.

PoRE definiert dabei **welche Informationen und Fähigkeiten für eine konkrete Operation benötigt werden**. Die jeweilige Integration ist dafür verantwortlich, diesen Context bereitzustellen.

Der Context kann vollständig aus einer externen Umgebung stammen, aus PoRE selbst stammen oder aus mehreren Quellen zusammengesetzt werden.

```text
                         NC-PoRe
                            |
                 Session Context Contract
                            |
          +-----------------+-----------------+
          |                 |                 |
     External Host      PoRE-native      Other Integration
          |                 |                 |
     host APIs         PoRE state       own context
          |                 |                 |
          +-----------------+-----------------+
                            |
                    required PoRE context
```

Der Core und die fachliche `ProductionSession` kennen keine provider-spezifischen Session- oder Teilnehmermodelle.

---

# Der Contract

Der konkrete technische Contract wird durch eine dafür zuständige Implementierungsentscheidung präzisiert. Auf Architekturebene umfasst er mindestens:

* **Session Identity** — welche Session bzw. welcher externe Kontext gemeint ist
* **Session Availability** — ob der Kontext für die angeforderte Operation verfügbar ist
* **Participant Identity** — wer an der Session teilnimmt bzw. identifizierbar ist
* **Participation / Authorization Context** — welche Teilnahme- und Berechtigungsinformationen für die Operation erforderlich sind
* **Provider Capabilities** — welche für die konkrete Operation benötigten Fähigkeiten die Integration bereitstellt

Nicht jede Integration muss alle Informationen aus derselben Quelle liefern.

Eine Integration darf zusätzliche Informationen und Fähigkeiten besitzen, ohne dass diese Teil des universellen PoRE-Core-Modells werden.

---

# Provider ist nicht gleich Session Owner

Eine externe Integration muss nicht die fachliche PoRE-Session besitzen.

Eine PoRE-Session kann ohne externen Session Provider existieren. Ebenso kann eine Integration einen externen Session Context an eine PoRE-Session binden.

Beispiel externe Integration:

```text
ProductionSession
       |
       +-- Session Context
              |
              +-- provider = external host
              +-- external identity = host context
              +-- availability = derived from host
              +-- participation = derived from host
```

Beispiel PoRE-native:

```text
ProductionSession
       |
       +-- Session Context
              |
              +-- provider = PoRE
              +-- identity/lifecycle = PoRE
              +-- participation = PoRE
```

Ein Storage Provider ist daher nicht automatisch ein Session Provider.

---

# Session Lifecycle und Availability

Der fachliche PoRE-Lifecycle und die externe Session-Verfügbarkeit sind getrennte Zustandsdimensionen.

Beispielsweise kann eine PoRE-Session `Completed` sein, obwohl der zugehörige externe Kontext noch existiert. Umgekehrt kann eine PoRE-Session noch aktiv sein, obwohl der externe Kontext gelöscht oder anderweitig nicht mehr verfügbar ist.

Daraus folgt:

```text
PoRE Session State
        AND
Provider Session Availability
        => operational usability
```

Das Löschen oder Ablaufdatum eines externen Kontexts wird nicht automatisch zu einem fachlichen PoRE-Status wie `Completed`.

Die Integration muss stattdessen den Verlust der externen Verfügbarkeit über den Session Context Contract ausdrücken. Die Application Layer entscheidet anschließend, welche PoRE-Operationen noch zulässig sind und welche Reaktion erforderlich ist.

`SessionAvailability` ist daher konzeptionell mehr als ein einfacher Boolean. Der konkrete Statusraum wird nur insoweit abstrahiert, wie PoRE ihn für seine eigenen Operationen benötigt.

---

# Participation

Teilnahme ist ein Bestandteil des Session Context, aber externe Teilnehmermodelle werden nicht in das PoRE-Core-Modell kopiert.

Eine Host-Integration darf beispielsweise unterschiedliche externe Teilnehmerarten unterscheiden. PoRE übernimmt daraus nur die Informationen, die für sein eigenes fachliches `Participation`- und Rollenmodell relevant sind.

```text
External participant identity/type
                |
                v
       Session Context
                |
                v
       PoRE Participation
                |
                v
       PoRE ParticipantRole
```

Provider-spezifische Teilnehmerarten bleiben provider-spezifisch.

---

# Capabilities

Capabilities werden nicht als universelles Abbild eines Providers verstanden.

Eine Integration kann wesentlich mehr können als PoRE benötigt. PoRE fragt nur die für die jeweilige Operation relevanten Fähigkeiten ab.

Provider-spezifische Meeting-, Gast-, Federation- oder andere Funktionen werden nicht automatisch zu Core-Abstraktionen.

Eine Integration darf fehlende Fähigkeiten intern durch eigene Komponenten ergänzen, solange die für PoRE erforderlichen Fähigkeiten über den Session Context Contract bereitgestellt werden.

---

# Verantwortungsgrenze

Die Architektur folgt damit dem Prinzip:

> **PoRE definiert, was es zum Arbeiten wissen und können muss. Die Integration entscheidet, woher und wie dieser Context bereitgestellt wird.**

Die Integration ist dabei eine Adapter- und Context-Grenze, keine neue Domain-Autorität.

---

# Konsequenzen

## Positive Auswirkungen

* bestehende Host-Semantik kann genutzt werden, ohne dass PoRE sie nachbauen muss
* der PoRE-Core bleibt unabhängig von konkreten Integrationsumgebungen
* eine PoRE-native Session bleibt möglich
* Session-/Teilnahme-Kontext und Storage bleiben getrennte Architekturachsen
* provider-spezifische Features müssen nicht in das universelle Domain-Modell übernommen werden
* die Aufnahme- und Application-Logik kann gegen einen stabilen PoRE-Contract arbeiten

## Negative Auswirkungen

* der Session Context Contract muss sorgfältig definiert werden
* Provider-Capabilities müssen explizit behandelt werden, wenn eine Operation nicht verfügbar ist
* die Übersetzung externer Identitäten in PoRE-Teilnehmeridentitäten benötigt klare Semantik
* die Reaktion auf verlorene externe Verfügbarkeit muss auf Application-Ebene definiert werden

Diese Nachteile werden bewusst akzeptiert.

---

# Betrachtete Alternativen

## Eine konkrete Host-Anwendung als Session-Modell für PoRE

Verworfen. Dadurch würde PoRE das Modell eines einzelnen Providers übernehmen und Integrationen unnötig erschweren.

## Universelles PoRE-Sessionmodell als vollständiger Ersatz für externe Session-Kontexte

Verworfen. Damit würde PoRE Funktionen nachbauen, die Integrationsumgebungen bereits bereitstellen können.

## Storage Provider und Session Provider als eine einzige Abstraktion

Verworfen. Datenspeicherung und Session-/Teilnahmekontext sind unterschiedliche Verantwortlichkeiten und können unabhängig voneinander implementiert werden.

## Einfache `is_valid()`-Abfrage für externe Sessions

Verworfen. Externe Verfügbarkeit kann mehrere relevante Zustände besitzen. PoRE benötigt eine abstrahierte Availability-Semantik, nicht bloß einen booleschen Wert.

---

# Beziehung zu bestehender Architektur

Diese Entscheidung ergänzt ADR-022 und ADR-026 durch eine explizite Provider-Grenze für Session- und Teilnahme-Kontext.

Sie baut auf ADR-031 für Identität und Rollen sowie auf ADR-062 für browserbasierte Gastteilnahme auf.

Sie ergänzt ADR-065: Storage Provider und Session Context sind bewusst getrennte Integrationsachsen.

Die bestehende `ProductionSession` bleibt eine fachliche Core-Struktur und wird nicht mit provider-spezifischen Sessiondaten angereichert.

---

# Nicht durch diese ADR festgelegt

Diese ADR legt insbesondere nicht fest:

* welche Context-Daten in einer konkreten Implementierung verpflichtend oder optional sind
* wie externe Identitäten technisch repräsentiert werden
* welche Availability-Zustände konkret verwendet werden
* wie Capability-Abfragen technisch aussehen
* wie Session-Erzeugung und Session-Bindung technisch funktionieren
* wie der Verlust eines externen Session Context behandelt wird

Diese Details werden durch konkrete technische Entscheidungen festgelegt, sobald sie für eine Implementierung erforderlich sind.

---

# English Version ([Deutsche Version oben](#deutsch))

## Context

NC-PoRe requires a defined technical context for a complete recording: the session must be available, participants must be identifiable and authorized for required operations, and the session and participation information needed for recording must be reliably available.

Nextcloud Talk already provides much of this context. Talk has its own semantics for conversations, identities, guests, registered users, federation, access rules and conversation lifecycle. NC-PoRe must not reproduce or copy that semantics into its domain model.

At the same time, the architecture must not assume that every integration environment provides the same capabilities.

---

# Decision

NC-PoRe defines a **Session Context Contract** as a clear boundary between PoRE application logic and an external integration environment.

PoRE defines **which information and capabilities are required for a concrete operation**. The respective integration is responsible for providing that context.

The context may come entirely from an external environment, from PoRE itself, or from multiple sources.

```text
                         NC-PoRe
                            |
                 Session Context Contract
                            |
          +-----------------+-----------------+
          |                 |                 |
     External Host      PoRE-native      Other Integration
          |                 |                 |
     host APIs         PoRE state       own context
          |                 |                 |
          +-----------------+-----------------+
                            |
                    required PoRE context
```

The Core and the domain `ProductionSession` know no provider-specific session or participant models.

---

# The Contract

The concrete technical contract is refined by a dedicated implementation decision. At architecture level it includes at least:

* **Session Identity** — which session or external context is meant
* **Session Availability** — whether the context is available for the requested operation
* **Participant Identity** — who participates in the session or can be identified
* **Participation / Authorization Context** — which participation and authorization information is required for the operation
* **Provider Capabilities** — which capabilities required for the operation are provided by the integration

Not every integration has to provide all information from the same source.

An integration may have additional information and capabilities without making them part of the universal PoRE Core model.

---

# Provider Is Not the Same as Session Owner

An external integration does not have to own the domain PoRE session.

A PoRE session may exist without an external session provider. Conversely, an integration may bind an external session context to a PoRE session.

Example external integration:

```text
ProductionSession
       |
       +-- Session Context
              |
              +-- provider = external host
              +-- external identity = host context
              +-- availability = derived from host
              +-- participation = derived from host
```

Example PoRE-native:

```text
ProductionSession
       |
       +-- Session Context
              |
              +-- provider = PoRE
              +-- identity/lifecycle = PoRE
              +-- participation = PoRE
```

A storage provider is therefore not automatically a session provider.

---

# Session Lifecycle and Availability

The domain PoRE lifecycle and external session availability are separate state dimensions.

For example, a PoRE session may be `Completed` while the associated external context still exists. Conversely, a PoRE session may still be active while the external context has been deleted or is otherwise unavailable.

Therefore:

```text
PoRE Session State
        AND
Provider Session Availability
        => operational usability
```

Deletion or expiry of an external context does not automatically become a domain PoRE state such as `Completed`.

The integration must express loss of external availability through the Session Context Contract. The Application Layer then decides which PoRE operations remain permitted and what response is required.

`SessionAvailability` is therefore conceptually more than a boolean. The concrete state space is abstracted only to the extent required by PoRE's own operations.

---

# Participation

Participation is part of the Session Context, but external participant models are not copied into the PoRE Core model.

A host integration may distinguish different external participant types. PoRE adopts only the information relevant to its own domain `Participation` and role model.

```text
External participant identity/type
                |
                v
       Session Context
                |
                v
       PoRE Participation
                |
                v
       PoRE ParticipantRole
```

Provider-specific participant types remain provider-specific.

---

# Capabilities

Capabilities are not treated as a universal mirror of a provider.

An integration may support much more than PoRE requires. PoRE queries only the capabilities relevant to the respective operation.

Provider-specific meeting, guest, federation or other functions do not automatically become Core abstractions.

An integration may provide missing capabilities through its own components, as long as the capabilities required by PoRE are exposed through the Session Context Contract.

---

# Responsibility Boundary

The architecture therefore follows this principle:

> **PoRE defines what it needs to know and be able to do. The integration decides where and how that context is provided.**

The integration is an adapter and context boundary, not a new domain authority.

---

# Consequences

## Positive Effects

* existing host semantics can be used without reproducing them in PoRE
* PoRE Core remains independent from concrete integration environments
* a PoRE-native session remains possible
* session/participation context and storage remain separate architectural axes
* provider-specific features do not have to become part of the universal domain model
* recording and application logic can operate against a stable PoRE contract

## Negative Effects

* the Session Context Contract must be defined carefully
* provider capabilities must be handled explicitly when an operation is unavailable
* translation of external identities into PoRE participant identities requires clear semantics
* the response to lost external availability must be defined at application level

These disadvantages are consciously accepted.

---

# Alternatives Considered

## A Concrete Host Application as the PoRE Session Model

Rejected. This would make PoRE adopt the model of one provider and unnecessarily complicate integrations.

## Universal PoRE Session Model as a Complete Replacement for External Session Contexts

Rejected. This would require PoRE to reproduce functions that integration environments may already provide.

## Storage Provider and Session Provider as One Abstraction

Rejected. Storage and session/participation context are different responsibilities and can be implemented independently.

## Simple `is_valid()` Query for External Sessions

Rejected. External availability can have several relevant states. PoRE requires an abstract availability semantic rather than a boolean only.

---

# Relationship to Existing Architecture

This decision complements ADR-022 and ADR-026 with an explicit provider boundary for session and participation context.

It builds on ADR-031 for identity and roles and ADR-062 for browser-based guest participation.

It complements ADR-065: Storage Provider and Session Context are deliberately separate integration axes.

The existing `ProductionSession` remains a domain Core structure and is not enriched with provider-specific session data.

---

# Not Defined by This ADR

This ADR does not define in particular:

* which context data is mandatory or optional in a concrete implementation
* how external identities are represented technically
* which availability states are used concretely
* how capability queries are implemented technically
* how session creation and session binding are implemented technically
* how loss of an external Session Context is handled

These details are defined by concrete technical decisions when required for an implementation.
