# Deutsch ([English version below](#english-version))

# ADR-067: Session Context and External Provider Contract

## Status

Proposed

## Date

2026-08-20

## Decision Type

Architecture

---

# Kontext

NC-PoRe benötigt für eine vollständige Aufnahme einen definierten technischen Kontext: Die Session muss verfügbar sein, Teilnehmer müssen identifizierbar und für die erforderlichen Operationen autorisierbar sein, und die für die Aufnahme benötigten Session- und Teilnahmeinformationen müssen zuverlässig verfügbar sein.

Nextcloud Talk liefert bereits einen großen Teil dieses Kontexts. Talk besitzt eigene Semantik für Conversations, Identitäten, Gäste, registrierte Benutzer, Federation, Zugriffsregeln und den Lebenszyklus von Conversations. NC-PoRe soll diese Semantik nicht nachbauen oder in sein Domain-Modell kopieren.

Gleichzeitig darf die Architektur nicht davon ausgehen, dass jeder zukünftige Integrationsanbieter dieselben Fähigkeiten besitzt. Eine zukünftige Integration, beispielsweise für Dropbox als Storage Provider, könnte einen eigenen Session Provider benötigen oder Teile des erforderlichen Kontexts vollständig selbst bereitstellen.

Damit besteht die Gefahr zweier falscher Extreme:

1. NC-PoRe übernimmt ein konkretes externes Session-Modell und wird davon abhängig.
2. NC-PoRe versucht, die Fähigkeiten jedes Providers selbst nachzubauen und verliert die Vorteile vorhandener Plattformen.

---

# Entscheidung

NC-PoRe definiert einen **Session Context Contract** als klaren Ein-/Ausstiegspunkt zwischen der PoRE-Anwendungslogik und einer externen Integrationsumgebung.

PoRE definiert dabei **welche Informationen und Fähigkeiten für eine konkrete Operation benötigt werden**. Die jeweilige Integration ist dafür verantwortlich, diesen Context bereitzustellen.

Der Context kann vollständig aus einem externen Provider stammen, aus PoRE selbst stammen oder aus mehreren Quellen zusammengesetzt werden.

Konzeptionell:

```text
                         NC-PoRe
                            |
                 Session Context Contract
                            |
          +-----------------+-----------------+
          |                 |                 |
     Nextcloud Talk     PoRE-native       Other Integration
          |                 |                 |
     Talk APIs          PoRE state        own provider(s)
          |                 |                 |
          +-----------------+-----------------+
                            |
                    required PoRE context
```

Der Core und die fachliche `ProductionSession` kennen keine provider-spezifischen Session- oder Teilnehmermodelle.

---

# Der Contract

Der konkrete technische Contract wird in einer späteren Implementierungsentscheidung präzisiert. Auf Architekturebene umfasst er mindestens die folgenden fachlichen Informationsbereiche:

* **Session Identity** — welche Session bzw. welcher externe Kontext gemeint ist
* **Session Availability** — ob der Kontext für die angeforderte Operation verfügbar ist
* **Participant Identity** — wer an der Session teilnimmt bzw. identifizierbar ist
* **Participation / Authorization Context** — welche Teilnahme- und Berechtigungsinformationen für die Operation erforderlich sind
* **Provider Capabilities** — welche für die konkrete Operation benötigten Fähigkeiten der Provider bereitstellt

Nicht jede Integration muss alle Informationen aus derselben Quelle liefern.

Ein Provider darf zusätzliche Informationen und Fähigkeiten besitzen, ohne dass diese Teil des universellen PoRE-Core-Modells werden.

---

# Provider ist nicht gleich Session Owner

Eine externe Integration muss nicht die fachliche PoRE-Session besitzen.

Eine PoRE-Session kann ohne externen Session Provider existieren. Ebenso kann eine Integration einen externen Session Context an eine PoRE-Session binden.

Beispiel Nextcloud Talk:

