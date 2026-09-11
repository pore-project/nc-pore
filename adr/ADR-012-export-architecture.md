# Deutsch ([English version below](#english-version))

# ADR-012: Export Architecture

## Status

Accepted

## Date

2026-07-22

---

# Kontext

NC-PoRe erzeugt hochwertige Mehrspuraufnahmen. Die Aufnahme ist jedoch nicht das Ende des Produktionsprozesses. Produktionsdaten sollen außerhalb von NC-PoRe weiterverarbeitet werden können, ohne den Benutzer an eine proprietäre Produktionsumgebung zu binden.

---

# Entscheidung

NC-PoRe trennt Aufnahme und Produktion.

Die Plattform erzeugt offene Produktionsdaten, die außerhalb von NC-PoRe weiterverarbeitet werden können.

---

# Exportprinzipien

Ein Export enthält die für die Weiterverarbeitung erforderlichen:

- Audiodaten
- Metadaten
- Synchronisationsinformationen
- Sessioninformationen

Der grundlegende Export besteht aus einzelnen Audiospuren und den zugehörigen Informationen. Er bleibt unabhängig von proprietären Produktionswerkzeugen.

---

# Exportverantwortung

Exportierte Daten gehören vollständig dem Benutzer.

NC-PoRe verhindert keine Weiterverarbeitung außerhalb der Plattform.

Offene Datenformate und der Zugriff auf die eigenen Produktionsdaten bleiben zentrale Anforderungen.

---

# Interoperabilität

Exportierte Produktionsdaten sollen mit etablierten externen Werkzeugen weiterverarbeitet werden können. Konkrete Werkzeug- oder Projektformate sind dabei eine Implementierungsfrage und werden nicht als Bestandteil dieser Architekturentscheidung festgeschrieben.

---

# Konsequenzen

## Positive Auswirkungen

- keine Abhängigkeit von NC-PoRe für die weitere Verarbeitung
- Unterstützung offener Produktionsworkflows
- professionelle externe Werkzeuge können verwendet werden
- langfristige Datenverfügbarkeit

## Negative Auswirkungen

- zusätzliche Exportlogik erforderlich
- Exportformate müssen gepflegt und getestet werden

---

# Betrachtete Alternativen

## Eigenes geschlossenes Projektformat

Verworfen.

Grund: Widerspricht der Datenhoheit und der FOSS-Philosophie.

---

## Nur fertige Audiodatei exportieren

Verworfen.

Grund: Nicht ausreichend für die Weiterverarbeitung von Mehrspur-Produktionsdaten.

---

# Hinweise

NC-PoRe stellt Rohmaterial und Produktionsdaten bereit. Die kreative Entscheidung über die weitere Bearbeitung bleibt beim Menschen und bei den von ihm gewählten Werkzeugen.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-012: Export Architecture

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe produces high-quality multitrack recordings. Recording is not the end of the production process. Production data should remain usable outside NC-PoRe without locking users into a proprietary production environment.

---

# Decision

NC-PoRe separates recording from production.

The platform produces open production data that can be processed outside NC-PoRe.

---

# Export Principles

An export contains the information required for further processing:

- audio data
- metadata
- synchronization information
- session information

The basic export consists of individual audio tracks and their associated information. It remains independent of proprietary production tools.

---

# Export Ownership

Exported data belongs entirely to the user.

NC-PoRe does not prevent further processing outside the platform.

Open data formats and access to the user's own production data remain core requirements.

---

# Interoperability

Exported production data should remain usable with established external tools. Concrete tool- or project-specific formats are implementation concerns and are not prescribed by this architectural decision.

---

# Consequences

## Positive Effects

- no dependency on NC-PoRe for further processing
- support for open production workflows
- professional external tools can be used
- long-term data availability

## Negative Effects

- additional export logic is required
- export formats require maintenance and testing

---

# Alternatives Considered

## Proprietary Closed Project Format

Rejected.

Reason: Contradicts data ownership and the FOSS philosophy.

---

## Export Only the Final Audio File

Rejected.

Reason: Insufficient for further processing of multitrack production data.

---

# Notes

NC-PoRe provides raw material and production data. Creative decisions about further processing remain with the human and the tools they choose.
