# Multi-binary support

`brewdistillery` supports installing multiple binaries from a single formula via
`project.bin = [..]` and `bin.install "a", "b"` rendering.

This document captures how multi-bin values are collected and persisted during
`bd init`.

## Prompt capture (`bd init`)

The interactive prompt label is:
- `Binary name(s)`

Input rules:
- Comma-separated values are supported.
- Whitespace is also treated as a separator within each comma group.
- Empty entries are ignored.

Examples:
- `tool` -> `["tool"]`
- `tool, toolctl` -> `["tool", "toolctl"]`
- `tool toolctl, helper` -> `["helper", "tool", "toolctl"]`

Note: bins are normalized by trimming, sorting, and deduplicating. The stored
order is alphabetical to keep outputs deterministic.

## Default sources

Default bin values resolve in this order:
1. `--bin-name` flags (one or more)
2. `project.bin` from existing config
3. Detected manifest bins
4. `tap.formula` when present
5. The resolved formula name

## Config schema impact

Bins are stored under `[project]`:

```toml
[project]
bin = ["tool", "toolctl"]
```

## Formula rendering

Rendering behavior depends on the number of bins:
- Single bin:
  - `bin.install "tool"`
- Multiple bins:
  - `bin.install "tool", "toolctl"`

See `docs/formula-template.md` for complete examples.
