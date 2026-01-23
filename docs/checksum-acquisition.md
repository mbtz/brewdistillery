# Checksum acquisition

This document defines how `bd release` downloads assets and computes SHA256.

## Strategy

- Download the selected release asset directly from its `browser_download_url`.
- Stream the response and compute SHA256 locally.
- Ignore checksum/signature assets (`.sha256`, `.sha256sum`, `.sig`, `.asc`).

## Limits and timeouts

Defaults (v0):
- Max download size: 200 MiB.
- HTTP request timeout: 60 seconds per request.

If the response declares a `Content-Length` that exceeds the limit, the
operation fails immediately. If the stream exceeds the limit while reading,
`bd` aborts and returns a network error.

## Retry and backoff

`bd release` retries downloads on transient failures:
- Retryable errors:
  - Network/connect/read errors
  - HTTP 429 (rate limit)
  - HTTP 5xx
- Attempts: 3 total
- Backoff: exponential, starting at 250ms, doubling each retry (capped at 2s)

Non-retryable errors (fail fast):
- HTTP 4xx (other than 429)
- Size-limit violations

## Dry-run behavior

With `--dry-run`, downloads are skipped and the SHA256 value is reported as
`DRY-RUN`. URLs that would be downloaded are still printed for visibility.

## Error messages

Typical failures:
- `download exceeds size limit (<size> bytes > <limit> bytes): <url>`
- `failed to download asset <url>: HTTP <status>`
- `failed to read asset: <error>`
