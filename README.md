# brewdistillery

Homebrew formula initialization and release helper. This is an early, in-progress CLI; the command surface is wired, but most workflows are still stubbed.

## Status

- Early development (pre-Homebrew). Expect incomplete behavior.
- CLI commands are available (`bd init`, `bd release`, `bd doctor`), but most actions are still stubbed.
- `bd init` is non-interactive only for now and writes `.distill/config.toml` plus a placeholder formula.
- `bd release` is a stub (version/tag validation only, no git or formula updates).

## Install (early build)

Requirements:
- Rust toolchain (rustup + cargo, stable)
- Git (for cloning this repo)

Option A: build from source (release binary)

```
git clone <this-repo>
cd brewdistillery
cargo build --release
```

Binary path:
- `target/release/bd`

Add it to your PATH (optional):

```
install -m 755 target/release/bd /usr/local/bin/bd
```

Option B: run from source (no install)

```
git clone <this-repo>
cd brewdistillery
cargo run -- --help
```

Option C: install locally with cargo

```
cd brewdistillery
cargo install --path .
```

Binary path:
- `~/.cargo/bin/bd`

Verify the install:

```
bd --help
```

Uninstall:

```
cargo uninstall brewdistillery
```

## Quick start

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

This writes `.distill/config.toml` in the CLI repo. Formula generation and tap
scaffolding are not implemented yet.

## Current capabilities

- `bd init --non-interactive`: writes `.distill/config.toml` and a placeholder formula file.
- `bd release`: validates `--version`/`--tag` and exits (no git or formula updates yet).
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
