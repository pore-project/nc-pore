# ADR-088 Versioning and Release Policy

- Status: Accepted
- Date: 2026-09-27
- Decision Type: Project / Release Policy

---

# Deutsch ([English version below](#english-version))
## Kontext

NC-PoRe verwendet derzeit Entwicklungsstände in der Form `0.1.0-dev.N`. Die `dev.N`-Kennung wurde bewusst eingeführt, um testbare und eindeutig unterscheidbare Entwicklungsstände zu markieren.

Der feste Präfix `0.1.0` ist dagegen keine fachliche Aussage über den jeweiligen Entwicklungsstand. Mit dem Übergang von der V1-Entwicklung in eine öffentliche stabile Release-Linie soll die Versionsnummer deshalb auf eine nachvollziehbare und langfristig belastbare SemVer-Struktur umgestellt werden.

Für NC-PoRe sollen damit zwei unterschiedliche Informationen sauber getrennt werden:

- die **Release-Version**, die die Kompatibilitäts- und Änderungsart ausdrückt;
- die **Pre-Release-/Development-Kennung**, die einen konkreten Entwicklungsstand innerhalb einer Release-Linie bezeichnet.

## Entscheidung

NC-PoRe verwendet für veröffentlichte Versionen und deren Vorstufen **Semantic Versioning (SemVer)** in der Form:

`MAJOR.MINOR.PATCH`

Pre-Releases und Entwicklungsstände werden als SemVer-Prerelease-Suffix geführt.

### 1. V1 erhält die Release-Baseline `1.0.0`

Die erste stabile öffentliche V1 von NC-PoRE wird als:

`1.0.0`

veröffentlicht.

Die bestehende frühe Entwicklungsreihe `0.1.0-dev.N` wird nicht rückwirkend umbenannt.

Beim Eintritt der V1 in den definierten Beta-/Release-Zyklus beginnt die V1-Pre-Release-Linie auf Basis von `1.0.0`.

Der vorgesehene Ablauf ist damit:

```text
0.1.0-dev.N
      |
      | V1 tritt in den Beta-/Release-Zyklus ein
      v
1.0.0-beta.1
1.0.0-beta.2
...
1.0.0-rc.1
...
1.0.0
```

Eine `alpha`-Linie ist nach derselben SemVer-Systematik möglich, wird für den aktuellen Übergang jedoch nicht künstlich eingeführt.

### 2. Development-Stände verwenden weiterhin `dev.N`

Während eines laufenden Release-Zyklus können konkrete Entwicklungsstände als Prerelease gekennzeichnet werden, zum Beispiel:

```text
1.1.0-dev.1
1.1.0-dev.2
1.1.0-dev.3
```

Der Zähler `N` ist ein **Entwicklungszähler**, kein SemVer-MAJOR-, MINOR- oder PATCH-Zähler.

Er dient dazu, zusammenhängende testbare Entwicklungs-/Review-Stände eindeutig zu unterscheiden.

Es gilt weiterhin die bestehende Arbeitsregel:

> Ein `dev.N`-Stand wird pro zusammenhängendem Entwicklungs-/Review-Zyklus geführt, nicht pro beliebigem internen Zwischenzustand.

### 3. Bedeutung von MAJOR, MINOR und PATCH ab `1.0.0`

#### PATCH

Die PATCH-Zahl wird erhöht, wenn ausschließlich rückwärtskompatible Fehlerbehebungen oder gleichartige Korrekturen veröffentlicht werden.

Beispiel:

```text
1.0.0
1.0.1
1.0.2
1.0.3
```

#### MINOR

Die MINOR-Zahl wird erhöht, wenn eine neue, rückwärtskompatible Funktionalität hinzukommt.

Beim Wechsel auf eine neue MINOR-Version wird PATCH auf `0` zurückgesetzt.

Beispiel:

```text
1.0.0
1.1.0
1.1.1
1.1.2
1.2.0
```

Dabei können auf eine MINOR-Version beliebig viele PATCH-Releases folgen.

#### MAJOR

Die MAJOR-Zahl wird erhöht, wenn eine Änderung eine bestehende unterstützte Schnittstelle oder Nutzung in einer inkompatiblen Weise verändert.

Beim Wechsel auf eine neue MAJOR-Version werden MINOR und PATCH auf `0` gesetzt.

Beispiel:

```text
1.7.4
2.0.0
```

Die konkrete Definition der bei NC-PoRe als öffentlich bzw. unterstützt geltenden API-/Integrationsflächen wird separat geklärt und ist nicht Bestandteil dieser ADR.

### 4. Pre-Release-Stufen

