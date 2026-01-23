# brewdistillery

Homebrew formula initialization and release helper. This is an early, in-progress CLI; the command surface is wired, but most workflows are still stubbed.

## Status

- Early development (pre-Homebrew). Expect incomplete behavior.
- CLI commands are available (`bd init`, `bd release`, `bd doctor`), but most actions are not implemented yet.

## Install (early build)

Requirements:
- Rust toolchain (rustup + cargo)
- Git (for cloning this repo)

Option A: build from source

```
git clone <this-repo>
cd brewdistillery
cargo build --release
```

Binary path:
- `target/release/bd`

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

If you want to test against a real repo, run `bd init` from inside a Git repository
that has a GitHub remote (public repos only in v0).

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
