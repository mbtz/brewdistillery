# Git integration approach (v0)

This note records the v0 decision to shell out to the `git` CLI and defines the
remote selection rules that all push operations must follow.

## Decision: use the git CLI (shelling out)

`brewdistillery` shells out to the user's installed `git` binary instead of
embedding libgit2.

Rationale:
- Git is already a hard requirement for v0.
- The CLI approach respects existing git config, auth helpers, and SSH setup.
- It keeps the dependency surface small and avoids libgit2 portability issues.
- The required operations are simple and well-supported by the CLI.

Out of scope for v0:
- Custom transports or credential flows beyond the user's git configuration.
- Git operations without a working `git` binary in `PATH`.

## Operations covered by the git module

The shared git module provides:
- clone a repo from a remote URL
- verify the working tree is clean
- add + commit one or more paths
- create a lightweight tag (with idempotency checks)
- push `HEAD` and push a tag
- select a remote deterministically

All git errors map to exit code 4 (`AppError::GitState`).

## Remote selection rules for push

Remote selection is deterministic and safe by default.

Selection order:
1. If a configured remote URL is available (for example `tap.remote`,
   `--tap-remote`, or `cli.remote`), prefer the remote whose URL exactly matches
   that value.
2. Otherwise, prefer `origin` when it exists.
3. Otherwise, if there is exactly one remote, use it.
4. Otherwise, fail with an actionable error.

Canonical ambiguity error message:
- `multiple git remotes found; configure origin or set a matching remote URL`

Notes:
- Remote URL matching is exact-string matching in v0.
- When `bd release` auto-clones a tap repo from `tap.remote`, the configured
  remote URL will match the cloned repo's `origin` remote.

## Idempotency rules enforced via git

- Dirty repos fail fast unless `--allow-dirty` is provided:
  - `<label> has uncommitted changes; re-run with --allow-dirty to continue`
- Tag creation is idempotent and fails when the tag already exists:
  - `tag '<tag>' already exists; re-run with --skip-tag or choose a new version`
