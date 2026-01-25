# Version update strategies (`version_update`)

`bd release` can optionally update version files in the CLI repo before tagging
and pushing. The default is no version updates.

This document reflects the behavior implemented in `src/version_update.rs`.

## Modes

`version_update.mode` supports three values:
- `none` (default): no files are changed
- `cargo`: update a `Cargo.toml` version
- `regex`: update a file using a regex replacement

## Cargo mode (`mode = "cargo"`)

Cargo mode updates a single manifest based on workspace shape.

Resolution rules:
1. If the root `Cargo.toml` has a `[package]` section:
   - Update `package.version` when `cargo_package` is unset.
   - If `cargo_package` matches the root package name, update the root.
   - When both `[package]` and `[workspace.package]` exist, `[package]` wins by default.
2. Else if the root has `[workspace.package]` and `cargo_package` is unset:
   - Update `workspace.package.version` in the root manifest.
3. Otherwise:
   - `cargo_package` is required.
   - The workspace is scanned to find the unique member package with that
     name, and that member's `package.version` is updated.

Key failure messages (exit code 3):
- `Cargo.toml not found at <path>`
- `version_update.mode=cargo requires version_update.cargo_package for workspaces without [package] or [workspace.package]`
- `Cargo package '<name>' not found in workspace`
- `multiple Cargo packages named '<name>' found: <paths>`

### Example: root package

```toml
[version_update]
mode = "cargo"
```

### Example: workspace member

```toml
[version_update]
mode = "cargo"
cargo_package = "cli"
```

## Regex mode (`mode = "regex"`)

Regex mode updates a single file by matching a pattern and applying a
replacement string.

Required fields:
- `version_update.regex_file`
- `version_update.regex_pattern`
- `version_update.regex_replacement`

All three are required and missing values fail with exit code 3.

### Example

```toml
[version_update]
mode = "regex"
regex_file = "VERSION.txt"
regex_pattern = "VERSION=\\d+\\.\\d+\\.\\d+"
regex_replacement = "VERSION={version}"
```

The replacement can include `{version}`.

## Dry-run behavior

In `--dry-run` mode, the update is computed but not written. When a change
would occur, `bd release` prints:
- `dry-run: would update version in <path>`
