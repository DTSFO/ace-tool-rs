# Backend Development Guidelines

This project is a single Rust crate plus an npm distribution wrapper. Backend work usually means changing the Rust library/binary under `src/`, its integration tests under `tests/`, or the packaging scripts under `npm/` and `scripts/`.

## Scope

- Rust crate: `src/lib.rs` exposes the library modules and `src/main.rs` owns CLI startup.
- Runtime modules: `config`, `mcp`, `tools`, `index`, `enhancer`, `service`, `strategy`, `utils`, and `http_logger`.
- Test suite: integration tests live in `tests/`; some focused unit tests are embedded in source modules.
- Packaging: `npm/ace-tool-rs/run.js`, `npm/platforms/*/package.json`, `npm/run.js`, and `scripts/` wrap or publish the Rust binary.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Where Rust modules, tools, services, tests, and packaging files belong | Current |
| [Persistence and Index Data](./database-guidelines.md) | Local bincode index, mtime cache, ignore rules, and configuration fingerprints | Current |
| [Error Handling](./error-handling.md) | `anyhow`, JSON-RPC errors, tool result text, HTTP JSON errors, and retry behavior | Current |
| [Logging Guidelines](./logging-guidelines.md) | `tracing`, stderr/stdout separation, optional HTTP request logging, and secret masking | Current |
| [Quality Guidelines](./quality-guidelines.md) | Rust style, tests, async/blocking boundaries, security limits, and verification commands | Current |

## Pre-Development Checklist

Before changing backend code, read the files that match the work:

- Module layout, new files, or public exports: `directory-structure.md`.
- Indexing, cache format, file scanning, blob hashing, or `.ace-tool` data: `database-guidelines.md`.
- CLI validation, MCP protocol behavior, HTTP/Web UI responses, or third-party API calls: `error-handling.md`.
- New logs, request logging, secrets, stdout/stderr behavior, or npm wrapper output: `logging-guidelines.md`.
- Any code change that affects behavior, tests, concurrency, parsing, or packaging: `quality-guidelines.md`.

Also read `.trellis/spec/guides/index.md` for cross-layer and code-reuse prompts when a change touches multiple modules or repeats existing patterns.

## Reliable Verification Commands

Use these commands for ordinary Rust changes:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

For npm wrapper changes, also run the smallest relevant Node command by path, for example:

```bash
node npm/ace-tool-rs/run.js --help
```

Do not write product logs or debug output to stdout in MCP server mode; stdout is reserved for JSON-RPC framing.
