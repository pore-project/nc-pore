# NC-PoRe Architecture

## Überblick

NC-PoRe ist eine selbstgehostete Podcast-Produktionsplattform.

Die Architektur trennt Aufnahme, Verwaltung und Produktion voneinander.

Grundprinzip:

> Audioaufnahme erfolgt lokal. Server dienen der Verwaltung, Speicherung und Weiterverarbeitung.

---

# Systemkomponenten

NC-PoRe besteht aus folgenden Hauptkomponenten:

## 1. Nextcloud Integration

Verantwortlich für:

- Benutzerverwaltung
- Projekte
- Sessions
- Rechteverwaltung
- Speicherung
- Zugriffskontrolle

Die Nextcloud-Integration bildet die zentrale Plattform.

---

## 2. Local Recorder

Der Local Recorder ist für die Audioaufnahme verantwortlich.

Aufgaben:

- Zugriff auf Mikrofone
- lokale Audioaufnahme
- lokale Zwischenspeicherung
- Chunk-Erzeugung
- Upload nach Abschluss der Aufnahme

Der Recorder benötigt während der Aufnahme keine dauerhafte Netzwerkverbindung.

---

## 3. Session Management

Eine Session beschreibt ein Aufnahmeereignis.

Eine Session enthält:

- Projekt
- Teilnehmer
- Rollen
- Einwilligungen
- Aufnahmestatus
- erzeugte Audiospuren

Beispiel:
Projekt
|
+-- Episode 42
|
+-- Session
|
+-- Host.wav
+-- Gast.wav

---

## 4. Audio Storage

Originalaufnahmen werden unverändert gespeichert.

Eigenschaften:

- offene Formate
- verlustfreie Speicherung möglich
- getrennte Spuren pro Teilnehmer
- Metadaten erhalten

---

## 5. Upload Pipeline

Der Upload erfolgt grundsätzlich:

- nach Ende der Aufnahme
- unabhängig vom Gespräch
- kontrolliert vom Recorder

Während der Aufnahme werden keine Audiodaten übertragen.

---

# Datenfluss

## Aufnahme
Mikrofon

↓

Local Recorder

↓

Lokale Chunks

↓

Finale Audiodatei

↓

Upload

↓

Nextcloud Storage

---

# Rollenmodell

NC-PoRe unterscheidet zwischen:

## Administrator

Serverweite Verwaltung.

## Moderator

Projekt- und Session-Verantwortlicher.

Berechtigungen:

- Sessions erstellen
- Teilnehmer einladen
- Rollen vergeben
- Aufnahmen verwalten

## Benutzer

Regulärer Teilnehmer.

## Editor

Produktionsrolle.

Berechtigungen:

- Zugriff auf Rohmaterial
- Bearbeitung vorbereiten

## Gast

Externer Teilnehmer.

Berechtigungen:

- Session betreten
- eigene Audiospur aufnehmen
- eigene Daten übertragen

---

# Erweiterbarkeit

Die Architektur soll Erweiterungen ermöglichen:

- DAW-Export
- Transkription
- KI-Unterstützung
- Produktionsworkflow
- Enterprise-Funktionen

Erweiterungen müssen den freien Kern respektieren.

---

# Architekturprinzipien

NC-PoRe folgt diesen Prinzipien:

1. Datenhoheit vor Komfort
2. Lokale Verarbeitung vor Cloud-Verarbeitung
3. Offene Formate vor proprietären Formaten
4. Modularität vor monolithischer Architektur
5. Transparenz vor Blackbox-Systemen