Für einen Release-Zyklus gelten die folgenden möglichen Stufen:

```text
<release>-alpha.N
<release>-beta.N
<release>-rc.N
<release>
```

Beispiel:

```text
1.0.0-alpha.1
1.0.0-beta.1
1.0.0-beta.2
1.0.0-rc.1
1.0.0
```

Eine stabile Release-Version trägt kein Pre-Release-Suffix.

Nach SemVer hat eine Pre-Release-Version eine niedrigere Versionspriorität als die zugehörige stabile Version.

### 5. Development und Release-Zähler werden nicht vermischt

Die folgenden Zähler haben unterschiedliche Bedeutungen und werden daher nicht gegeneinander fortgeschrieben:

```text
1.2.0-dev.17
│ │ │     └── Entwicklungsstand
│ │ └────── PATCH
│ └──────── MINOR
└────────── MAJOR
```

Wenn aus einer Entwicklungsreihe `1.2.0-dev.N` ein stabiles Release wird, lautet die stabile Version `1.2.0`; der `dev.N`-Zähler wird nicht Bestandteil der Release-Version.

Ebenso bedeutet ein neuer `dev.N`-Stand nicht automatisch eine MINOR- oder PATCH-Erhöhung. Die Release-Klassifikation wird nach der Art der enthaltenen Änderungen festgelegt.

### 6. Examples für die NC-PoRE-Entwicklung

Eine typische Entwicklung innerhalb einer kompatiblen Feature-Linie kann so aussehen:

```text
1.0.0
1.0.1
1.0.2
1.1.0
1.1.0-dev.1
1.1.0-dev.2
1.1.0-beta.1
1.1.0-rc.1
1.1.0
1.1.1
```

Eine nächste inkompatible Generation beginnt entsprechend mit:

```text
2.0.0-beta.1
...
2.0.0
```

### 7. Konsistenz der Versionsangaben

Die für eine auslieferbare NC-PoRe-Version maßgebliche App-Version muss mit der tatsächlich ausgelieferten Version übereinstimmen.

Versionsangaben in `appinfo/info.xml`, Release-Artefakten und begleitender Projektdokumentation dürfen nicht dauerhaft auseinanderlaufen.

Die Versionsnummer eines konkreten Entwicklungsstands muss vor CI-/Review-Abschluss eindeutig bestimmbar sein.

## Konsequenzen

- `1.0.0` ist die stabile öffentliche V1 von NC-PoRe.
- `0.1.0-dev.N` bleibt die bestehende frühe Entwicklungsreihe und wird nicht rückwirkend umgeschrieben.
- Die V1-Beta-/Release-Linie verwendet `1.0.0-beta.N`, danach `1.0.0-rc.N` und schließlich `1.0.0`.
- PATCH beschreibt kompatible Fehlerkorrekturen.
- MINOR beschreibt kompatible Funktionserweiterungen.
- MAJOR beschreibt inkompatible Änderungen an unterstützten Schnittstellen.
- `dev.N` beschreibt Entwicklungsstände und ist unabhängig von MAJOR/MINOR/PATCH.
- Dokumentation und ausgelieferte App-Version sollen konsistent gehalten werden.
- Die genaue Definition der öffentlichen API-/Integrationsfläche bleibt eine separate offene Aufgabe.

## Relationship to Existing Architecture

Diese Entscheidung ergänzt insbesondere:

- ADR-037 Development Workflow and Source of Truth
- ADR-071 Recording Capture, Preservation and Transport Formats
- die bestehende Entwicklungsregel für `dev.N`

Sie trifft keine Architekturentscheidung über Recording, Capture, Preservation, Transport oder die Core-Domäne.

## Status

Diese Entscheidung ist angenommen und gilt ab sofort als Grundlage für die künftige Versions- und Releaseplanung von NC-PoRe.

---

# English Version ([Deutsche Version oben](#deutsch))
## Context

NC-PoRe currently uses development versions in the form `0.1.0-dev.N`. The `dev.N` suffix was intentionally introduced to identify concrete, testable development states.

The fixed `0.1.0` prefix, however, does not describe the semantic meaning of the current release state. When V1 moves into a public stable release line, versioning should therefore use a consistent and durable Semantic Versioning structure.

NC-PoRe should keep two distinct pieces of information separate:

- the **release version**, which describes compatibility and the type of change;
- the **pre-release/development identifier**, which identifies a concrete state within a release line.

## Decision

NC-PoRE uses **Semantic Versioning (SemVer)** for releases in the form:

`MAJOR.MINOR.PATCH`

Development and pre-release states use SemVer prerelease identifiers.

