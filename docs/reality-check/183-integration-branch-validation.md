# Reality Check 183: Integration Branch Validation

## Purpose

Validate the combined recorder/filesystem and Nextcloud/WebDAV implementation before synchronizing the developer workstation.

## Boundary

The validation is deterministic and must not require production credentials or access to the Home Nextcloud instance.

## Success criteria

- workspace formatting and compilation succeed;
- workspace tests succeed;
- architecture checks succeed;
- recorder filesystem boundary remains available;
- Nextcloud transfer contract tests remain deterministic;
- no credentials are committed.

The live Home-Nextcloud `Pore-Test` check is a separate infrastructure test after this gate is green.
