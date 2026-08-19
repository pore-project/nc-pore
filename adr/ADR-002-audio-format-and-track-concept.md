# Deutsch ([English version below](#english-version))

# ADR-002: Audioformat und Spurkonzept

## Status

Angenommen

## Datum

2026-07-22

---

# Kontext

NC-PoRe ist nicht nur ein Aufnahmewerkzeug, sondern eine Podcast-Produktionsplattform.

Für professionelle Nachbearbeitung ist es notwendig, die einzelnen Teilnehmer getrennt bearbeiten zu können.

Eine während der Aufnahme erzeugte Mischung würde diese Möglichkeiten einschränken:

- individuelle Lautstärkeanpassung wäre schwieriger
- Störgeräusche könnten nicht gezielt entfernt werden
- unterschiedliche Bearbeitung einzelner Stimmen wäre nicht möglich
- spätere Produktionsschritte wären eingeschränkt

Darüber hinaus entstehen bei einer browserbasierten Aufnahme zwei unterschiedliche Anforderungen an die Audioqualität:

- Während der Produktion kann eine möglichst hohe bzw. verlustfreie Qualität für die weitere Bearbeitung gewünscht sein.
- Für die dauerhafte Archivierung kann ein geringeres Datenvolumen ausreichend sein.

Eine Architektur, die ausschließlich ein festes Audioformat vorgibt, würde diese unterschiedlichen Anforderungen unnötig miteinander verknüpfen.

---

# Entscheidung

NC-PoRe speichert Aufnahmen grundsätzlich als getrennte Monospuren pro Teilnehmer.

Jeder Teilnehmer erzeugt eine eigene Audiospur.

Beispiel:

Episode_042/

audio/

host.wav

gast.wav

cohost.wav

metadata.json

Die Aufnahme wird nicht zu einer gemeinsamen Audiodatei gemischt.

Die konkrete technische Audio-Repräsentation wird nicht als unveränderliches Projektformat festgelegt. Stattdessen verwendet eine Session ein konfigurierbares **Audio-Qualitätsprofil**.

Das Audio-Qualitätsprofil beschreibt die gewünschte tatsächliche Audioqualität und das damit verbundene Datenvolumen. Es ist die fachliche Vorgabe für die technische Auswahl geeigneter Audio-Repräsentationen.

Das Profil berücksichtigt mindestens zwei Ebenen:

- **Produktions-/Transportqualität:** Die Qualität, die während der Session und bei der Übertragung vom Browser zum PoRe-Server erhalten werden soll.
- **Archiv-/Persistenzqualität:** Die Qualität, die dauerhaft gespeichert werden soll.

Der Besitzer bzw. Betreiber der Session kann das Audio-Qualitätsprofil konfigurieren. Für Sessions ohne explizite Konfiguration wird ein projektseitig definierter Default verwendet.

Transport- und Persistenzformat müssen nicht identisch sein. Die Architektur soll unterschiedliche Repräsentationen ermöglichen, wenn dies technisch sinnvoll ist.

Gleichzeitig soll Audio nicht unnötig mit höherer Qualität bzw. größerem Datenvolumen übertragen werden als für das gewählte Produktions-/Transportprofil erforderlich ist. Wenn die gewünschte Produktionsqualität bereits verlustbehaftet ist, soll keine unnötige verlustfreie Übertragung erfolgen, nur um das Material anschließend auf die gewünschte Archivqualität zu reduzieren.

Das Archivprofil darf keine höhere tatsächliche Quellqualität voraussetzen, als durch die Produktions-/Transportstufe erhalten wurde.

Eine spätere Speicherung in einem technisch höherwertigen oder unkomprimierten Format stellt durch eine frühere verlustbehaftete Kodierung entfernte Informationen nicht wieder her. Beispielsweise kann ein verlustbehaftet kodiertes Signal zwar als PCM in einem WAV-Container gespeichert werden, die resultierende Audioqualität bleibt jedoch durch die vorherige verlustbehaftete Kodierung begrenzt.

Konkrete Codecs und Container werden durch diese ADR nicht festgelegt. Geeignete Verfahren werden in einem späteren technischen Arbeitsschritt anhand insbesondere folgender Kriterien bewertet:

