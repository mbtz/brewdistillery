# brewdistillery

Homebrew formula initialization and release helper. This is an early, in-progress CLI; the command surface is wired, but most workflows are still stubbed.

## Status

- Early development (pre-Homebrew). Expect incomplete behavior.
- CLI commands are available (`bd init`, `bd release`, `bd doctor`), but some workflows are still stubbed.
- `bd init` is non-interactive only for now and writes `.distill/config.toml` plus a placeholder formula.
- `bd release` fetches GitHub releases, selects assets, computes SHA256, and updates the formula with a preview (no git commit/tag/push yet).

## Install (early build)

Homebrew installation is not available yet. Use one of the source-based options below.

Requirements:
- Rust toolchain (rustup + cargo, stable)
- Git (for cloning this repo)
- Homebrew (optional; only needed for `bd doctor --audit` or `bd init --tap-new`)

Option A: build from source (release binary)

```
git clone <repo-url>
cd brewdistillery
cargo build --release
```

Binary path:
- `target/release/bd`

Add it to your PATH (optional):

```
install -m 755 target/release/bd /usr/local/bin/bd
```

On Apple Silicon (Homebrew default prefix):

```
install -m 755 target/release/bd /opt/homebrew/bin/bd
```

Option B: install locally with cargo

```
cd brewdistillery
cargo install --path .
```

Binary path:
- `~/.cargo/bin/bd`

Option C: run from source (no install)

```
git clone <this-repo>
cd brewdistillery
cargo run -- --help
```

Verify the install:

```
bd --help
```

Uninstall:

```
cargo uninstall brewdistillery
```

If you installed the binary manually, remove it from the path you used
(`/usr/local/bin/bd` or `/opt/homebrew/bin/bd`).

## Usage (early testing)

If you did not install the binary, run via `cargo run --` instead of `bd`.

Quick start:

```
bd --help
bd init --help
bd release --help
bd doctor --help
```

If you want to test against a real repo, run `bd init --non-interactive` from inside
the CLI repository (public GitHub remotes only in v0). Metadata detection currently
supports `Cargo.toml`, `package.json`, `pyproject.toml`, and `go.mod`.

Example (explicit fields):

```
bd init --non-interactive \
  --tap-path ../homebrew-brewtool \
  --host-owner acme \
  --host-repo brewtool \
  --formula-name brewtool \
  --description "Brew tool" \
  --homepage "https://github.com/acme/brewtool" \
  --license MIT \
  --bin-name brewtool \
  --version 0.1.0
```

This writes `.distill/config.toml` in the CLI repo and a placeholder formula in the
tap path. Tap scaffolding and git automation are still in progress.

Dry-run init (no writes):

```
bd init --non-interactive --dry-run \
  --tap-path ../homebrew-brewtool \
  --host-owner acme \
  --host-repo brewtool \
  --formula-name brewtool \
  --description "Brew tool" \
  --homepage "https://github.com/acme/brewtool" \
  --license MIT \
  --bin-name brewtool \
  --version 0.1.0
```

Release testing (no git commit/tag/push yet):

```
bd release --version 0.1.0 --dry-run
```

`bd release` currently expects a public GitHub Release with matching assets for the
requested version. If asset selection fails, pass `--asset-name` or
`--asset-template` explicitly.

## Current capabilities

- `bd init --non-interactive`: writes `.distill/config.toml` and a placeholder formula file.
- `bd release`: fetches the GitHub release, selects assets, computes SHA256, and updates the formula with a preview (no git commit/tag/push yet).
- `bd doctor`: CLI wiring only (checks are still stubbed).

If you run into missing fields in non-interactive mode, provide explicit flags for
all required inputs (see `bd init --help`).

## Config location

By default, config is read from and written to:

- `.distill/config.toml` in your CLI repo

Use `--config <path>` to point elsewhere.

## Notes

- Homebrew-based install is not available yet.
- The planned config path is `.distill/config.toml` in your CLI repo.

## Contributing

Run tests:

```
cargo test
```

## License

MIT
