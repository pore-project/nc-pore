# Deutsch ([English version below](#english-version))

# ADR-005: Zustimmung und Transparenz bei Aufnahmen

## Status

Angenommen

## Datum

2026-07-22

---

# Kontext

Podcast- und Gesprächsaufnahmen betreffen persönliche Kommunikation und können personenbezogene Daten enthalten.

Eine technische Möglichkeit zur Aufnahme darf nicht bedeuten, dass Teilnehmer unbewusst oder ohne klare Information aufgenommen werden.

NC-PoRe soll Datenschutz nicht nur technisch ermöglichen, sondern aktiv sichtbar machen.

---

# Entscheidung

NC-PoRe behandelt die Zustimmung zur Aufnahme als Bestandteil einer Session.

Eine Aufnahme darf nur stattfinden, wenn die erforderlichen Zustimmungen vorliegen.

---

# Transparenz bei der Aufnahme

Während einer laufenden Aufnahme muss für alle Teilnehmer sichtbar sein:

- dass eine Aufnahme aktiv ist
- welche Spur lokal aufgezeichnet wird
- welcher Status die eigene Aufnahme besitzt

Beispiele:

- Aufnahme aktiv
- Mikrofon: verbunden
- Lokale Aufnahme: läuft
- Upload: wartet bis Session-Ende

---

# Zustimmungsmodell

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
```

## Gastaufnahmen

NC-PoRe unterstützt externe Gäste ohne vollständiges Benutzerkonto.

Gäste:

- erhalten die Aufnahmeinformation
- bestätigen die Teilnahme
- dürfen nur die vorgesehenen Aktionen ausführen
- können nicht auf fremde Daten zugreifen

# Konsequenzen

## Positive Auswirkungen

- hohe Transparenz gegenüber Teilnehmern
- nachvollziehbare Zustimmung
- bessere Datenschutzgrundlage
- höheres Vertrauen bei Gästen
- klare Verantwortlichkeiten

## Negative Auswirkungen

- zusätzlicher Session- und Datenverwaltungsaufwand
- Zustimmung muss technisch gespeichert werden
- Änderungen an Einwilligungstexten müssen versioniert werden

# Betrachtete Alternativen

## Stille Aufnahme mit nachträglicher Information

Verworfen.

Gründe:

- widerspricht der Projektphilosophie
- reduziert Vertrauen
- entspricht nicht dem gewünschten Umgang mit Daten

## Einmalige globale Zustimmung

Nicht ausreichend.

Gründe:

- Eine Zustimmung soll immer einen konkreten Aufnahmevorgang betreffen.

# Hinweise

Transparenz ist ein Kernfeature von NC-PoRe.

Die Anzeige einer aktiven Aufnahme ist kein Warnhinweis, sondern ein Vertrauenssignal.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-005: Consent and Recording Transparency

## Status

Accepted

## Date

2026-07-22

---

# Context

Podcast and conversation recordings involve personal communication and may contain personal data.

The technical ability to record must not mean that participants are recorded without awareness or clear information.

NC-PoRe should not only enable data protection technically, but also make it actively visible.

---

# Decision

NC-PoRe treats consent to recording as part of a session.

A recording may only take place when the required consents have been obtained.

---

# Recording Transparency

During an active recording, all participants must be able to see:

- that a recording is active
- which track is being recorded locally
- the status of their own recording

Examples:

- Recording active
- Microphone: connected
- Local recording: running
- Upload: waiting until session end

---

# Consent Model

Consent is documented on a session basis.

A consent record contains at least:

- participant ID
- session ID
- time of consent
- version of the consent information
- status

Example:

```json
{
  "participant": "guest-123",
  "session": "episode-042",
  "consent": true,
  "timestamp": "2026-07-22T20:00:00Z"
}
```

## Guest Recordings

NC-PoRe supports external guests without a full user account.

Guests:

- receive the recording information
- confirm their participation
- may only perform the intended actions
- cannot access other participants' data

# Consequences

## Positive Effects

- high transparency for participants
- traceable consent
- stronger data protection basis
- greater trust among guests
- clear responsibilities

## Negative Effects

- additional session and data management effort
- consent must be stored technically
- changes to consent texts must be versioned

# Alternatives Considered

## Silent Recording with Subsequent Information

Rejected.

Reasons:

- contradicts the project philosophy
- reduces trust
- does not reflect the intended approach to data

## One-Time Global Consent

Insufficient.

Reasons:

- Consent should always relate to a specific recording process.

# Notes

Transparency is a core feature of NC-PoRe.

Displaying an active recording is not a warning, but a signal of trust.
