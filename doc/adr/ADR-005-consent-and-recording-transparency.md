# ADR-005: Consent and Recording Transparency

## Status

Accepted

## Date

2026-07-22

---

# Context

Podcast- und Gesprächsaufnahmen betreffen persönliche
Kommunikation und können personenbezogene Daten enthalten.

Eine technische Möglichkeit zur Aufnahme darf nicht bedeuten,
dass Teilnehmer unbewusst oder ohne klare Information
aufgenommen werden.

NC-PoRe soll Datenschutz nicht nur technisch ermöglichen,
sondern aktiv sichtbar machen.

---

# Decision

NC-PoRe behandelt die Zustimmung zur Aufnahme als Bestandteil
einer Session.

Eine Aufnahme darf nur stattfinden, wenn die erforderlichen
Zustimmungen vorliegen.

---

# Recording Transparency

Während einer laufenden Aufnahme muss für alle Teilnehmer
sichtbar sein:

- dass eine Aufnahme aktiv ist
- welche Spur lokal aufgezeichnet wird
- welcher Status die eigene Aufnahme besitzt

Beispiele:
● Aufnahme aktiv

Mikrofon:
verbunden

Lokale Aufnahme:
läuft

Upload:
wartet bis Session-Ende

---

# Consent Model

Die Zustimmung wird sessionbezogen dokumentiert.

Eine Zustimmung enthält mindestens:

- Teilnehmer-ID
- Session-ID
- Zeitpunkt der Zustimmung
- Version der Einwilligungsinformation
- Status

Beispiel:

```json
{
  "participant": "guest-123",
  "session": "episode-042",
  "consent": true,
  "timestamp": "2026-07-22T20:00:00Z"
}

Gastaufnahmen

NC-PoRe unterstützt externe Gäste ohne vollständiges
Benutzerkonto.

Gäste:

erhalten die Aufnahmeinformation
bestätigen die Teilnahme
dürfen nur die vorgesehenen Aktionen ausführen
können nicht auf fremde Daten zugreifen
Consequences
Positive Auswirkungen
hohe Transparenz gegenüber Teilnehmern
nachvollziehbare Zustimmung
bessere Datenschutzgrundlage
höheres Vertrauen bei Gästen
klare Verantwortlichkeiten
Negative Auswirkungen
zusätzlicher Session- und Datenverwaltungsaufwand
Zustimmung muss technisch gespeichert werden
Änderungen an Einwilligungstexten müssen versioniert werden
Alternatives considered
Stille Aufnahme mit nachträglicher Information

Verworfen.

Gründe:

widerspricht der Projektphilosophie
reduziert Vertrauen
entspricht nicht dem gewünschten Umgang mit Daten
Einmalige globale Zustimmung

Nicht ausreichend.

Gründe:

Eine Zustimmung soll immer einen konkreten Aufnahmevorgang
betreffen.

Notes

Transparenz ist ein Kernfeature von NC-PoRe.

Die Anzeige einer aktiven Aufnahme ist kein Warnhinweis,
sondern ein Vertrauenssignal.