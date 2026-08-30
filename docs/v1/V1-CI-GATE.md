# PoRE V1 CI gate

The V1 Talk integration branch is required to pass the dedicated `V1 Talk Connector` workflow before integration review.

The workflow validates:

- browser-script syntax for the Talk connector, recording controller and bootstrap
- workspace compilation with the repository Rust toolchain
- workspace tests
- Rust formatting

The browser runtime tests remain a separate browser/Jest validation concern because the GitHub runner does not provide a Nextcloud Talk runtime.
