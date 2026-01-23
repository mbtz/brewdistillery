# brewdistillery config schema (v1)

Path: `.distill/config.toml`

This document describes the v1 config schema used by `brewdistillery`.

## Precedence
Resolution order for any configurable field:
- CLI flags (highest)
- `.distill/config.toml`
- Repo autodetect (manifests, git remote)
- Interactive prompts (fallback)

## Compatibility and migration
- `schema_version` is optional today. If omitted, `brewdistillery` treats it as v1.
- Unknown fields are preserved and round-tripped on save. This allows forward-compatible
  upgrades without losing data.
- Future versions may add new keys or sections; older clients should ignore unknown fields.

## Top-level keys
- `schema_version` (int, optional): schema marker. Must be >= 1 when provided.
- `project` (table): CLI metadata.
- `cli` (table): CLI repo identity.
- `tap` (table): tap repo identity and formula location.
- `artifact` (table): release asset strategy and selection rules.
- `release` (table): tag and commit templates.
- `version_update` (table): optional version bump automation.
- `host` (table): host API configuration (GitHub only in v0).
- `template` (table): formula template overrides.

## Section details

### [project]
- `name` (string): CLI display name.
- `description` (string): short description used in formula.
- `homepage` (string): URL for the project.
- `license` (string): SPDX identifier.
- `bin` (array of string): binary names. Must not be empty strings.

### [cli]
- `owner` (string): GitHub owner for the CLI repo.
- `repo` (string): GitHub repo for the CLI repo.
- `remote` (string): git remote URL.
- `path` (string): local path to the CLI repo.

### [tap]
- `owner` (string): GitHub owner for the tap repo.
- `repo` (string): GitHub repo for the tap repo (e.g. `homebrew-foo`).
- `remote` (string): git remote URL for the tap repo.
- `path` (string): local path to the tap repo.
- `formula` (string): formula name (file is `Formula/<formula>.rb`).
- `formula_path` (string): explicit path to the formula file.

### [artifact]
- `owner` (string): GitHub owner hosting release assets (defaults to `cli.owner`).
- `repo` (string): GitHub repo hosting release assets (defaults to `cli.repo`).
- `strategy` (string): `release-asset` or `source-tarball`.
- `asset_template` (string): template for asset names. Example:
  `mycli-{version}-{os}-{arch}.tar.gz`
- `asset_name` (string): explicit asset name override.
- `targets` (table-of-tables): per-target overrides for OS/arch splits.
See `docs/asset-selection.md` for the selection rules and OS/arch normalization.

#### [artifact.targets.<target>]
- `asset_template` (string): target-specific template.
- `asset_name` (string): target-specific asset name override.

### [release]
- `tag_format` (string): tag formatting template (default is plain version).
- `tarball_url_template` (string): template for source tarball URLs.
- `commit_message_template` (string): commit message template for tap updates.

### [version_update]
- `mode` (string): `none`, `cargo`, or `regex`.
- `cargo_package` (string): package name when using cargo workspaces.
- `regex_file` (string): file to update when using regex mode.
- `regex_pattern` (string): regex pattern to match version.
- `regex_replacement` (string): replacement template.

### [host]
- `provider` (string): `github`.
- `api_base` (string): override base API URL (optional).

### [template]
- `path` (string): path to a custom formula template file.
- `install_block` (string): raw Ruby snippet for the formula `install` block.

## Minimal example

```
schema_version = 1

[project]
name = "mycli"
bin = ["mycli"]

[cli]
owner = "acme"
repo = "mycli"

[tap]
owner = "acme"
repo = "homebrew-mycli"
formula = "mycli"

[artifact]
strategy = "release-asset"
asset_template = "mycli-{version}-{os}-{arch}.tar.gz"
```
