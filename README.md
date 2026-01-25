# brewdistillery

Homebrew formula initialization and release helper for CLI authors.

## Status

- Rust CLI, GitHub-only v0.
- Core commands implemented: `bd init`, `bd release` (alias `bd ship`), `bd doctor`.
- Dry-run is network-free and safe for CI input validation.

## Install (early build)

Requirements:
- Rust (stable toolchain)
- Git
- Homebrew (optional; required only for `bd init --tap-new` or `bd doctor --audit`)

Install from the workspace:

```bash
cargo install --path . --locked --bin bd
```

Run without installing:

```bash
cargo run -- --help
```

## Quickstart

### 1) Initialize config and formula

Example non-interactive init:

```bash
bd init --non-interactive \
  --tap-path ../homebrew-brewtool \
  --host-owner acme \
  --host-repo brewtool \
  --tap-owner acme \
  --tap-repo homebrew-brewtool \
  --formula-name brewtool \
  --description "Brew tool" \
  --homepage "https://github.com/acme/brewtool" \
  --license MIT \
  --bin-name brewtool \
  --version 0.1.0 \
  --asset-template "brewtool-{version}.tar.gz"
```

Create the tap repo via the GitHub API by adding `--create-tap` to the command
above and running with `GITHUB_TOKEN` (or `GH_TOKEN`) set.

Import an existing formula without overwriting it:

```bash
bd init --import-formula --tap-path ../homebrew-brewtool
```

### 2) Dry-run a release (no network calls)

```bash
bd release --version 0.1.0 --dry-run
```

Notes:
- Dry-run requires `--version` or `--tag`.
- Dry-run requires a local tap path (or an absolute `tap.formula_path`) when `tap.remote` is configured.
- If your release tags use a `v` prefix, set `[release] tag_format = "v{version}"`.

### 3) Validate setup

```bash
bd doctor
bd doctor --audit   # requires Homebrew
```

## Config Sample

See `docs/config.example.toml` for a full example. Minimal shape:

```toml
schema_version = 1

[project]
name = "brewtool"
description = "Brew tool"
homepage = "https://github.com/acme/brewtool"
license = "MIT"
bin = ["brewtool"]

[cli]
owner = "acme"
repo = "brewtool"

[tap]
owner = "acme"
repo = "homebrew-brewtool"
path = "../homebrew-brewtool"
formula = "brewtool"

[artifact]
strategy = "release-asset"
asset_template = "brewtool-{version}.tar.gz"
```

## Formula Sample

Rendered formulae always include an explicit version stanza:

```ruby
class Brewtool < Formula
  desc "Brew tool"
  homepage "https://github.com/acme/brewtool"
  url "https://github.com/acme/brewtool/releases/download/0.1.0/brewtool-0.1.0.tar.gz"
  sha256 "deadbeef"
  license "MIT"
  version "0.1.0"

  def install
    bin.install "brewtool"
  end
end
```

## Exit Codes

See `docs/errors.md` for the full catalog.

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | unexpected/internal |
| 2 | missing config/inputs |
| 3 | invalid input/blocked overwrite |
| 4 | git state error |
| 5 | network/API failure |
| 6 | audit failure |

## Specs and References

- `docs/documentation-outputs.md`
- `docs/init-prompt-flow.md`
- `docs/release-orchestration.md`
- `docs/non-interactive.md`

## Contributing

Run checks locally:

```bash
cargo fmt
cargo test
```

## License

MIT
