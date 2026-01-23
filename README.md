# brewdistillery

Homebrew formula initialization and release helper. This is an early, in-progress CLI; the command surface is wired, but some workflows are still stubbed.

## Status

- Early development (pre-Homebrew). Expect incomplete behavior.
- CLI commands are available (`bd init`, `bd release`, `bd doctor`), but some workflows are still stubbed.
- `bd init` is interactive by default (use `--non-interactive` for CI) and writes `.distill/config.toml` plus a starter formula.
- `bd release` fetches GitHub releases, selects assets, computes SHA256, updates the formula with a preview, and commits/pushes by default (use `--no-push` or `--skip-tag`).

## Install (early build)

Homebrew installation is not available yet. Use one of the source-based options below.

Requirements:
- Rust toolchain (rustup + cargo, stable)
- Git (for cloning this repo)
- Homebrew (optional; only needed for `bd doctor --audit` or `bd init --tap-new`)
- Binary name is `bd` (crate name is `brewdistillery`).

Recommended quick path (clone + local install):

```
git clone <repo-url>
cd brewdistillery
cargo install --path . --locked
bd --help
```

Option A: install locally with cargo (recommended)

```
git clone <repo-url>
cd brewdistillery
cargo install --path . --locked
# or explicitly:
cargo install --path . --locked --bin bd
```

Binary path:
- `~/.cargo/bin/bd`

Make sure `~/.cargo/bin` is on your PATH (or use the full path above).

Option B: install directly from git (no manual build)

```
cargo install --git <repo-url> --locked --bin bd
```

Option C: build from source (release binary)

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

Option D: run from source (no install)

```
git clone <repo-url>
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

Alias (same as `bd release`):

```
bd ship --help
```

From source (no install):

```
cargo run -- --help
cargo run -- init --help
cargo run -- release --help
cargo run -- doctor --help
```

If you want to test against a real repo, run `bd init` from inside the CLI repository
(public GitHub remotes only in v0). Metadata detection currently supports `Cargo.toml`,
`package.json`, `pyproject.toml`, and `go.mod`.

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

Import an existing formula into config (keeps formula file untouched):

```
bd init --import-formula --tap-path ../homebrew-brewtool
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

Release testing (writes formula + commit by default; use `--dry-run` or `--no-push`):

```
bd release --version 0.1.0 --dry-run
```

`bd release` currently expects a public GitHub Release with matching assets for the
requested version. If asset selection fails, pass `--asset-name` or
`--asset-template` explicitly.

## Current capabilities

- `bd init` (interactive or `--non-interactive`): writes `.distill/config.toml` and a starter formula file with preview support.
- `bd init --import-formula`: imports an existing formula into config without overwriting the formula file.
- `bd release`: fetches the GitHub release, selects assets, computes SHA256, updates the formula, and commits/pushes to the tap (use `--no-push` and `--skip-tag` to opt out).
- `bd doctor`: validates required config fields, checks tap/formula paths, and optionally runs `brew audit --new-formula`.

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
