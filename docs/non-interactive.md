# Non-interactive contract

This document defines the required inputs, autodetect sources, and failure
behavior for `--non-interactive` usage. It is intended as the source of truth
for CLI validation and CI-friendly error messaging.

## Precedence

Inputs resolve in this order:
1) Flags
2) Config file (`.distill/config.toml` unless overridden)
3) Repo autodetect

If a required field cannot be resolved after this precedence chain, the command
must fail without writing files or making network requests.

## Error model

- Missing required inputs: exit code 2 (`AppError::MissingConfig`).
- Invalid input values (naming, semver, license): exit code 3.
- Git state failures: exit code 4.
- Network failures: exit code 5.
- Audit failures (`bd doctor --audit`): exit code 6.

### Canonical error messages

- Missing fields (init/release):
  `missing required fields for --non-interactive: <comma-separated list>`
- Missing config for doctor:
  `config not found at <path>`

## `bd init --non-interactive`

### Required fields (if not resolved by config or autodetect)

- CLI repo identity:
  - `--host-owner` + `--host-repo`, or
  - a detectable GitHub remote (origin/upstream) that resolves to owner/repo.
- Tap identity:
  - `--tap-path`, or
  - `--tap-remote`, or
  - `--tap-owner` + `--tap-repo`.
- Formula fields:
  - `--formula-name` (defaults to repo name if detected)
  - `--description`
  - `--homepage`
  - `--license`
  - `--version`
- Binary names:
  - `--bin-name` (one or more)

### Autodetect sources

- Project metadata from supported manifests (`Cargo.toml`, `package.json`,
  `pyproject.toml`, `go.mod`).
- License fallback from `LICENSE*` filenames (see `docs/metadata-detection.md`).
- Repo name/remote from git remotes in the CLI repo.

### Overwrite policy

- Existing config/formula changes require `--force` or `--yes` to overwrite.
- Without `--force/--yes`, `bd init` must fail before writing.

## `bd release --non-interactive`

### Required fields (if not resolved by config or autodetect)

- Tap repo path or remote:
  - `tap.path` or `tap.remote`, or
  - `--tap-path` / `--tap-remote` override.
- Formula identity:
  - `tap.formula` or `tap.formula_path`.
- Artifact strategy and selection:
  - `artifact.strategy` (`release-asset` or `source-tarball`),
  - `artifact.asset_name` or `artifact.asset_template` (or per-target override
    when OS/arch matrix is enabled).
- Version input:
  - `--version` or `--tag`, unless latest GitHub Release tag can be resolved
    (stable only unless `--include-prerelease`).
  See `docs/release-discovery.md` for the discovery and prerelease rules.

### Autodetect sources

- Config file `.distill/config.toml` (primary).
- GitHub Release API for version/tag discovery (stable by default).

### Failure behavior

- If any required field is missing, exit with:
  `missing required fields for --non-interactive: <fields>` (exit code 2).
- If release assets cannot be resolved, exit with:
  `no release assets match target <...>; available assets: ...` (exit code 3).

## `bd doctor --non-interactive`

### Required fields

- Config file present and valid (`.distill/config.toml` or `--config`).

### Failure behavior

- Missing config: `config not found at <path>` (exit code 2).
- Invalid config: `invalid config at <path>: <details>` (exit code 3).
- With `--strict`, warnings are treated as errors.

## Notes for CI usage

- Always pass explicit overrides for any field that might be ambiguous when
  multiple remotes or manifests exist.
- Use `--dry-run` to validate inputs without writing or pushing.
