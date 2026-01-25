# Host abstraction (`HostClient`)

`brewdistillery` isolates host-specific behavior behind the `HostClient` trait
in `src/host/mod.rs`. GitHub is the only v0 provider, but the interface is
designed to support additional hosts later.

## Trait shape (v0)

```rust
pub trait HostClient {
    fn latest_release(&self, owner: &str, repo: &str, include_prerelease: bool)
        -> Result<Release, AppError>;

    fn release_by_tag(&self, owner: &str, repo: &str, tag: &str, include_prerelease: bool)
        -> Result<Release, AppError>;

    fn download_sha256(&self, url: &str, max_bytes: Option<u64>)
        -> Result<String, AppError>;
}
```

## Data model

The host trait returns a normalized release model:
- `Release.tag_name`
- `Release.draft`
- `Release.prerelease`
- `Release.assets[]`
  - `name`
  - `download_url`
  - `size` (optional)

This keeps asset selection and formula rendering host-agnostic.

## Behavioral contract

Implementations must:
- Enforce draft/prerelease rules at the API boundary.
- Return stable, actionable error messages via `AppError`.
- Support deterministic asset selection by providing a complete asset list.
- Respect configured download limits via `max_bytes`.

## Versioning plan

The current trait is intentionally small. If additional host features are
required later, prefer adding new methods with sensible defaults at the call
sites rather than overloading existing methods.

This reduces breaking changes while keeping the release orchestration logic
predictable.
