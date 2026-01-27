# Preview, confirmation, and atomic writes

This document defines the preview/diff UX contract, confirmation behavior, and
atomic write guarantees for `bd init` and `bd release`.

## Preview structure (multi-repo)

Preview output is generated from `RepoPlan` entries. Each plan has:
- `label` (e.g., `cli`, `tap`)
- `repo_root` (path for relative rendering)
- `writes` (planned file writes)

Plans are processed in the order they are built (typically CLI first, then tap).
Within each repo, files are ordered by their relative path.

### Summary format

For each repo with changes:
```
Repo: <label> (<repo_root>)
  - <relative/path> (new|modified)
```

If a write is a no-op (content matches existing file), it is omitted.

### Diff format

A unified diff is printed for each change, in the same order as the summary.
Diff headers are namespaced by repo label:
- `a/<label>/<relative/path>`
- `b/<label>/<relative/path>`

The diff uses a context radius of 3 lines.

### No-op messaging

If no files change after planning:
- `bd init` prints: `init: no changes to apply`
- `bd release` prints: `release: no changes to apply`

`bd release` may also emit `release: no changes to commit` or
`release: no formula changes to commit` after preview if nothing is staged.

## Dry-run contract

`--dry-run` always prints the preview summary/diff and then exits without
writing any files. Additional dry-run messaging:
- `bd init`: `dry-run: no changes applied`
- `bd release`: prints planned actions (tag/commit/push summary) and exits

Dry-run does not create commits or tags, and does not push. Release dry-run
also skips asset downloads and checksum computation.

## Confirmation behavior

### `bd init`

Interactive `bd init` shows the preview, then prompts:
`Apply these changes?` (default: yes)

If the user declines, the command exits with:
`init: cancelled`

When `--yes` or `--force` is provided (or when overwrite is otherwise allowed),
confirmation is skipped and the changes are applied.

Non-interactive and dry-run flows never prompt.

### `bd release`

`bd release` prints the preview and planned actions but does not prompt by
default. Use `--dry-run` for a safe preview, or `--no-push` to prevent pushes.

## Atomic write guarantees

Each file is written atomically:
1. A temporary file is created in the target directory.
2. The content is written and fsynced.
3. The temp file is persisted (atomic rename) to the final path.

This ensures individual files are never left partially written. Temp files are
cleaned up automatically if the write fails.

### Rollback behavior

Writes are atomic per file, but **not** transactional across multiple files or
repos. If a failure occurs mid-apply, earlier files may already be persisted.
The command returns an error; re-running is safe and will show a new preview.

### Temp file strategy

Temp files are created in the same directory as the final target path to ensure
atomic renames across filesystems. On failure, the temp file is dropped and
removed automatically by the OS/`tempfile` crate.

## Examples

Summary + diff for a multi-repo update:
```
Repo: cli (/path/to/cli)
  - .distill/config.toml (modified)
Repo: tap (/path/to/homebrew-foo)
  - Formula/foo.rb (modified)

diff --git a/cli/.distill/config.toml b/cli/.distill/config.toml
@@
- old = true
+ old = false

diff --git a/tap/Formula/foo.rb b/tap/Formula/foo.rb
@@
- sha256 "old"
+ sha256 "new"
```