- Audioqualität
- Datenvolumen bei Transport und Speicherung
- Browser-Unterstützung
- Encoding- und Decoding-Aufwand
- CPU- und Speicherbedarf
- Eignung für Streaming bzw. chunkweise Verarbeitung
- Eignung für weitere Bearbeitung
- Lizenz- und Patentsituation
- Verfügbarkeit geeigneter Implementierungen

Damit bleibt die Architektur offen für verlustfreie und verlustbehaftete Verfahren sowie für unterschiedliche technische Formate. Beispiele wie FLAC, MP3, Opus oder Vorbis stellen keine Vorfestlegung dar.

Die aktuelle Implementierungsphase betrachtet ausschließlich die browserbasierte Teilnahme. Dedizierte oder native Clients sind nicht Bestandteil dieser Phase.

---

# Konsequenzen

## Positive Auswirkungen

- getrennte Bearbeitung der Teilnehmer bleibt möglich
- Produktionsqualität und Archivbedarf können unabhängig betrachtet werden
- unnötige Übertragungsdaten können vermieden werden
- der Betreiber kann Qualität und Datenvolumen an den konkreten Anwendungsfall anpassen
- konkrete Codec-Entscheidungen können anhand technischer Erkenntnisse getroffen werden
- die Architektur bleibt offen für geeignete lizenzfreie bzw. offene Verfahren
- keine Qualitätsverluste durch Vorabmischung

---

## Negative Auswirkungen

- zusätzliche Konfiguration der Audioqualität
- mehrere mögliche Audio-Repräsentationen müssen technisch unterstützt bzw. konvertiert werden
- Transport und Persistenz müssen hinsichtlich ihrer Qualitätsgrenzen korrekt aufeinander abgestimmt werden
- Codec-Auswahl und Browser-Unterstützung müssen vor der konkreten Implementierung untersucht werden

---

# Betrachtete Alternativen

## Festes Masterformat für alle Sessions

Verworfen.

Ein fest vorgegebenes Format würde Transport, Produktion und Archivierung unnötig koppeln und könnte bei unterschiedlichen Anforderungen zu unnötigem Datenvolumen führen.

---

## Verlustfreie Übertragung unabhängig vom gewählten Qualitätsprofil

Verworfen.

Eine grundsätzlich verlustfreie Übertragung wäre zwar qualitativ flexibel, würde aber unnötiges Datenvolumen verursachen, wenn der Betreiber ausdrücklich eine geringere verlustbehaftete Produktionsqualität gewählt hat.

---

## Gemeinsamer Stereo-Mix während der Aufnahme

Verworfen.

Gründe:

- keine individuelle Bearbeitung möglich
- Fehler sind dauerhaft eingebrannt
- entspricht nicht professionellen Produktionsabläufen

---

## Mehrkanal-WAV mit allen Teilnehmern

Nicht als primäres Format gewählt.

Begründung:

Mehrkanal-WAV kann technisch mehrere Spuren enthalten,
aber einzelne Monodateien bieten:

- bessere Kompatibilität
- einfachere Archivierung
- bessere Zusammenarbeit zwischen verschiedenen DAWs

---

# Hinweise

Die Trennung von Aufnahme, Transport und Produktion ist ein Grundprinzip von NC-PoRe.

NC-PoRe erzeugt Rohmaterial, dessen gewünschte Qualitätsstufe durch das Audio-Qualitätsprofil bestimmt wird.

Die kreative Bearbeitung erfolgt in spezialisierten Produktionswerkzeugen.

Das Audio-Qualitätsprofil ist eine fachliche Vorgabe. Die konkrete Auswahl und Implementierung von Codecs und Containern ist davon getrennt und wird technisch untersucht.

---

# English Version ([Deutsche Version oben](#deutsch))

# ADR-002: Audio Format and Track Concept

## Status

Accepted

## Date

2026-07-22

---

# Context

NC-PoRe is not only a recording tool but a podcast production platform.

For professional post-production, it is necessary to be able to process each participant separately.

A mix created during recording would limit these possibilities:

