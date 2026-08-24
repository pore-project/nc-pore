# Milestone: Nextcloud and Recorder Integration

## Status

In progress — integration branch created.

## Scope

This milestone combines the previously parallel recorder/filesystem reality-check work with the Nextcloud/WebDAV transfer work on a common integration branch.

The integration preserves:

- real CPAL input-device discovery and RODECaster probing;
- filesystem-backed RecordingArtifact persistence and its boundary test;
- ADR-070 for host-configurable delivery format with FLAC as the default when unspecified;
- provider-neutral transfer metadata;
- deterministic Nextcloud/WebDAV artifact-transfer tests;
- runtime-only Nextcloud credentials.

## Next validation

CI must validate the combined branch before the working copy is synchronized. The real Home-Nextcloud test against `Pore-Test` remains a separate infrastructure reality check and is not part of ordinary CI.