```text
ProductionSession
       |
       +-- Session Context
              |
              +-- provider = Nextcloud Talk
              +-- external identity = Talk conversation token
              +-- availability = derived from Talk
              +-- participation = derived from Talk
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

Beispiel einer zukünftigen Dropbox-Integration:

```text
Dropbox Integration
       |
       +-- Storage Provider
       |
       +-- optional Session Provider
              |
              +-- supplies whatever the Session Context Contract requires
```

Ein Storage Provider ist daher nicht automatisch ein Session Provider.

---

# Session Lifecycle und Availability

Der fachliche PoRE-Lifecycle und die externe Session-Verfügbarkeit sind getrennte Zustandsdimensionen.

Beispielsweise kann eine PoRE-Session `Completed` sein, obwohl die zugehörige Talk-Conversation noch existiert. Umgekehrt kann eine PoRE-Session noch aktiv sein, obwohl ihre externe Conversation gelöscht oder anderweitig nicht mehr verfügbar ist.

Daraus folgt:

```text
PoRE Session State
        AND
Provider Session Availability
        => operational usability
```

Das Löschen oder Ablaufdatum einer externen Conversation wird nicht automatisch zu einem fachlichen PoRE-Status wie `Completed`.

Die Integration muss stattdessen den Verlust der externen Verfügbarkeit über den Session Context Contract ausdrücken. Die Application Layer entscheidet anschließend, welche PoRE-Operationen noch zulässig sind und welche Reaktion erforderlich ist.

`SessionAvailability` ist daher konzeptionell mehr als ein einfacher Boolean. Der konkrete Statusraum wird providerunabhängig nur insoweit abstrahiert, wie PoRE ihn für seine eigenen Operationen benötigt.

---

# Participation

Teilnahme ist ein Bestandteil des Session Context, aber externe Teilnehmermodelle werden nicht in das PoRE-Core-Modell kopiert.

Nextcloud Talk darf beispielsweise User, Guests, Public-Link-Teilnehmer, E-Mail-Gäste oder federierte Benutzer unterscheiden. PoRE übernimmt daraus nur die Informationen, die für sein eigenes fachliches `Participation`- und Rollenmodell relevant sind.

Dadurch bleibt die Übersetzung explizit:

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

Beispielsweise kann Nextcloud Talk umfangreiche Meeting-, Lobby-, Guest- oder Federation-Funktionen besitzen, ohne dass daraus automatisch entsprechende Core-Abstraktionen entstehen.

Ein zukünftiger Provider darf fehlende Fähigkeiten intern durch eigene Komponenten ergänzen. Ein Provider kann beispielsweise einen eigenen Session Provider verwenden, um Informationen bereitzustellen, die die Plattform selbst nicht besitzt.

---

# Verantwortungsgrenze

Die Architektur folgt damit dem Prinzip:

> **PoRE definiert, was es zum Arbeiten wissen und können muss. Die Integration entscheidet, woher und wie dieser Context bereitgestellt wird.**

Die Integration ist dabei eine Adapter- und Context-Grenze, keine neue Domain-Autorität.

---

# Konsequenzen

## Positive Auswirkungen

* Nextcloud Talk kann seine vorhandene Session-, Identitäts- und Teilnahme-Semantik liefern, ohne dass PoRE sie nachbauen muss.
* Der PoRE-Core bleibt unabhängig von Nextcloud und anderen Plattformen.
* Eine PoRE-native Session bleibt möglich.
* Ein zukünftiger Provider kann fehlende Fähigkeiten durch eigene Session-Provider oder weitere Komponenten ergänzen.
* Session-/Teilnahme-Kontext und Storage bleiben getrennte Architekturachsen.
* Provider-spezifische Features müssen nicht in das universelle Domain-Modell übernommen werden.
* Die Aufnahme- und Application-Logik kann gegen einen stabilen PoRE-Contract arbeiten.

## Negative Auswirkungen

* Der Session Context Contract muss sorgfältig definiert werden, damit er weder zu provider-spezifisch noch zu abstrakt wird.
* Provider-Capabilities müssen explizit behandelt werden, wenn eine Operation nicht überall verfügbar ist.
* Die Übersetzung externer Identitäten in PoRE-Teilnehmeridentitäten benötigt klare Semantik.
* Die Reaktion auf verlorene externe Verfügbarkeit muss auf Application-Ebene definiert werden.

---

# Betrachtete Alternativen

## Nextcloud Talk als Session-Modell für PoRE

Verworfen. Dadurch würde PoRE das Modell eines einzelnen Providers übernehmen und zukünftige Integrationen unnötig erschweren.

## Universelles PoRE-Sessionmodell als vollständiger Ersatz für Provider

Verworfen. Damit würde PoRE Funktionen nachbauen, die Plattformen wie Nextcloud Talk bereits zuverlässig bereitstellen.

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

# Zukünftige Betrachtungen

Eine spätere Implementierungs-ADR muss den konkreten Contract definieren, insbesondere:

* welche Context-Daten verpflichtend sind
* welche Daten optional sind
* wie externe Identitäten repräsentiert werden
* welche Availability-Zustände PoRE tatsächlich benötigt
* wie Capability-Abfragen aussehen
* wie Session-Erzeugung und Session-Bindung funktionieren
* wie der Verlust eines externen Session Context behandelt wird
* wie Providerwechsel bzw. neue Bindungen behandelt werden

Erst nach dieser Definition sollte die aktuelle Feasibility-/Client-Schicht auf den neuen Contract umgebaut werden.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-067: Session Context and External Provider Contract

## Status

Proposed

## Date

2026-08-20

## Decision Type

Architecture

---

# Context

NC-PoRe needs a defined technical context for a complete recording: the session must be available, participants must be identifiable and authorizable for the required operations, and the session and participation information required by the recording workflow must be reliably available.

Nextcloud Talk already provides much of this context. Talk has its own semantics for conversations, identities, guests, registered users, federation, access rules and conversation lifecycle. NC-PoRe should consume these capabilities rather than rebuild or copy Talk's model into its domain model.

At the same time, the architecture must not assume that every future integration provider has the same capabilities. A future integration, for example Dropbox as a storage provider, may require its own session provider or may provide parts of the required context itself.

This creates two architectural failure modes:

1. NC-PoRe adopts a concrete external session model and becomes dependent on it.
2. NC-PoRe attempts to rebuild every provider capability itself and loses the benefits of existing platforms.

---

# Decision

NC-PoRe defines a **Session Context Contract** as the explicit entry/exit boundary between PoRE application logic and an external integration environment.

PoRE defines **which information and capabilities are required for a concrete operation**. The integration is responsible for providing that context.

The context may be provided entirely by an external provider, by PoRE itself, or by a combination of multiple sources.

Conceptually:

```text
                         NC-PoRe
                            |
                 Session Context Contract
                            |
          +-----------------+-----------------+
          |                 |                 |
     Nextcloud Talk     PoRE-native       Other Integration
          |                 |                 |
     Talk APIs          PoRE state        own provider(s)
          |                 |                 |
          +-----------------+-----------------+
                            |
                    required PoRE context
