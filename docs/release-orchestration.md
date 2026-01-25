# Release orchestration and idempotency (v0)

This document defines the release flow implemented by `bd release` and the
canonical error behavior for common failure paths.

## Step-by-step flow

`bd release` executes the following stages in order:

1. Validate that `.distill/config.toml` exists.
2. Resolve the tap repo root.
3. Resolve required config fields for non-interactive release.
4. Resolve version and tag inputs.
5. Discover the GitHub Release when using `release-asset` strategy (skipped in `--dry-run`).
6. Select the release asset(s) and compute SHA256 values (dry-run derives asset URLs locally and uses `SHA256=DRY-RUN`).
7. Enforce idempotency checks against the existing formula.
8. Enforce clean git working trees unless `--allow-dirty` is set.
9. Apply optional version update automation in the CLI repo.
10. Render the formula.
11. Preview the change and apply it atomically.
12. Commit and push the tap repo update (unless `--no-push`).
13. Create and push the tag in the CLI repo (unless `--skip-tag` or `--no-push`).

Notes:
- In `--dry-run` mode, the command performs no network calls (no tap clone,
  no GitHub API requests, and no downloads).
- Dry-run requires an explicit `--version` or `--tag`.
- Dry-run also requires `tap.path` or an absolute `tap.formula_path` when
  `tap.remote` is configured, because remotes are not auto-cloned.

## Tap root resolution rules

Tap root resolution follows this order:

1. `--tap-path`
2. `tap.path`
3. If `tap.formula_path` is absolute, derive the tap root from it.
4. If a tap remote URL is available and this is not a dry-run, auto-clone into
   a temp dir.
5. In `--dry-run` mode, `tap.remote` is never cloned; require `tap.path` or an
   absolute `tap.formula_path`.
6. Otherwise, continue without a tap root and fail during required-field
   validation.

## Idempotency and overwrite matrix

Scenario | Behavior | Exit code
---|---|---
Formula already targets requested version and `--force` not set | Fail fast with guidance | 3
Tag already exists and `--skip-tag` not set | Fail fast with guidance | 4
Tap path missing but tap remote is configured (non-dry-run) | Auto-clone and continue | 0
Tap path missing but tap remote is configured (dry-run) | Fail requiring local tap path or absolute formula path | 2
Tap path missing and no tap remote configured | Fail during required-field validation | 2
No matching release asset(s) | Fail with actionable asset-selection error | 3

Canonical idempotency messages:
- `formula already targets version <version>; re-run with --force to re-apply`
- `tag '<tag>' already exists; re-run with --skip-tag or choose a new version`

## Dry-run output contract (sample)

Example dry-run output with a local tap path:

```text
dry-run: would download https://github.com/acme/tool/releases/download/1.2.3/tool-1.2.3-darwin-arm64.tar.gz
Repo: tap (/path/to/homebrew-tool)
  - Formula/tool.rb (modified)
```

## Canonical failure messages and exit codes

Missing required config or inputs (exit code 2):
- `missing required fields for --non-interactive: <fields>`
- `missing required fields for --dry-run: version or tag`
- `dry-run requires tap.path or an absolute tap.formula_path; tap.remote cannot be auto-cloned`
- Tap path guidance must mention the remote option:
  `tap.path, tap.remote, or tap.formula_path`

Invalid user input (exit code 3):
- `formula already targets version <version>; re-run with --force to re-apply`
- Asset selection ambiguity and missing-asset errors (see `docs/asset-selection.md`).

Git state failures (exit code 4):
- `<label> has uncommitted changes; re-run with --allow-dirty to continue`
- `tag '<tag>' already exists; re-run with --skip-tag or choose a new version`
- `multiple git remotes found; configure origin or set a matching remote URL`
