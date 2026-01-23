# Tap repo path resolution and auto-clone

This document defines how `brewdistillery` resolves the tap repository path
and when it auto-clones a tap repo during release.

## Inputs

Tap repo location can be provided by:
- CLI flags: `--tap-path`, `--tap-remote`, `--tap-owner`, `--tap-repo`
- Config fields: `tap.path`, `tap.remote`, `tap.owner`, `tap.repo`,
  `tap.formula`, `tap.formula_path`

## Path resolution rules

### `bd init`

- Non-interactive mode requires one of:
  - `--tap-path`, or
  - `--tap-remote`, or
  - `--tap-owner` + `--tap-repo`
- If only owner/repo are provided, `bd init` records them in config and does
  not assume a local path unless `--tap-path` is specified.

### `bd release`

Resolve the tap root in this order:
1) `--tap-path` (explicit override)
2) `tap.path` from config
3) `tap.formula_path` when absolute (tap root is derived from the formula path)
4) `tap.formula_path` when relative (tap root is the current working directory)
5) `tap.remote` (auto-clone to a temp directory)

If none of the above are available, release fails with:
`missing required fields for --non-interactive: tap.path or tap.remote`
(exit code 2).

## Auto-clone behavior (release)

When `tap.remote` is set and no tap path can be resolved:
- Clone the tap repo into a temp directory prefixed with
  `brewdistillery-tap-`.
- The clone destination is `<temp>/tap`.
- Emit a message:
  `release: cloned tap repo from <remote> to <path>`.

## Cleanup

- Temp clones are deleted automatically when the release run completes,
  whether it succeeds or fails.
- If the process is interrupted, the temp directory may remain; it is safe
  to delete manually.