### 1. V1 uses the `1.0.0` release baseline

The first stable public V1 of NC-PoRe will be released as:

`1.0.0`

The existing early-development series `0.1.0-dev.N` is not renamed retroactively.

When V1 enters the defined beta/release cycle, the V1 pre-release line starts from the `1.0.0` baseline.

The intended flow is:

```text
0.1.0-dev.N
      |
      | V1 enters the beta/release cycle
      v
1.0.0-beta.1
1.0.0-beta.2
...
1.0.0-rc.1
...
1.0.0
```

An `alpha` line is possible under the same SemVer system, but is not introduced artificially for the current transition.

### 2. Development states continue to use `dev.N`

During a release cycle, concrete development states may be represented as:

```text
1.1.0-dev.1
1.1.0-dev.2
1.1.0-dev.3
```

The `N` value is a **development counter**, not a SemVer MAJOR, MINOR or PATCH counter.

It identifies coherent testable development/review states.

The existing working agreement remains:

> One `dev.N` state is used per coherent development/review cycle, not per arbitrary internal intermediate state.

### 3. Meaning of MAJOR, MINOR and PATCH after `1.0.0`

#### PATCH

PATCH is incremented for backward-compatible bug fixes and equivalent corrective changes.

Example:

```text
1.0.0
1.0.1
1.0.2
1.0.3
```

#### MINOR

MINOR is incremented when backward-compatible functionality is added.

When moving to a new MINOR version, PATCH resets to `0`.

Example:

```text
1.0.0
1.1.0
1.1.1
1.1.2
1.2.0
```

Any number of PATCH releases may follow a MINOR release.

#### MAJOR

MAJOR is incremented when a change modifies a supported interface or usage in an incompatible way.

When moving to a new MAJOR version, MINOR and PATCH reset to `0`.

Example:

```text
1.7.4
2.0.0
```

The concrete definition of which API/integration surfaces are considered public or supported by NC-PoRe is handled separately and is not part of this ADR.

### 4. Pre-release stages

The possible stages of a release cycle are:

```text
<release>-alpha.N
<release>-beta.N
<release>-rc.N
<release>
```

Example:

```text
1.0.0-alpha.1
1.0.0-beta.1
1.0.0-beta.2
1.0.0-rc.1
1.0.0
```

A stable release has no pre-release suffix.

Under SemVer, a pre-release version has lower precedence than its corresponding stable release.

### 5. Development and release counters are not mixed

The following counters have different meanings and therefore are not advanced as one sequence:

```text
1.2.0-dev.17
│ │ │     └── development state
│ │ └────── PATCH
│ └──────── MINOR
└────────── MAJOR
```

When a development series `1.2.0-dev.N` becomes a stable release, the stable version is `1.2.0`; the `dev.N` identifier is not part of the release version.

Likewise, a new `dev.N` state does not automatically imply a MINOR or PATCH increase. Release classification is based on the nature of the included changes.

### 6. NC-PoRE development example

A typical evolution within a compatible feature line may look like:

```text
1.0.0
1.0.1
1.0.2
1.1.0
1.1.0-dev.1
1.1.0-dev.2
1.1.0-beta.1
1.1.0-rc.1
1.1.0
1.1.1
```

A new incompatible generation begins accordingly with:

```text
2.0.0-beta.1
...
2.0.0
```

### 7. Version consistency

The app version shipped in a release must match the actual release version.

Version values in `appinfo/info.xml`, release artifacts and maintained project documentation must not remain inconsistent.

The version of a concrete development state must be unambiguous before CI/review completion.

## Consequences

- `1.0.0` is the stable public V1 of NC-PoRE.
- `0.1.0-dev.N` remains the historical early-development line and is not rewritten retroactively.
- The V1 beta/release line uses `1.0.0-beta.N`, then `1.0.0-rc.N`, and finally `1.0.0`.
- PATCH represents compatible bug fixes.
- MINOR represents compatible feature additions.
- MAJOR represents incompatible changes to supported interfaces.
- `dev.N` identifies development states independently of MAJOR/MINOR/PATCH.
- Documentation and shipped app versions should remain consistent.
- The exact definition of the public API/integration surface remains a separate open task.

## Relationship to Existing Architecture

This decision complements in particular:

- ADR-037 Development Workflow and Source of Truth
- ADR-071 Recording Capture, Preservation and Transport Formats
- the existing `dev.N` development rule

It does not change any architecture decision about recording, capture, preservation, transport or the Core domain.

## Status

This decision is accepted and is effective immediately as the basis for future NC-PoRe versioning and release planning.
