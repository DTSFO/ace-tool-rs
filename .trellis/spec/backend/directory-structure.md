# Directory Structure

ace-tool-rs is a single Rust crate with a binary entrypoint, a library surface, integration tests, and npm packaging wrappers. Keep new code in the module that owns the runtime concern; do not create feature folders that mix CLI, protocol, indexing, and provider-specific HTTP logic.

## Top-Level Layout

```text
src/
  main.rs                  # CLI parsing, tracing setup, mode selection, process exits
  lib.rs                   # public module declarations and selected re-exports
  config.rs                # Config, CLI override structs, file type defaults, upload heuristics
  mcp/                     # JSON-RPC/MCP transport, protocol types, server dispatch
  tools/                   # MCP tool argument schemas and tool execution wrappers
  index/                   # file scanning, blob splitting, local index cache, search uploads
  enhancer/                # prompt enhancement flow and local review Web UI
  service/                 # remote API provider clients and shared prompt/API helpers
  strategy/                # adaptive upload strategy and runtime metrics
  utils/                   # path normalization and project-local .ace-tool helpers
  http_logger.rs           # optional request/response log file writer
tests/                     # integration-style coverage by module or workflow
npm/                       # npm binary wrapper and platform package metadata
scripts/                   # release/build helper scripts
skills/                    # repository-owned assistant skills for local agents
```

Reference files:
- `src/lib.rs` declares every Rust module exported by the crate.
- `src/main.rs` owns `clap` arguments and decides between CLI subcommands, MCP server, `--index-only`, and `--enhance-prompt`.
- `tests/config_test.rs`, `tests/index_test.rs`, and `tests/third_party_api_test.rs` show the test-per-area pattern.
- `skills/ace-tool-rs/SKILL.md` is the source skill installed into local agent skill directories.

## Module Ownership

Use the existing runtime boundaries:

- `src/main.rs`: CLI-only concerns such as argument validation, tracing subscriber setup, mode branching, and process exit codes.
- `src/config.rs`: normalized configuration and default lists. Add new CLI-backed config fields to `ConfigOptions`, `Config`, `Config::new`, and tests in `tests/config_test.rs`.
- `src/mcp/`: protocol framing and JSON-RPC dispatch. `src/mcp/server.rs` handles stdin/stdout transport and method routing; `src/mcp/types.rs` holds serializable protocol structs.
- `src/tools/`: MCP tool schemas and argument validation. Tool modules should convert MCP arguments into domain calls, as `src/tools/search_context.rs` does before constructing `IndexManager`.
- `src/index/`: codebase indexing and search. Keep file collection, cache loading/saving, blob hashing, upload batching, and retrieval request construction here.
- `src/enhancer/`: prompt enhancement orchestration and local Web UI session management. Keep browser interaction and `hyper` server code in `src/enhancer/server.rs`.
- `src/service/`: provider-specific HTTP payloads and response parsing. Shared helpers such as conversation history parsing, URL joining, prompt rendering, and auth error mapping belong in `src/service/common.rs`.
- `src/strategy/`: adaptive upload behavior only. Put runtime counters and latency windows in `metrics.rs`; put AIMD adjustments in `adaptive.rs`.
- `src/utils/`: cross-platform path and project-local helper code. Path conversion belongs in `src/utils/path_normalizer.rs`; `.ace-tool` directory/index path helpers belong in `src/utils/project_detector.rs`.

Avoid putting provider-specific HTTP payload structs in `tools/` or `enhancer/`. The local pattern is that tool/enhancer code chooses an endpoint and `service/*` owns the exact API contract.

## Public API and Re-Exports

The crate uses `src/lib.rs` as the public surface for tests and downstream callers:

```rust
pub mod config;
pub mod enhancer;
pub mod http_logger;
pub mod index;
pub mod mcp;
pub mod service;
pub mod strategy;
pub mod tools;
pub mod utils;

pub use config::{get_upload_strategy, CliOverrides, Config, UploadStrategy};
pub use enhancer::PromptEnhancer;
pub use index::{Blob, IndexManager, IndexResult, IndexStats};
```

When adding a module, add it to `src/lib.rs` only if tests or external users need access. Prefer keeping provider modules `pub(crate)` under `src/service/mod.rs`, matching `claude`, `codex`, `gemini`, and `openai`.

## Adding MCP Tools

