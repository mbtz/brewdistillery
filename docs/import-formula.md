# `bd init --import-formula` merge and overwrite policy

Import mode treats the existing formula as the source of truth for formula
fields while using config/flags/autodetect to fill gaps.

This document reflects the current implementation in `src/commands/init.rs`.

## Goals and invariants

- The formula file is never overwritten in import mode.
- Formula fields win when present.
- Config and flags only fill missing values.
- Non-interactive import fails fast when required fields remain unresolved.

## How the formula is located

Import mode resolves the formula path in this order:
1. `tap.formula_path` from config (if set)
2. `<tap.path>/Formula/<formula>.rb` when the formula name is known
3. If the formula name is empty, scan `<tap.path>/Formula/*.rb`:
   - 0 matches: error
   - 1 match: use it
   - 2+ matches: require `--formula-name`

When `tap.path` cannot be resolved and no `tap.remote` is provided, import
fails with:
- `missing tap path; set --tap-path or --tap-remote to import formula`

## Field-level merge rules

Formula fields are parsed using simple Ruby line matching and quoted string
extraction.

The following fields are imported when present in the formula:

| Field | Formula source | Behavior |
| --- | --- | --- |
| Formula name | filename stem | Must match `--formula-name` when provided |
| Description | `desc "..."` | Overrides resolved value |
| Homepage | `homepage "..."` | Overrides resolved value |
| License | `license "..."` | Overrides resolved value |
| Version | `version "..."` | Normalized via semver rules |
| Binaries | `bin.install ...` | Overrides resolved bins |

If a field is missing in the formula, the resolved value is taken from:
1. Flags
2. Config
3. Repo autodetect
4. Prompts (interactive mode only)

## Name mismatch behavior

If `--formula-name` is provided and it does not match the existing formula file
name, import fails with:
- `formula name '<requested>' does not match existing formula '<existing>'`

This is an exit code 3 error.

## Non-interactive required fields

After importing formula fields, `bd init --non-interactive --import-formula`
requires the following fields to be resolved:
- `formula-name`
- `description`
- `homepage`
- `license`
- `version`
- `bin-name`

If any are missing, the command fails with exit code 2:
- `missing required fields for --import-formula: <comma-separated list>`

## Overwrite behavior

Import mode still writes config and must respect overwrite guards:
- If config differs and `--force/--yes` is not set, fail with:
  - `config already exists at <path>; re-run with --force or --yes to overwrite`

Because the formula file is not rewritten in import mode, formula overwrite
checks do not apply.

## Preview behavior

Import mode still previews changes before writing:
- Only the CLI repo is listed in the preview summary.
- The preview diff covers `.distill/config.toml` only.