- individual volume adjustment would be more difficult
- unwanted noise could not be removed selectively
- different processing of individual voices would not be possible
- later production steps would be constrained

Browser-based recording also creates two distinct requirements for audio quality:

- During production, high or lossless quality may be desirable for further processing.
- For long-term archiving, a lower data volume may be sufficient.

An architecture based on one fixed audio format would unnecessarily couple these different requirements.

---

# Decision

NC-PoRe stores recordings as separate mono tracks per participant.

Each participant produces a separate audio track.

Example:

Episode_042/

audio/

host.wav

gast.wav

cohost.wav

metadata.json

The recording is not mixed into a single audio file.

The concrete technical audio representation is not fixed as an immutable project-wide format. Instead, a session uses a configurable **audio quality profile**.

The audio quality profile describes the desired actual audio quality and the associated data volume. It is the functional requirement used to select suitable technical audio representations.

The profile covers at least two levels:

- **Production/transport quality:** The quality that should be preserved during the session and when transferring audio from the browser to the PoRe server.
- **Archive/persistence quality:** The quality that should be retained for permanent storage.

The session owner or operator can configure the audio quality profile. A project-defined default is used when no explicit configuration is provided.

Transport and persistence formats do not have to be identical. The architecture shall allow different representations where this is technically useful.

At the same time, audio should not be transferred at a higher quality or with a larger data volume than required by the selected production/transport profile. If the desired production quality is already lossy, lossless transfer should not be performed unnecessarily only to reduce the material to the desired archive quality afterwards.

The archive profile must not require a higher actual source quality than was preserved by the production/transport stage.

Storing a previously lossy-encoded signal later in a technically higher-grade or uncompressed format does not restore information removed by the earlier lossy encoding. For example, a lossy-encoded signal can technically be stored as PCM in a WAV container, but the resulting audio quality remains limited by the previous lossy encoding.

This ADR does not prescribe specific codecs or containers. Suitable approaches will be evaluated in a later technical step using criteria including:

- audio quality
- data volume for transport and storage
- browser support
- encoding and decoding effort
- CPU and memory requirements
- suitability for streaming or chunked processing
- suitability for further processing
- licensing and patent situation
- availability of suitable implementations

This keeps the architecture open to lossless and lossy approaches as well as different technical formats. Examples such as FLAC, MP3, Opus, or Vorbis are not a commitment to any particular implementation.

The current implementation phase considers browser-based participation only. Dedicated or native clients are not part of this phase.

---

# Consequences

## Positive Effects

- separate processing of participants remains possible
- production quality and archive requirements can be considered independently
- unnecessary transfer data can be avoided
- the operator can adapt quality and data volume to the specific use case
- concrete codec decisions can be made based on technical findings
- the architecture remains open to suitable royalty-free or open approaches
- no quality loss caused by pre-mixing

---

## Negative Effects

- additional audio-quality configuration
- multiple possible audio representations may need to be supported or converted
- transport and persistence must be aligned correctly with regard to their quality limits
- codec selection and browser support must be investigated before concrete implementation

---

# Alternatives Considered

## Fixed Master Format for All Sessions

Rejected.

A fixed format would unnecessarily couple transport, production, and archiving and could cause unnecessary data volume for different requirements.

---

## Lossless Transfer Regardless of the Selected Quality Profile

Rejected.

Lossless transfer would provide maximum quality flexibility, but it would cause unnecessary data volume when the operator has explicitly selected a lower lossy production quality.

---

## Common Stereo Mix During Recording

Rejected.

Reasons:

- no individual processing possible
- errors are permanently baked in
- does not correspond to professional production workflows

---

## Multichannel WAV with All Participants

Not selected as the primary format.

Rationale:

Multichannel WAV can technically contain multiple tracks,
but individual mono files provide:

- better compatibility
- simpler archiving
- better collaboration between different DAWs

---

# Notes

The separation of recording, transport, and production is a fundamental principle of NC-PoRe.

NC-PoRe produces raw material whose desired quality level is determined by the audio quality profile.

Creative editing takes place in specialized production tools.

The audio quality profile is a functional requirement. The concrete selection and implementation of codecs and containers is separate from that requirement and will be investigated technically.
