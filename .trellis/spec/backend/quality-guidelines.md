# Quality Guidelines

The codebase favors small Rust modules, explicit boundary validation, integration tests for public behavior, and defensive limits around protocol/file/HTTP inputs. Match the existing patterns before adding abstractions.

## Formatting and Style

- Rust edition is 2021.
- Formatting follows `rustfmt`; verify with `cargo fmt --check`.
- `.editorconfig` requires UTF-8, LF, final newline, trimmed trailing whitespace, and 4-space Rust indentation.
- Use `serde` structs for JSON/protocol payloads instead of manual string assembly.
- Prefer `Arc<Config>` for shared runtime configuration, matching `Config::new` and consumers such as `IndexManager`, `PromptEnhancer`, and MCP tools.
- Keep comments short and useful; the existing code uses comments mainly to explain protocol, security, platform, and concurrency choices.

## Required Verification

For ordinary Rust changes, run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

When a change only edits Trellis specs or Markdown, at minimum search `.trellis/spec` for Trellis template marker phrases and remove any matches before finishing.

For npm wrapper or package metadata changes, also run the smallest relevant Node command, such as `node npm/ace-tool-rs/run.js --help`, and check that wrapper diagnostics use stderr.

## Test Placement

Use the existing test structure:

- Public module behavior: integration tests in `tests/`.
- Private pure helpers: source-level unit tests near the helper.
- HTTP provider contracts: `wiremock` in `tests/third_party_api_test.rs`.
- Filesystem behavior: `tempfile::TempDir`.
- Async behavior: `#[tokio::test]`.
- Environment-variable tests: guard global env mutations with a `Mutex`, as in `tests/prompt_enhancer_test.rs` and `src/enhancer/prompt_enhancer.rs` tests.

Do not write tests that depend on the developer's home directory, network access, real API tokens, or persistent `.ace-tool` state.

## What to Test

Add or update tests when changing:

- CLI/config fields: defaults, normalization, required combinations, and invalid values in `tests/config_test.rs`.
- MCP protocol behavior: JSON-RPC structs, method dispatch, error codes, framing helpers, header parsing, size limits.
- Tool schemas or argument handling: `tests/tools_test.rs` and `tests/enhance_prompt_test.rs`.
- Indexing: blob hashing, chunk paths, file inclusion/exclusion, config hash, index version, corrupted/oversized cache, atomic save, unicode/special paths.
- Provider calls: URL construction, auth headers, success parsing, XML tag extraction, no-tag fallback, auth failures, empty responses, conversation history, and version-prefix deduplication.
- Web UI server: CORS headers, JSON response helpers, session lifecycle, bind address behavior, request body limits.
- Adaptive strategy: warmup, CLI overrides, rate-limit downgrade, 5xx exclusion, cooldown.

When modifying a bug-prone helper, add a focused regression test that fails without the fix. Recent examples are version-prefix deduplication tests in provider URL builders and body/UTF-8 boundary tests in `http_logger`.

## Async and Blocking Boundaries

Do not run heavy filesystem walks, file reads, or rayon loops directly on the async runtime. `IndexManager::index_project` uses `tokio::task::spawn_blocking` for directory scanning and parallel file processing.

Use `FuturesUnordered` for bounded concurrent async uploads, as in `upload_blobs_adaptive`. Let `AdaptiveStrategy` own concurrency and timeout changes.

Do not hold async locks across slow external work unless the current pattern requires it. `EnhancerServer::start` deliberately holds the `actual_addr` write lock through bind/setup to avoid port readiness races; copy that pattern only for similar readiness invariants.

## Security and Resource Limits

Preserve existing input limits:

- MCP line messages: 10 MB.
- LSP header line: 1 KB.
- LSP payload: 10 MB.
- Local Web UI request body: 1 MB.
- Blob content/file size: 128 KB per file/blob before upload.
- Batch upload payload: 1 MB.
- Index cache file: 256 MB.
- HTTP log body: 10 KB.
- Third-party injected search context: 12,000 characters.

When adding a new parser, body reader, cache reader, or protocol field, define a size limit or justify why an existing one applies.

For Web UI binding, keep the non-loopback warning in `src/enhancer/server.rs`. The UI is unauthenticated.

## Cross-Platform Paths

Path code must preserve WSL/Windows/Unix behavior:

- Use `RuntimeEnv::detect` and `normalize_path` when accepting a project root.
- Use `normalize_relative_path` for blob/index keys.
- Do not manually split Windows paths by backslash in indexing code.
- Keep tests in `tests/path_normalizer_test.rs` updated for UNC, `/mnt/<drive>`, unicode, and round-trip cases.

## Provider API Contracts

Use provider-specific payload structs and `serde` renames. Keep URL joining through `build_api_url` so `https://host/v1` plus `/v1/messages` does not become `/v1/v1/messages`.

Provider response parsing should:

- map 401/403 through shared auth helpers where applicable,
- return empty response errors,
- preserve useful response body context in parse/status errors,
- extract `<augment-enhanced-prompt>` when providers return it,
- call `replace_tool_names` before returning enhanced text.

Do not duplicate conversation history parsing; use `parse_chat_history` from `src/service/common.rs`.

## Compatibility and Behavior Preservation

Before changing a constant, environment variable, protocol name, model default, endpoint path, npm package name, or file path, search the whole repo first. Many values are mirrored in README, tests, npm metadata, or provider-specific modules.

Examples:

- Version values appear in `Cargo.toml`, README examples, npm package metadata, and platform package metadata.
- Endpoint environment variables are documented in README and tested in `tests/prompt_enhancer_test.rs`.
- `search_context` and legacy `codebase-retrieval` alias are both handled in `src/mcp/server.rs`.
- `USER_AGENT` in `src/lib.rs` is used for outbound Augment-compatible requests.

## Forbidden Patterns

- No network-dependent tests.
- No real API tokens in tests, docs, or logs.
- No stdout diagnostics in MCP paths.
- No unbounded request/body/cache reads.
- No blocking filesystem scans on the async runtime.
- No manual JSON construction for protocol/provider payloads when `serde` can model it.
- No broad refactors across unrelated modules just to add a small feature.
- No new global environment mutation in tests without a mutex and restoration.
