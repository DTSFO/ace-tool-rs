# Error Handling

The project uses `anyhow::Result` for fallible runtime flows, explicit JSON-RPC errors for protocol failures, plain text tool results for MCP tool execution failures, and JSON HTTP responses for the local Web UI. Keep those boundaries separate.

## General Rust Pattern

Use `anyhow::{anyhow, Result}` for application and provider flows where callers only need a message. This is the dominant pattern in:

- `src/main.rs`
- `src/config.rs`
- `src/index/manager.rs`
- `src/enhancer/prompt_enhancer.rs`
- `src/enhancer/server.rs`
- `src/service/*.rs`
- `src/mcp/server.rs`

Prefer `ok_or_else`, `map_err`, and contextual `anyhow!` messages at validation boundaries:

```rust
let token = args.token.ok_or_else(|| anyhow!("--token is required"))?;
```

Use typed enums only when downstream logic branches on the category. `src/strategy/metrics.rs::ErrorType` is the local example: upload outcomes classify timeout, rate limit, server, client, and network errors for adaptive strategy decisions.

## CLI Errors

`src/main.rs` returns `anyhow::Result<()>`. Missing required CLI arguments and invalid mode combinations should return `Err(anyhow!(...))`; do not print ad hoc errors before returning.

Current rules:

- MCP server and `--index-only` require `--base-url` and `--token`.
- `--enhance-prompt` with Augment `new`/`old` endpoints requires `--base-url` and `--token`.
- `--enhance-prompt` with third-party endpoints may omit ACE `--base-url` and `--token`, but if one is supplied both must be supplied.
- `PROMPT_ENHANCER_INCLUDE_SEARCH_CONTEXT=1` with third-party endpoints requires project root plus ACE search config.

For `--index-only`, `main.rs` maps `IndexResult.status` to process behavior:

- `success`: return `Ok(())`.
- `partial`: log warnings and exit with code 2.
- anything else: return an error.

## MCP Protocol Errors

Protocol and JSON-RPC errors are represented with `JsonRpcResponse::error` in `src/mcp/types.rs` and returned by `src/mcp/server.rs`.

Follow the current error codes:

- `-32700`: JSON parse error.
- `-32601`: unknown JSON-RPC method.
- `-32602`: missing params, invalid params, invalid tool arguments, disabled tool, or unknown tool.
- `-32603`: internal serialization errors while building responses.

Requests without `id` are notifications and must not receive responses. Known initialized notifications are ignored; unknown notifications are debug-logged only.

Reference files:
- `src/mcp/server.rs`
- `src/mcp/types.rs`
- `tests/mcp_test.rs`
- `tests/mcp_server_test.rs`

## MCP Tool Execution Errors

Tool `execute` methods return `ToolResult { text }` rather than `Result<T>`. Argument and business failures inside a tool are returned as text beginning with `Error:`.

Examples:

- `src/tools/search_context.rs` returns `Error: query is required`, `Error: project_root_path is required`, or `Error: Project path does not exist`.
- `src/tools/enhance_prompt.rs` returns `Error: prompt is required` or `Error: <enhancer error>`.

Do not convert these tool execution failures into JSON-RPC protocol errors after arguments have deserialized successfully. The current MCP behavior wraps them in `CallToolResult` text content.

## Transport and Framing Errors

MCP stdin reading is deliberately defensive:

- Line mode rejects lines over 10 MB.
- LSP mode rejects header lines over 1 KB.
- LSP mode rejects more than `MAX_HEADER_COUNT` header/blank lines.
- LSP mode rejects payloads over 10 MB.
- LSP mode rejects invalid UTF-8 payloads.

`McpServer::run` logs read errors and continues reading the next message. Preserve this resilience for malformed input. Do not let one malformed request terminate the server loop.

## Indexing Errors

Indexing distinguishes recoverable local failures from fatal workflow failures:

- File walk entry failures, metadata failures, read failures, large files, and binary files are logged/skipped.
- Cache load failures return an empty index and trigger rebuild.
- Transient per-file processing errors preserve old entries when possible.
- Empty file discovery or no processed files returns `IndexResult { status: "error" }`.
- Failed upload batches return `status: "partial"` when the index save succeeds.
- Index save failure returns `status: "error"`.

`search_context` treats `index_project().status == "error"` as fatal and `partial` as a warning. Keep that distinction when adding new index statuses.

Reference files:
- `src/index/manager.rs`
- `tests/index_test.rs`

## HTTP Provider Errors

Provider modules convert remote API responses into `anyhow::Error` messages:

- Augment endpoints in `src/service/augment.rs` map 401 to `Token invalid or expired` and 403 to `Access denied, token may be disabled`.
- Third-party providers share `map_auth_error` from `src/service/common.rs`, producing provider-specific 401/403 messages.
- Non-success HTTP statuses include status and response body.
- JSON parse errors include the response body for provider diagnostics.
- Empty provider responses are explicit errors, not empty strings.
- Codex refusal content becomes `Codex API refusal: ...`.

Reference tests:
- `tests/third_party_api_test.rs`
- provider unit tests in `src/service/claude.rs`, `src/service/openai.rs`, `src/service/gemini.rs`, and `src/service/codex.rs`

## Upload Retry and Partial Failure Rules

`IndexManager::upload_batch_internal` handles upload failures differently by class:

- 401/403/400 are client errors and are not retried.
- 429 honors `Retry-After` and retries while attempts remain.
- 5xx retries with exponential backoff while attempts remain.
- reqwest timeouts and other network errors retry while attempts remain.
- Each batch returns `BatchUploadResult` with `success`, `latency_ms`, `blob_names`, and optional `ErrorType`.

Keep retry classification aligned with `AdaptiveStrategy::record_outcome`; changing one without the other can degrade concurrency behavior.

## Local Web UI HTTP Errors

The enhancer Web UI uses `hyper` and returns JSON error bodies for API routes:

- Missing `session` query: 400.
- Unknown session: 404.
- Body too large or unreadable: 400.
- Invalid JSON body: 400.
- Already completed/timed-out session: 400.
- Missing enhance callback or re-enhance failure: 500.

Use `json_error_response` in `src/enhancer/server.rs` for API errors so error strings are safely JSON-escaped. Keep request bodies capped by `read_body_with_limit` and `MAX_BODY_SIZE`.

## Panic and Unwrap Policy

Production code should return errors for external input, I/O, HTTP, and protocol parsing failures. Existing `unwrap()` calls in production code are mostly limited to:

- static regex initialization,
- building responses with known-valid headers/status values,
- JSON serialization of small internal response bodies.

New code should not use `unwrap()` on user input, filesystem data, network data, environment variables, or JSON payloads from clients/providers. Tests can use `unwrap()` and `expect()` for setup and assertions.

## Anti-Patterns

- Do not use `thiserror` for new error types unless callers need structured matching; `anyhow` is the current application pattern.
- Do not return JSON-RPC errors for successful tool dispatch where the tool itself failed.
- Do not abort MCP server loop on one malformed stdin message.
- Do not hide provider response bodies from parse/status errors when they are needed for debugging, but avoid logging secrets.
- Do not introduce panics for malformed external input.