Follow the existing two-layer tool pattern:

- Add a `src/tools/<tool_name>.rs` module with a static tool definition, `get_input_schema()`, argument struct, `ToolResult`, and `execute`.
- Export the module from `src/tools/mod.rs`.
- Register the tool in `src/mcp/server.rs` in both `handle_list_tools` and `handle_call_tool`.
- Keep business logic outside the MCP dispatch branch; the branch should deserialize args, run the tool, and wrap `CallToolResult`.

Reference files:
- `src/tools/search_context.rs`
- `src/tools/enhance_prompt.rs`
- `src/mcp/server.rs`
- `tests/tools_test.rs`
- `tests/mcp_test.rs`

## Adding or Changing API Providers

Provider implementations are split by API:

- `src/service/augment.rs`: Augment `/prompt-enhancer` and `/chat-stream`.
- `src/service/claude.rs`: Anthropic messages API.
- `src/service/openai.rs`: OpenAI chat completions.
- `src/service/gemini.rs`: Gemini generate content.
- `src/service/codex.rs`: OpenAI Responses API shape used for Codex.
- `src/service/common.rs`: shared prompt rendering, history parsing, URL joining, endpoint enum, third-party config, and auth error mapping.

For a new provider, add a provider file under `src/service/`, re-export only the public call function from `src/service/mod.rs`, and route it from `src/enhancer/prompt_enhancer.rs`. Mirror the tests in `tests/third_party_api_test.rs` with `wiremock` expectations for path, auth header, success body, auth failure, empty response, and URL normalization.

## Tests

Prefer integration tests in `tests/` for public behavior:

- `tests/config_test.rs`: config normalization, defaults, and upload strategy thresholds.
- `tests/index_test.rs`: blob hashing, file collection, index format, atomic save, and edge cases.
- `tests/mcp_server_test.rs` and `tests/mcp_test.rs`: framing helpers and JSON-RPC types.
- `tests/prompt_enhancer_test.rs`: endpoint selection, templates, conversation parsing, and environment-driven config.
- `tests/third_party_api_test.rs`: provider HTTP contracts with `wiremock`.
- `tests/enhancer_server_test.rs`: local Web UI session/server helpers.

Use source-level unit tests only when the function is private and the behavior is tightly local, as in `src/strategy/metrics.rs`, `src/strategy/adaptive.rs`, and provider URL builders.

## Packaging Files

The npm layer is a wrapper around the Rust binary:

- `npm/ace-tool-rs/run.js` resolves optional platform packages and spawns the binary with inherited stdio.
- `npm/run.js` downloads GitHub release assets and deliberately writes wrapper diagnostics to stderr so MCP stdout stays clean.
- `npm/platforms/*/package.json` describes binary-only optional packages.

Keep packaging version fields aligned with `Cargo.toml` and README examples when bumping releases. Do not add stdout logging to npm wrappers unless the command is explicitly not used as an MCP transport.

## Scenario: CLI and Skill Distribution

### 1. Scope / Trigger

- Trigger: changing `src/main.rs`, `scripts/install.sh`, `npm/ace-tool-rs/run.js`, or `skills/ace-tool-rs`.
- Scope: first-class CLI subcommands, legacy MCP-compatible flags, repository-owned skill installation, and npm/source-checkout wrapper behavior.

### 2. Signatures

- `ace-tool-rs mcp [--config <path>] [--base-url <url>] [--token <token>] [--transport auto|lsp|line] [config flags...]`
- `ace-tool-rs index [--config <path>] [--project-root <path>] [--base-url <url>] [--token <token>] [config flags...]`
- `ace-tool-rs search [--config <path>] [--project-root <path>] --query <text> [--base-url <url>] [--token <token>] [config flags...]`
- `ace-tool-rs enhance [--config <path>] --prompt <text> [--conversation-history <text>] [--project-root <path>] [ACE config flags...] [prompt UI flags...]`
- `ace-tool-rs install-skill [--agents codex,claude,pi] [--source <dir>] [--force]`
- Legacy: `ace-tool-rs --base-url <url> --token <token> [--index-only] [--enhance-prompt <text>] [--transport ...]`

### 3. Contracts

