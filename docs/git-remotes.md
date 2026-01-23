# Git remote selection and ambiguity rules

This document defines how `brewdistillery` selects git remotes when inferring
repo identity (GitHub only in v0), and how ambiguity is handled.

## Selection order

When resolving owner/repo from git remotes:

1) Explicit overrides via flags or config:
   - `--host-owner` + `--host-repo` (CLI/artifact)
   - `cli.owner` + `cli.repo`
2) Explicit remote URL when present:
   - `cli.remote` (CLI repo)
3) Git remotes in the CLI repo (GitHub only):
   - Prefer `origin` if it is a GitHub URL.
   - Otherwise, if exactly one GitHub remote exists, use it.
   - Otherwise, treat as ambiguous.

Artifact repo identity follows the same order, but defaults to the CLI repo
identity if `artifact.owner`/`artifact.repo` are unset.

## Ambiguity behavior (non-interactive)

If remote selection is ambiguous in non-interactive mode, the command fails
with exit code 4 (`AppError::GitState`). No files are written.

### Canonical error messages

- Multiple GitHub remotes:
  `multiple git remotes found; specify --host-owner/--host-repo`

- No GitHub remotes and no explicit overrides:
  `no GitHub remote found; specify --host-owner/--host-repo`

- GitHub remote exists but is non-standard or unparsable:
  `unable to parse GitHub remote URL; specify --host-owner/--host-repo`

## Interactive behavior

When interactive mode is enabled, ambiguity should trigger a prompt listing
remotes by name and URL. The selected remote is then used for owner/repo
inference.
