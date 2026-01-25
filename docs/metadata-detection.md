# Repository metadata detection

This document describes how `brewdistillery` derives project metadata from
common manifest files. The detector uses a best-effort, first-match strategy
and leaves missing fields unset so the init flow can prompt or require flags.

## Precedence and conflict handling

The detector scans the repository root for all supported manifests:

1) `Cargo.toml`
2) `package.json`
3) `pyproject.toml`
4) `go.mod`

If none of these are present or parseable, metadata detection falls back to
git remote inference (name + homepage). If the git remote is missing or
non-GitHub, metadata detection returns `None` for all fields and the init
flow must prompt or require flags.

Conflict resolution:
- When multiple manifests exist, the detector attempts to merge fields.
- If two sources provide different non-empty values for the same field,
  the detector records a conflict and leaves the field unset.
- Conflicts are surfaced to callers so interactive flows can prompt.
- Non-interactive commands treat conflicts as errors (exit code 3) with
  guidance to set explicit values via flags or config.

## Shared behavior

- Binaries: bin names are normalized by trimming, removing empty values,
  sorting, and de-duplicating.
- License fallback: if a manifest does not provide a license, the detector
  checks for well-known license filenames and infers SPDX values from the
  filename (see License detection).
- Homepage fallback: if no manifest provides a homepage, the detector uses the
  GitHub remote URL (if available).
- Name fallback: if no manifest provides a name, the detector uses the repo
  name from the GitHub remote (if available).
- Partial results: any field may be missing; callers should treat missing
  values as prompts in interactive mode and required flags in non-interactive
  mode.

## Field mapping summary

| Manifest | Name | Description | Homepage | License | Version | Bin detection |
| --- | --- | --- | --- | --- | --- | --- |
| `Cargo.toml` | `package.name` or `workspace.package.name` | `package.description` | `package.homepage` | `package.license` | `package.version` | `[[bin]].name` entries; fallback to `package.name` |
| `package.json` | `name` (unscoped) | `description` | `homepage` | `license` | `version` | `bin` string -> name; `bin` object -> keys |
| `pyproject.toml` `[project]` | `project.name` | `project.description` | `project.urls` (Homepage/home/Repository) | `project.license` | `project.version` | `[project.scripts]` keys |
| `pyproject.toml` `[tool.poetry]` | `tool.poetry.name` | `tool.poetry.description` | `tool.poetry.homepage` | `tool.poetry.license` | `tool.poetry.version` | `[tool.poetry.scripts]` keys |
| `go.mod` | module path basename | (none) | (none) | (none) | (none) | module name |

Missing fields are left unset for prompts or required flags.

## Rust (Cargo)

Source file: `Cargo.toml` at repo root.

Fields:
- `project.name`: `package.name` (or `workspace.package.name`).
- `project.description`: `package.description`.
- `project.homepage`: `package.homepage`.
- `project.license`: `package.license`.
- `project.version`: `package.version`.
- `project.bin`: `[[bin]].name` entries.
  - If no `[[bin]]` entries exist, default to `package.name`.

Workspace handling:
- If `[package]` is absent, the detector falls back to `[workspace.package]`.
- The detector does not scan workspace members; only the root `Cargo.toml`
  is consulted.

Edge cases:
- If both `[package]` and `[workspace.package]` are missing, Cargo metadata
  is treated as absent and detection continues with other manifests.
- If `package.name` is missing, bin defaulting does not occur and the bin list
  remains empty.

## Node (package.json)

Source file: `package.json` at repo root.

Fields:
- `project.name`: `name` (may include scope, e.g. `@acme/tool`).
- `project.description`: `description`.
- `project.homepage`: `homepage`.
- `project.license`: `license`.
- `project.version`: `version`.
- `project.bin`:
  - If `bin` is a string, use the unscoped package name (e.g. `@acme/tool`
    -> `tool`).
  - If `bin` is an object, use the keys as bin names.
  - If `bin` is missing, the bin list is left empty.

## Python (pyproject.toml)

Source file: `pyproject.toml` at repo root.

The detector first checks `[project]` (PEP 621). If missing, it then checks
`[tool.poetry]`.

### [project]

Fields:
- `project.name`: `project.name`.
- `project.description`: `project.description`.
- `project.homepage`: derived from `project.urls` (first match):
  `Homepage`, `homepage`, `Home`, `home`, `Repository`.
- `project.license`:
  - `project.license` string, or
  - `project.license.text` / `project.license.file` if provided as a table.
- `project.version`: `project.version`.
- `project.bin`: keys from `[project.scripts]`.

### [tool.poetry]

Fields:
- `project.name`: `tool.poetry.name`.
- `project.description`: `tool.poetry.description`.
- `project.homepage`: `tool.poetry.homepage`.
- `project.license`: `tool.poetry.license`.
- `project.version`: `tool.poetry.version`.
- `project.bin`: keys from `[tool.poetry.scripts]`.

## Go (go.mod)

Source file: `go.mod` at repo root.

Fields:
- `project.name`: last path segment of the `module` declaration.
- `project.bin`: defaults to the derived module name.
- Other fields (description, homepage, license, version) are left unset.

If the `module` line is missing or empty, Go metadata detection is treated as
absent.

## Conflict resolution examples

- `Cargo.toml` has `package.name = "brewtool"` and `package.json` has
  `name = "other"`: `name` is treated as conflicting and left unset.
- `package.json` defines `bin = { "brewtool": "...", "brewctl": "..." }`
  while `pyproject.toml` defines `project.scripts = { "brewtool": "..." }`:
  the bin list is treated as conflicting and left unset.

In `bd init` interactive mode, these conflicts are shown as hints and the
user is prompted to provide explicit values. In non-interactive mode, the
command fails fast with a conflict error.

## License detection (fallback)

If a manifest does not provide a license, the detector checks the repo root
for these filenames (first match wins unless multiple conflicting matches
are found):

- `LICENSE`, `LICENSE.txt`, `LICENSE.md`
- `COPYING`, `COPYING.txt`, `COPYING.md`
- `LICENSE-MIT`
- `LICENSE-APACHE`, `LICENSE-APACHE-2.0`
- `LICENSE-BSD`, `LICENSE-BSD-3-CLAUSE`
- `LICENSE-GPL`, `LICENSE-GPL-3.0`

Filename-to-SPDX mapping:
- Contains `MIT` -> `MIT`
- Contains `APACHE` -> `Apache-2.0`
- Contains `BSD-3` or `BSD3` -> `BSD-3-Clause`
- Contains `BSD` -> `BSD-2-Clause`
- Contains `GPL-3` -> `GPL-3.0-only`
- Contains `GPL` -> `GPL-2.0-only`

If conflicting license filenames are found (e.g. both MIT and Apache),
license detection returns `None` and the init flow should prompt.

## Edge cases

- Cargo workspaces: only the root `Cargo.toml` is parsed; workspace member
  manifests are ignored.
- Multi-bin projects: multiple `[[bin]]` entries (Cargo) or `bin` objects
  (Node) produce a normalized, de-duplicated bin list.
- Missing license/homepage: if not provided by any manifest, the detector
  attempts a license filename match and a GitHub remote homepage fallback.

## Notes for prompting

- Always surface the detected metadata along with the source (Cargo/Node/etc).
- If the bin list is empty, require at least one `--bin-name` or prompt for it.
- If name/description/homepage/license/version are missing, treat them as
  required fields in non-interactive mode.
