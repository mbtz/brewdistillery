# `bd doctor` checks and failure modes

This document enumerates the checks performed by `bd doctor`, how warnings are
handled, and how `--audit` integrates with Homebrew.

It matches the implementation in `src/commands/doctor.rs`.

## Entry conditions

`bd doctor` requires `.distill/config.toml` to exist. If it does not, the
command fails with exit code 2:
- `config not found at <path>`

## Check categories

`bd doctor` collects issues as either errors or warnings.

### Required config fields (errors)

The following fields must be present and non-empty:
- `project.name`
- `project.description`
- `project.homepage`
- `project.license`
- `project.bin` (must not be empty)
- `cli.owner`
- `cli.repo`

### Tap identity and formula resolution

Tap identity must be resolvable via one of:
- `tap.path`
- `tap.remote`
- `tap.owner` + `tap.repo`

Failure is an error:
- `tap identity is missing (set tap.path, tap.remote, or tap.owner+tap.repo)`

Additional tap checks:
- If `tap.path` (or `--tap-path`) is set but does not exist, it is an error.
- `tap.formula` must be set; otherwise it is an error.

Formula path resolution uses:
1. `tap.formula_path` when set
2. `<tap.path>/Formula/<tap.formula>.rb` when `tap.path` is available

If the formula path cannot be resolved, doctor emits a warning:
- `formula path cannot be resolved without tap.path or tap.formula_path`

### Formula content checks (errors)

When the formula path resolves and the file exists, doctor validates:
- Class name matches the expected class derived from `tap.formula`
- Required stanzas exist:
  - `desc`
  - `homepage`
  - `url`
  - `sha256`
  - `license`
  - `version`

Representative error messages:
- `formula class name '<found>' does not match expected '<expected>'`
- `formula is missing sha256`

### Artifact configuration (warnings)

Artifact configuration is advisory in doctor mode:
- Missing `artifact.strategy` is a warning.
- Missing both `artifact.asset_template` and `artifact.asset_name` is a warning.

## Warnings vs errors and `--strict`

Default behavior:
- Errors cause exit code 3.
- Warnings are printed but exit 0.

With `--strict`:
- Warnings are treated as errors (exit code 3).

Output formats:
- Errors: `doctor found issues` followed by bullet list.
- Warnings: `doctor warnings` followed by bullet list.
- Clean: `doctor: ok`

## `--audit` integration

When `--audit` is set, `bd doctor` runs:
- `brew audit --new-formula <formula_path>`

Audit-specific failures use exit code 6:
- `brew audit requires a resolved formula path`
- `failed to run brew audit: <err>`
- `brew audit failed` (stdout/stderr appended when present)

On success, doctor prints:
- `brew audit: ok`
