# Browser-to-Application Artifact Transport

The browser recorder is a capture client. After local finalization it produces a persistence handoff containing authoritative production/recording identity plus distinct technical capture and recorder-session identities.

The handoff is intentionally independent of WebDAV and persistence implementation details.

## Required runtime chain

```text
Browser capture
  -> authenticated transport
  -> Application BrowserRecordingArtifact
  -> RecordingArtifactProcessor
  -> PersistenceProvider
  -> SynchronizationWork
  -> ArtifactTransfer
  -> Nextcloud
  -> authoritative recording completion
```

The repository currently contains the Rust domain/application/recorder/infrastructure workspace and the Nextcloud PHP host integration, but no Rust HTTP runtime composition root. Consequently this document defines the runtime boundary without pretending that a PHP route or Rust library is already an accepting HTTP service.

## Acceptance criteria

A concrete runtime is complete only when it can demonstrate:

1. authenticated browser submission;
2. decoding and validation of all required identities;
3. construction of `BrowserRecordingArtifact`;
4. persistence through the existing `RecordingArtifactProcessor`;
5. creation/recovery of existing synchronization work;
6. transfer through the existing `ArtifactTransfer` implementation;
7. remote integrity verification;
8. completion through the existing authoritative recording completion boundary.

No direct browser-to-WebDAV path is permitted.