- MCP mode uses stdout only for JSON-RPC frames; all diagnostics stay on stderr through `tracing`.
- `index` prints a human summary only for the subcommand path. Legacy `--index-only` preserves log-only stdout behavior.
- `search` prints only tool search text to stdout and returns an error if the tool result begins with `Error:`.
- `enhance` prints only the enhanced prompt to stdout.
- `install-skill` copies the source skill directory into `~/.codex/skills/ace-tool-rs`, `~/.claude/skills/ace-tool-rs`, and `~/.pi/agent/skills/ace-tool-rs` for the selected agents.
- `skills/ace-tool-rs` is source-controlled. Installed copies under user home are generated local state and must not be committed.
- ACE credentials resolve in this order: CLI flags, TOML config file, then `ACE_BASE_URL` / `ACE_TOKEN` environment variables for backward compatibility.
- Default config path is `~/.config/ace-tool-rs/config.toml`, or `$XDG_CONFIG_HOME/ace-tool-rs/config.toml` when `XDG_CONFIG_HOME` is set. The file currently supports `base_url` and `token`.

### 4. Validation & Error Matrix

- Missing base URL in MCP, index, or search mode after CLI/config/env resolution -> base URL required error.
- Missing token in MCP, index, or search mode after CLI/config/env resolution -> token required error.
- Malformed TOML config -> parse error including the config path, never the token value.
- Third-party enhance mode with only one of `--base-url` / `--token` -> paired-argument error.
- Subcommand plus legacy top-level flags -> reject; users must place options after the subcommand.
- `install-skill` target exists without `--force` -> refuse to overwrite that skill directory.
- `install-skill --source` missing `SKILL.md` -> reject before copying.

### 5. Good/Base/Bad Cases

- Good: `ace-tool-rs search --project-root "$PWD" --query "auth flow" --base-url "$ACE_BASE_URL" --token "$ACE_TOKEN"` prints results and no wrapper diagnostics.
- Base: `ace-tool-rs --base-url "$ACE_BASE_URL" --token "$ACE_TOKEN"` still starts the MCP server for existing configs.
- Bad: `ace-tool-rs --base-url "$ACE_BASE_URL" search --token "$ACE_TOKEN" --query auth` mixes legacy and subcommand placement and must fail.

### 6. Tests Required

- Clap help contains `mcp`, `index`, `search`, `enhance`, and `install-skill`.
- Legacy `--index-only` still parses with top-level `--base-url` / `--token`.
- `search` parses `--project-root` and `--query`.
- `search` parses `--config`, and credentials load from that TOML file when CLI flags are absent.
- CLI `--base-url` / `--token` override values from the TOML file.
- `install-skill` default agents are Codex, Claude, and Pi.
- Skill installation refuses an existing target without `--force` and replaces only that target with `--force`.
- Wrapper check: `node npm/ace-tool-rs/run.js --help` succeeds from a source checkout where platform optional packages are absent.

### 7. Wrong vs Correct

#### Wrong

```bash
ace-tool-rs --base-url "$ACE_BASE_URL" --token "$ACE_TOKEN" search --query auth
```

This makes command parsing ambiguous and can accidentally treat MCP transport flags as subcommand flags.

#### Correct

```bash
ace-tool-rs search --base-url "$ACE_BASE_URL" --token "$ACE_TOKEN" --query auth
```

Put subcommand-specific options after the subcommand. Keep the old top-level form only for legacy MCP, `--index-only`, and `--enhance-prompt` compatibility.

## Naming Conventions

- Rust modules use `snake_case` filenames and `mod.rs` for directory modules already present in the tree.
- Serde structs model external protocol fields with Rust `snake_case` plus `#[serde(rename = "...")]` at the boundary, as in `src/mcp/types.rs`, `src/enhancer/server.rs`, and `src/service/gemini.rs`.
- Constants for environment variables and fixed protocol defaults are uppercase, for example `ENV_ENHANCER_ENDPOINT`, `MAX_BODY_SIZE`, and `USER_AGENT`.
- Paths stored in index entries and blobs use forward slashes via `normalize_relative_path`.

## Anti-Patterns

- Do not put long provider-specific request structs into `src/main.rs`, `src/tools/`, or `src/enhancer/prompt_enhancer.rs`.
- Do not add new top-level directories for ordinary runtime modules; extend the existing module tree first.
- Do not expose private provider modules publicly unless tests or crate users need that surface.
- Do not print diagnostics to stdout in code paths used by MCP transport.
