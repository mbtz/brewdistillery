# Formula version handling

`brewdistillery` always renders an explicit `version` stanza and uses it for
idempotency checks during release.

This document reflects current behavior in `src/formula.rs`,
`src/version.rs`, and `src/commands/release.rs`.

## Rendering rules

- Every rendered formula includes:
  - `version "<normalized-version>"`
- The version string is the normalized semver value (no leading `v`).

## Version normalization and validation

Version inputs must be valid semver (prerelease/build metadata allowed).

Normalization rules:
- `--tag v1.2.3` normalizes to version `1.2.3`.
- If both `--version` and `--tag` are set, they must match after normalization.

Key error messages (exit code 3):
- `invalid --version '<value>': expected semver (e.g. 1.2.3)`
- `invalid --tag '<value>': expected semver (e.g. 1.2.3)`
- `--version '<version>' does not match --tag '<tag>'`

## Release idempotency check

Before rendering, `bd release` checks the existing formula's `version` stanza:
- If the existing version matches the requested version and `--force` is not
  set, release fails with exit code 3:
  - `formula already targets version <version>; re-run with --force to re-apply`

The existing version is read by scanning the formula file for the first
`version "..."` line.

## Import mode interaction

In `bd init --import-formula`, the existing formula's `version` is imported
when present and normalized using the same semver rules.
