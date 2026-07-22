# ADR-008: Client Architecture

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe benötigt eine zuverlässige lokale Audioaufnahme.

Die Aufnahmequalität darf nicht von einem Browser, einer
Serververbindung oder externen Diensten abhängig sein.

Gleichzeitig soll NC-PoRe möglichst einfach zugänglich sein,
insbesondere für Gäste und gelegentliche Teilnehmer.

Daraus entsteht ein Zielkonflikt:

* maximale technische Kontrolle für professionelle Aufnahmen
* möglichst einfacher Zugang für Teilnehmer

---

# Decision

NC-PoRe verwendet eine modulare Client-Architektur.

Die lokale Aufnahme erfolgt durch einen spezialisierten
Recorder-Client.

Der Recorder ist für folgende Aufgaben verantwortlich:

* Zugriff auf Audiohardware
* lokale Aufnahme
* Chunk-Verwaltung
* Metadaten-Erzeugung
* lokale Sicherheit
* Upload-Vorbereitung

Der Server übernimmt keine primäre Audioaufnahme.

---

# Client-Varianten

NC-PoRe unterstützt perspektivisch unterschiedliche
Client-Varianten.

## Professional Recorder

Für regelmäßige Podcaster und Produktionsumgebungen.

Eigenschaften:

* maximale Audioqualität
* erweiterte Einstellungen
* zuverlässige lokale Speicherung
* professionelle Workflows

---

## Guest Recorder

Für externe Teilnehmer.

Ziel:

* möglichst einfache Teilnahme
* geringe Einstiegshürde
* sichere Session-Teilnahme

Der Gast benötigt keine umfangreiche Verwaltung.

---

# Architekturmodell

```
                Nextcloud Server

                    |
                    |
          Session Management
                    |
        +-----------+-----------+
        |                       |
        |                       |
 Professional Client      Guest Client

        |                       |
        +-----------+-----------+

              lokale Aufnahme

                    |

             Upload nach Session-Ende
```

---

# Browser-basierte Aufnahme

Eine reine Browser-Aufnahme wird nicht als
Primärarchitektur verwendet.

Gründe:

* eingeschränkte Kontrolle über Hardwarezugriff
* abhängig vom Browserverhalten
* schwierigeres Fehlerhandling
* eingeschränkte Möglichkeiten für professionelle Workflows

Browserbasierte Teilnahme kann jedoch zukünftig als
vereinfachter Zugang unterstützt werden.

---

# Consequences

## Positive Auswirkungen

* professionelle Aufnahmequalität möglich
* klare Trennung zwischen Aufnahme und Server
* bessere Erweiterbarkeit
* geeignet für verschiedene Nutzergruppen
* unabhängiger von Browserherstellern

---

## Negative Auswirkungen

* zusätzliche Softwarekomponente erforderlich
* Installation kann notwendig sein
* mehrere Clients müssen gepflegt werden

---

# Alternatives considered

## Ausschließliche Web-App

Verworfen als Hauptlösung.

Grund:

Eine Web-App bietet nicht die notwendige Kontrolle für
professionelle lokale Audioaufnahme.

---

## Ausschließlicher Desktop-Client

Nicht ausreichend.

Grund:

Gelegenheitsnutzer und Gäste benötigen einen einfacheren
Zugang.

---

# Notes

Die Client-Architektur unterstützt das Grundprinzip von NC-PoRe:

> Professionelle Werkzeuge für diejenigen, die sie benötigen,
> einfache Teilnahme für diejenigen, die nur beitragen.