```

The Core and the domain `ProductionSession` do not know provider-specific session or participant models.

---

# The Contract

The concrete technical contract will be specified in a later implementation decision. At architecture level it covers at least:

* **Session Identity** — which session or external context is being referenced
* **Session Availability** — whether that context is available for the requested operation
* **Participant Identity** — who participates in the session and can be identified
* **Participation / Authorization Context** — participation and authorization information required for the operation
* **Provider Capabilities** — capabilities required for the concrete operation

Not every integration has to source all information from the same place.

A provider may expose additional information and capabilities without making them part of the universal PoRE Core model.

---

# Provider Does Not Equal Session Owner

An external integration does not have to own the PoRE domain session.

A PoRE session may exist without an external session provider. Conversely, an integration may bind an external session context to a PoRE session.

For example, Nextcloud Talk can provide conversation identity, availability and participation information, while a PoRE-native integration can provide the same context from PoRE state.

A future Dropbox integration may contain both a storage provider and, if needed, its own session provider. A storage provider is therefore not automatically a session provider.

---

# Session Lifecycle and Availability

The PoRE domain lifecycle and external session availability are separate state dimensions.

A PoRE session may be `Completed` while its Talk conversation still exists. Conversely, a PoRE session may still be active while its external conversation has been deleted or is otherwise unavailable.

Therefore:

```text
PoRE Session State
        AND
