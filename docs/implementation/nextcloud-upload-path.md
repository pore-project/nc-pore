# Nextcloud upload path

NC-PoRe uses a host-configurable remote root. The default root is `audio` until the podcast host configures another root.

PoRe owns the structure below that root:

`<root>/YYYY/MM/DD/HH-MM - <display-name> - <stable-id>/`

`YYYY/MM/DD` and `HH-MM` are derived from the recording start timestamp. The display name is supplied by the host/session and sanitized for filesystem/WebDAV use. The stable ID guarantees uniqueness and remains independent of the display name. If no display name is available, the folder is `HH-MM - <stable-id>`.

The root is the only host-configurable path component. Provider-specific WebDAV URL construction remains in the Nextcloud adapter.
