# Technical Design

## Architecture

The conversion keeps the existing Rust crate and npm package shape. `src/main.rs` remains the CLI and process-mode owner, while indexing, search, prompt enhancement, and MCP protocol logic stay in their current modules.

The CLI will support both:

- legacy flat flags: `--index-only`, `--enhance-prompt`, and default MCP server mode,
- new subcommands: `mcp`, `index`, `search`, `enhance`, and `install-skill`.

This avoids breaking existing MCP and npm users while giving skills stable command forms to invoke.

## CLI Contract

Shared ACE options:

- `--base-url <url>`
- `--token <token>`
- `--max-lines-per-blob <n>`
- `--upload-timeout <seconds>`
- `--upload-concurrency <n>`
- `--retrieval-timeout <seconds>`
- `--no-adaptive`

Mode-specific options:

- `mcp`: transport, web/browser prompt-enhancer flags.
- `index`: `--project-root <path>`, default current directory.
- `search`: `--project-root <path>`, `--query <text>`.
- `enhance`: `--prompt <text>`, optional `--conversation-history`, optional `--project-root`.
- `install-skill`: `--agents <codex,claude,pi>`, optional `--source <path>`, optional `--force`.

Legacy usage maps into the same internal mode handlers as subcommands.

## Skill Package

The repository will contain `skills/ace-tool-rs/`:

- `SKILL.md`: concise agent instructions, trigger conditions, CLI commands, safety rules.
- `agents/openai.yaml`: UI metadata for Codex-style skill lists.

The skill names the local `ace-tool-rs` CLI and avoids embedding credentials. It directs agents to use semantic search when file locations are unknown and to use exact shell/text search for identifiers.

## Installer

`install-skill` copies the repository skill folder to selected local directories:

- Codex: `~/.codex/skills/ace-tool-rs`
- Claude: `~/.claude/skills/ace-tool-rs`
- Pi: `~/.pi/agent/skills/ace-tool-rs`

The command fails if a target exists unless `--force` is supplied. This prevents accidental overwrite of manually edited skills. The current Pi `ace-context-engine` skill is unrelated and remains untouched.

## Data Flow

Search subcommand:

`CLI args -> Config -> SearchContextTool -> IndexManager -> ACE API -> stdout result`

Index subcommand:

`CLI args -> Config -> IndexManager::index_project -> exit code`

Enhance subcommand:

`CLI args/env -> Config or third-party config -> PromptEnhancer -> stdout result`

MCP subcommand:

`CLI args -> Config -> McpServer -> stdout JSON-RPC frames`

Skill install:

`CLI args -> resolve repo skill source -> validate SKILL.md -> copy files -> stderr status`

## Compatibility

- The binary name remains `ace-tool-rs`.
- Existing README MCP examples continue to work.
- Existing `--index-only` and `--enhance-prompt` behavior remains accepted.
- MCP mode still writes only protocol frames to stdout; CLI commands that intentionally return user data may print to stdout.

## Error Handling

- Use `anyhow::Result` in `src/main.rs` handlers.
- Missing required CLI args return errors rather than ad hoc prints.
- Tool execution errors from `SearchContextTool` remain text results; the CLI maps `Error:` text to a failed process.
- `install-skill` validates source and target paths and refuses existing targets unless forced.

## Security

- Secrets are accepted only through CLI args or environment variables at runtime.
- No real token is added to docs, tests, committed task artifacts, or skill content.
- `.gitignore` covers `.env*`, local runtime state, and generated staging.

## Rollback

If a subcommand path breaks, legacy flat-flag handling can remain as the fallback. The skill package and installer are additive and can be removed without changing indexing or MCP internals.
