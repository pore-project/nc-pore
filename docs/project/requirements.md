# NC-PoRe Requirements

## Ziel

NC-PoRe ist eine selbstgehostete Podcast-Produktionsplattform mit lokaler Audioaufnahme und kontrollierter Übergabe der Aufnahmen an die eigene Infrastruktur.

---

# Funktionale Anforderungen

## Aufnahme

NC-PoRe muss:

- Audio lokal auf dem Teilnehmergerät aufnehmen.
- Die Aufnahme unabhängig von der Netzwerkqualität ermöglichen.
- Die verfügbare Aufnahmequalität des Eingabegeräts möglichst originalgetreu erhalten.
- Mehrere Recording-Teilnehmer getrennt aufnehmen können.

---

## Speicherung

NC-PoRe muss:

- lokale und kontrollierte Zwischenspeicherung ermöglichen.
- Aufnahmen in wiederherstellbaren Einheiten speichern.
- abgeschlossene Aufnahmen kontrolliert an die zentrale Umgebung übertragen können.
- die Originalaufnahmen beziehungsweise ihre maßgebliche Aufnahmeinformation unverändert erhalten.

---

## Teilnehmer

NC-PoRE muss unterstützen:

- interne Benutzer
- externe Gäste
- Rollen und Berechtigungen
- eine eindeutige Zuordnung von Personen zu Recording Sessions

Welche Personen an einer konkreten Aufnahme teilnehmen und welche Berechtigungen sie besitzen, wird durch die fachlichen Session- und Rollenregeln bestimmt.

---

## Datenschutz

NC-PoRe muss:

- transparent über laufende Aufnahmen informieren.
- die erforderlichen Zustimmungen beziehungsweise Aufnahmefreigaben nachvollziehbar behandeln.
- ohne Abhängigkeit von einer externen Cloud-Infrastruktur betrieben werden können.

---

## Export und Weiterverarbeitung

NC-PoRe soll ermöglichen:

- Weiterverarbeitung der aufgenommenen Audiodaten mit externen Werkzeugen.
- strukturierte Ablage der Audiospuren und zugehörigen Metadaten.
- Nutzung offener beziehungsweise nachvollziehbarer Datenrepräsentationen.

Konkrete Exportformate oder Integrationen werden durch die jeweils getroffenen Architektur- und Implementierungsentscheidungen bestimmt.

---

# Nicht-Ziele der aktuellen Ausbaustufe

Nicht Bestandteil der aktuellen Ausbaustufe sind insbesondere:

- Videoaufnahme
- Live-Mixing
- Streaming
- automatische Veröffentlichung
- automatische Verarbeitung durch externe KI-Dienste

Diese Nicht-Ziele beschreiben den aktuellen Scope und stellen keine Zusage für spätere Funktionen dar.
