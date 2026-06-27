# Logging Guidelines

Runtime logs use `tracing`. The most important convention is transport safety: MCP JSON-RPC uses stdout, so diagnostic logs must go to stderr or to an opt-in log file.

## Tracing Setup

`src/main.rs` initializes tracing like this:

```rust
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    .with(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

This keeps MCP stdout clean and lets users control verbosity through `RUST_LOG`.

Do not replace this with stdout logging. `println!` is only acceptable for intentional command output, such as the final enhanced prompt printed by `--enhance-prompt` mode in `src/main.rs`.

## Log Levels

Follow the existing level choices:

- `debug!`: protocol internals and high-volume detail, such as received/sent JSON-RPC payloads in `src/mcp/server.rs` or filtered metric details in `src/strategy/metrics.rs`.
- `info!`: lifecycle milestones and normal progress, such as server start, indexing phases, provider calls, batch starts, strategy adjustments, and session completion.
- `warn!`: recoverable problems or degraded behavior, such as skipped files, partial indexing, port fallback, missing index data, browser-open failure, or non-loopback Web UI binding.
- `error!`: failed operations that stop a request/session/batch or represent protocol/service errors, such as parse failures, failed re-enhancement, failed index save, or upload batch failure.

Reference files:
- `src/main.rs`
- `src/index/manager.rs`
- `src/mcp/server.rs`
- `src/enhancer/prompt_enhancer.rs`
- `src/enhancer/server.rs`
- `src/strategy/adaptive.rs`

## What to Log

Log state transitions and operational context that helps diagnose behavior:

- CLI mode: `--index-only`, `--enhance-prompt`, and MCP server startup.
- Project/index flow: project root, file scan start/count, cached/new blob counts, upload batch progress, partial failures, final stats.
- Adaptive upload: initial strategy and upgrade/downgrade reasons.
- Provider calls: provider name, API URL, and duration.
- Web UI: server start address, session creation/completion, browser-open fallback, timeout.
- Recoverable file issues: skipped large files, unreadable entries, invalid/corrupted index cache.

Keep messages concrete and tied to the operation. Avoid generic "failed" messages without path/status/provider context.

## What Not to Log

Do not log raw secrets:

- ACE bearer token from CLI `--token`.
- `PROMPT_ENHANCER_TOKEN`.
- Authorization, cookie, API key, or proxy auth headers.
- Full request/response bodies unless the user explicitly opted into `ACE_HTTP_LOG`.

Do not log user prompt text or conversation history at `info` level. Current provider code logs API URL and duration, not request bodies. Preserve that default privacy boundary.

Do not write diagnostic output to stdout in MCP server mode or npm wrapper code used by MCP transport.

## Optional HTTP Request Logging

`src/http_logger.rs` provides opt-in request/response file logging when `ACE_HTTP_LOG` is one of:

```text
1, true, yes, on
```

When enabled:

- logs go to `.ace-tool/http_requests.log`,
- writes are protected by a global mutex,
- bodies are capped at 10 KB and truncated on UTF-8 character boundaries,
- JSON bodies are pretty-printed before truncation,
- sensitive headers are masked,
- `.ace-tool` is created when needed and failures are warned, not fatal.

Use `http_logger::is_enabled()` before serializing request bodies. The index and Augment provider code already uses lazy serialization so normal runs do not pay body-serialization cost.

Reference files:
- `src/http_logger.rs`
- `tests/http_logger_test.rs`
- HTTP logging calls in `src/index/manager.rs` and `src/service/augment.rs`

## Secret Masking

`http_logger::is_sensitive_header` treats these headers as sensitive:

- `authorization`
- `set-cookie`
- `cookie`
- `x-api-key`
- `x-auth-token`
- `proxy-authorization`
- `x-goog-api-key`

`mask_token` preserves only the first and last four characters for longer tokens and masks short tokens as `****`. Do not add new request logging paths that bypass `mask_sensitive_header`.

The Augment service deliberately uses `REDACTED_TOKEN` when constructing log headers for enhancer requests. Follow that pattern when an existing helper would otherwise receive a raw secret.

## MCP and Npm Stdout Safety

MCP protocol responses are written with `write_message` in `src/mcp/server.rs`; stdout must contain only JSON-RPC frames in server mode.

The npm wrappers follow the same rule:

- `npm/ace-tool-rs/run.js` inherits stdio and lets the Rust binary own protocol stdout.
- `npm/run.js` comments that wrapper logs and extraction output must go to stderr so Rust stdout remains the only protocol stream.

Use `console.error` for wrapper diagnostics. Do not add `console.log` to wrapper paths that can run as MCP servers.

## Log File Paths

Use project-root-aware `.ace-tool` paths where possible:

- `http_logger::log_request(Some(project_root), ...)` writes logs beside the indexed project.
- `http_logger::log_request(None, ...)` writes to current directory `.ace-tool`.

For search/index calls, pass the project root so logs are stored with the indexed project.

## Anti-Patterns

- Do not initialize a second tracing subscriber.
- Do not print debug diagnostics with `println!` or `eprintln!` from Rust when `tracing` is available.
- Do not serialize HTTP bodies for logging unless `ACE_HTTP_LOG` is enabled.
- Do not log full tokens, cookies, API keys, prompts, or conversation history by default.
- Do not make HTTP log write failures fail the user operation.
