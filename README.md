# brewdistillery

Homebrew formula initialization and release helper. This is an early, in-progress CLI; the command surface is wired, but most workflows are still stubbed.

## Status

- Early development (pre-Homebrew). Expect incomplete behavior.
- CLI commands are available (`bd init`, `bd release`, `bd doctor`), but most actions are not implemented yet.

## Install (early build)

Requirements:
- Rust toolchain (rustup + cargo)

Option A: build from source

```
git clone <this-repo>
cd brewdistillery
cargo build --release
```

Binary path:
- `target/release/bd`

Option B: install locally with cargo

```
cd brewdistillery
cargo install --path .
```

Binary path:
- `~/.cargo/bin/bd`

## Quick start

```
bd --help
bd init --help
bd release --help
bd doctor --help
```

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
