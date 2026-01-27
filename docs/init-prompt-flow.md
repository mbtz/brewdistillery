# `bd init` prompt flow and mapping

This document describes the interactive prompt sequence for `bd init`, the
source of defaults for each field, and the overwrite/confirmation rules.

It is aligned to the current implementation in `src/commands/init.rs`.

## Precedence for defaults

Interactive defaults resolve using the same precedence chain as the rest of the
CLI:
1. Flags
2. Existing config (`.distill/config.toml`)
3. Repo autodetect (`Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`)

Prompts are used when a value is still missing or when the user wants to edit a
default.

## Interactive flow (`bd init`)

The interactive flow prompts in this order:
1. Formula name
2. Description
3. Homepage
4. License (SPDX)
5. Version
6. Binary name(s)
7. GitHub owner
8. GitHub repo
9. Tap path
10. Tap owner (when `--tap-new` or `--create-tap` is set)
11. Tap repo (when `--tap-new` or `--create-tap` is set)
12. Artifact strategy (when not supplied via flags or config)

Optional inputs are resolved but not currently prompted unless needed:
- `tap.remote`, `tap.owner`, `tap.repo`
- `artifact.asset_template`
- Template overrides (`template.path`, `template.install_block`)

### Field defaults and behaviors

- Formula name:
  - Default: `--formula-name`, then `tap.formula`, then `--repo-name` or
    detected repo name.
  - Validation uses the formula normalization rules documented in
    `docs/formula-naming.md`.
- Description/homepage/license/version:
  - Defaults come from flags, config, then detected metadata.
  - These prompts require non-empty values.
- Binary name(s):
  - Default uses detected bins, then `tap.formula`, then the formula name.
  - Multiple binaries are supported via comma-separated input.
- GitHub owner/repo:
  - Defaults prefer explicit flags/config.
  - If missing, they are inferred from the homepage when it points to GitHub.
- Tap path:
  - Default repo name: `tap.repo` or `homebrew-<project_name>`.
  - Default path: sibling of the CLI repo (`../homebrew-<project_name>`).

## Import flow (`bd init --import-formula`)

When `--import-formula` is set without `--non-interactive`, the flow changes:
- GitHub owner/repo and tap location are resolved first.
- The existing formula is loaded and becomes the source of truth for formula
  fields (`desc`, `homepage`, `license`, `version`, and `bin.install`).
- Prompts are only shown for missing fields.
- The formula file is never overwritten in import mode; only config is updated.

See `docs/import-formula.md` for the field-level merge policy.

## Preview and confirmation

After resolution, `bd init` always shows a preview before any writes:
- A repo summary (which repos will change).
- A unified diff when files already exist.

Example preview shape:

```text
Repo: cli (/path/to/cli)
  - .distill/config.toml (modified)
Repo: tap (/path/to/homebrew-tool)
  - Formula/tool.rb (modified)
```

If there are no changes, `bd init` prints:
- `init: no changes to apply`

If `--dry-run` is set, it also prints:
- `dry-run: no changes applied`

## Overwrite and confirmation rules

Interactive mode defaults to safe behavior:
- When `--force` or `--yes` is not set, the user must confirm:
  - Prompt: `Apply these changes?`
  - Default answer: yes
- If the user declines, the command exits cleanly with:
  - `init: cancelled`

Non-interactive overwrite guards are enforced earlier and fail fast with clear
messages:
- Config overwrite blocked:
  - `config already exists at <path>; re-run with --force or --yes to overwrite`
- Formula overwrite blocked:
  - `formula already exists at <path>; re-run with --force or --yes to overwrite`

Both errors exit with code 3.

## `--tap-new` behavior and fallback

When `--tap-new` is provided, `bd init` uses Homebrew to scaffold the tap:
- Required inputs:
  - `tap-owner`
  - `tap-repo`
- It resolves the Homebrew tap directory via `brew --repo` and uses:
  - `<brew-repo>/Library/Taps/<owner>/<repo>`

Failure behaviors are explicit:
- If Homebrew is missing or cannot be executed:
  - `brew tap-new requires Homebrew; failed to run brew --repo: <err>`
- If `brew --repo` fails:
  - `brew --repo failed` (with stdout/stderr appended when present)
- If the tap path exists but is not a directory:
  - `tap path exists but is not a directory: <path>`
- If `brew tap-new` fails:
  - `brew tap-new <owner>/<repo> failed` (with stdout/stderr appended)

All `--tap-new` failures exit with code 3.

## `--create-tap` behavior and fallback

When `--create-tap` is provided, `bd init` creates the tap repository on
GitHub and then reuses the normal tap remote + clone behavior.

Key rules:
- `--create-tap` cannot be combined with `--tap-new`.
- `--create-tap` cannot be combined with an existing `tap.remote`.
- Non-interactive mode requires `tap-owner` and `tap-repo`.

Dry-run behavior:
- Prints: `dry-run: would create tap repo <owner>/<repo> on GitHub`
- Sets `tap.remote` to `https://github.com/<owner>/<repo>.git` without making
  a network request.

Common failures:
- `--create-tap cannot be used with --tap-new`
- `--create-tap cannot be used with --tap-remote`
- `missing required fields for --create-tap: tap-owner, tap-repo`
- `GitHub token missing; set GITHUB_TOKEN or GH_TOKEN to create the tap repo`

See `docs/github-tap-creation.md` for the full API flow details.
