# Repository identity model

This document defines how `brewdistillery` resolves repository identity for
CLI, tap, and artifact repositories, including precedence and examples.

## Identity definitions

- CLI repo: the repository where the CLI source lives and where
  `.distill/config.toml` resides.
- Tap repo: the Homebrew tap repository that contains `Formula/<formula>.rb`.
- Artifact repo: the repository that hosts release assets or source tarballs
  used by the formula (defaults to the CLI repo).

## Precedence rules

All identity values resolve in this order:
1) CLI flags
2) `.distill/config.toml`
3) Repo autodetect (git remote or manifest-derived)
4) Interactive prompts (when enabled)

If a required identity cannot be resolved after this chain, the command fails
in non-interactive mode.

## Mapping table

Identity | Config fields | Flags | Autodetect
---|---|---|---
CLI repo | `cli.owner`, `cli.repo`, `cli.remote`, `cli.path` | `--host-owner`, `--host-repo` | git remote (GitHub only in v0)
Tap repo | `tap.owner`, `tap.repo`, `tap.remote`, `tap.path`, `tap.formula`, `tap.formula_path` | `--tap-owner`, `--tap-repo`, `--tap-remote`, `--tap-path` | none (user-supplied)
Artifact repo | `artifact.owner`, `artifact.repo` | `--host-owner`, `--host-repo` | defaults to CLI repo identity

Notes:
- `--host-owner/--host-repo` are used for the CLI repo during `bd init` and for
  the artifact repo during `bd release` (with fallback to `cli.*`).
- Tap identity can be expressed by owner/repo, a remote URL, or a local path.

## Examples

### Single-repo (CLI + artifacts in same repo)

```
[cli]
owner = "acme"
repo = "brewtool"

[artifact]
strategy = "release-asset"
asset_template = "brewtool-{version}-{os}-{arch}.tar.gz"
```

In this case, `artifact.owner`/`artifact.repo` inherit from `cli.*`.

### Split repos (artifacts hosted elsewhere, separate tap)

```
[cli]
owner = "acme"
repo = "brewtool"

[tap]
owner = "acme"
repo = "homebrew-brewtool"
path = "../homebrew-brewtool"
formula = "brewtool"

[artifact]
owner = "acme-release"
repo = "brewtool-releases"
strategy = "release-asset"
asset_template = "brewtool-{version}-{os}-{arch}.tar.gz"
```

Here, `artifact.*` explicitly points to a different repo while the tap is
stored separately.
