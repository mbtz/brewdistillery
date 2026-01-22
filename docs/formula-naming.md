# Formula naming and class normalization

## Formula name normalization rules

Inputs come from `--formula-name` or the repo name (prompted if empty). The name is normalized
before validation and persisted in config/formula paths.

Normalization steps:
- Trim leading/trailing whitespace.
- Map spaces, underscores, and dashes to a single `-`.
- Lowercase ASCII letters.
- Collapse consecutive dashes into a single dash.
- Trim leading/trailing dashes.

Validation rules (post-normalization):
- Must be non-empty.
- Must not start with `homebrew-`.
- Allowed characters: `a-z`, `0-9`, and `-` only.

If any validation fails, surface an `InvalidInput` error and exit with code 3.

## Class name normalization rules

Class names are derived from the normalized formula name:
- Split on `-` (and `_` defensively, though normalization should remove them).
- Capitalize the first character of each segment.
- Lowercase the remaining characters of each segment.
- Preserve digits as-is.

No acronym preservation is attempted (e.g., `http` -> `Http`).

## Examples

| Input | Normalized formula | Class name |
| --- | --- | --- |
| `My Tool` | `my-tool` | `MyTool` |
| `my__tool` | `my-tool` | `MyTool` |
| `foo--bar` | `foo-bar` | `FooBar` |
| `FOO_bar` | `foo-bar` | `FooBar` |
| `http-server` | `http-server` | `HttpServer` |
| `cli2-tools` | `cli2-tools` | `Cli2Tools` |
| `brew2-go` | `brew2-go` | `Brew2Go` |
| `git-HTTP` | `git-http` | `GitHttp` |
| `  _my__tool_  ` | `my-tool` | `MyTool` |

Invalid examples:
- `homebrew-foo` (reserved `homebrew-` prefix)
- `foo@bar` (illegal character `@`)
- `---` (normalizes to empty)