Provider Session Availability
        => operational usability
```

Deletion or expiry of an external conversation does not automatically become a PoRE domain state such as `Completed`.

The integration expresses loss of external availability through the Session Context Contract. The Application Layer decides which PoRE operations remain valid and what reaction is required.

`SessionAvailability` is therefore conceptually more than a boolean. The provider-independent state space should be kept only as broad as required by PoRE operations.

---

# Participation

Participation is part of the Session Context, but external participant models are not copied into the PoRE Core model.

Nextcloud Talk may distinguish users, guests, public-link participants, email guests or federated users. PoRE only consumes the information required for its own `Participation` and role model.

The mapping is therefore explicit:

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

Capabilities are not intended to be a universal mirror of a provider.

An integration may support significantly more than PoRE needs. PoRE asks only for capabilities relevant to the current operation.

Nextcloud Talk may provide extensive meeting, lobby, guest or federation functionality without those capabilities automatically becoming Core abstractions.

A future provider may compensate for missing platform capabilities through its own components. For example, an integration may use its own session provider to supply information the platform itself does not provide.

---

# Responsibility Boundary

The architecture follows this principle:

> **PoRE defines what it needs to know and be able to do. The integration decides where and how that context is provided.**

The integration is an adapter and context boundary, not a new domain authority.

---

# Consequences

## Positive Effects

* Nextcloud Talk can provide its existing session, identity and participation semantics without PoRE rebuilding them.
* The PoRE Core remains independent of Nextcloud and other platforms.
* PoRE-native sessions remain possible.
* Future providers can supplement missing capabilities through their own session providers or other components.
* Session/participation context and storage remain separate architectural axes.
* Provider-specific features do not have to enter the universal domain model.
* Recording and application logic can operate against a stable PoRE contract.

## Negative Effects

* The Session Context Contract must be defined carefully to avoid becoming either provider-specific or excessively abstract.
* Provider capabilities must be handled explicitly when an operation is not universally available.
* Mapping external identities to PoRE participant identities requires clear semantics.
* Handling loss of external availability must be defined at application level.

---

# Alternatives Considered

## Nextcloud Talk as the PoRE Session Model

Rejected. This would make PoRE adopt one provider's model and unnecessarily complicate future integrations.

## Universal PoRE Session Model as a Complete Replacement for Providers

Rejected. This would force PoRE to rebuild functionality that platforms such as Nextcloud Talk already provide reliably.

## Storage Provider and Session Provider as One Abstraction

Rejected. Data storage and session/participation context are different responsibilities and may be implemented independently.

## Simple `is_valid()` Check for External Sessions

Rejected. External availability can have multiple relevant states. PoRE needs availability semantics, not merely a boolean.

---

# Relationship to Existing Architecture

This decision complements ADR-022 and ADR-026 with an explicit provider boundary for session and participation context.

It builds on ADR-031 for identity and roles and ADR-062 for browser-based guest participation.

It complements ADR-065 by keeping Storage Providers and Session Context as separate integration axes.

The existing `ProductionSession` remains a domain Core structure and is not enriched with provider-specific session data.

---

# Future Considerations

A later implementation ADR must define the concrete contract, including:

* mandatory and optional context data
* representation of external identities
* availability states actually required by PoRE
* capability queries
* session creation and binding semantics
* handling loss of an external session context
* handling provider changes and new bindings

Only after that definition should the current feasibility/client layer be migrated to the new contract.
