# Convert MCP server to CLI skill distribution

## Goal

Turn the current MCP-first fork into a local CLI plus agent skill distribution while preserving existing MCP compatibility. The result should install on this machine, expose a reusable skill to Codex, Claude, and Pi, and be pushed to the fork remote.

## Background

- The repository is a single Rust crate with npm platform wrappers.
- `src/main.rs` currently exposes MCP server mode plus flat `--index-only` and `--enhance-prompt` flags.
- Existing MCP tools are `search_context` and `enhance_prompt`.
- The npm wrapper forwards stdio to the Rust binary, so stdout must remain protocol-safe in MCP server mode.
- Local assistant skill directories exist at `~/.codex/skills`, `~/.claude/skills`, and `~/.pi/agent/skills`.
- Pi already has an unrelated `ace-context-engine` skill; the new skill must not overwrite it.
- The fork remote is `origin = https://github.com/DTSFO/ace-tool-rs.git`.

## Requirements

1. Preserve MCP server behavior and existing flags so existing configs continue to work.
2. Add first-class CLI subcommands for:
   - running the MCP server,
   - indexing a project,
   - searching a project,
   - enhancing a prompt,
   - installing assistant skills locally.
3. Keep configuration validation consistent with current behavior:
   - ACE indexing/search/server modes require `--base-url` and `--token`.
   - Third-party enhance mode may still read provider credentials from environment variables.
   - Secrets must never be written to committed files.
4. Add a repository-owned skill package that teaches agents when and how to use the local CLI.
5. Add an installer path that installs the skill for Codex, Claude, and Pi on this machine without overwriting unrelated skills.
6. Ensure `.gitignore` covers generated local runtime state, installed binaries, temporary skill staging, and secret-bearing local config.
7. Update user-facing docs to explain CLI subcommands, skill install, and MCP compatibility.
8. Verify against the provided ACE-compatible test endpoint/key locally without committing or documenting the key.
9. Commit and push the completed work to the fork remote.

## Acceptance Criteria

- [ ] `ace-tool-rs --help` shows subcommands while existing flat flag usage still works.
- [ ] `ace-tool-rs mcp --base-url ... --token ...` starts the MCP server with the same transport options as before.
- [ ] `ace-tool-rs index --project-root <path> --base-url ... --token ...` indexes a project and exits with the same success/partial/error semantics as `--index-only`.
- [ ] `ace-tool-rs search --project-root <path> --query <text> --base-url ... --token ...` prints only search results to stdout.
- [ ] `ace-tool-rs enhance --prompt <text> ...` preserves current `--enhance-prompt` behavior.
- [ ] `ace-tool-rs install-skill --agents codex,claude,pi` installs the repo skill into the three local skill directories.
- [ ] Repository docs include CLI and skill setup examples using placeholders, not real tokens.
- [ ] `.gitignore` excludes `.ace-tool/`, build artifacts, logs, env files, and local skill/install staging while leaving source skill files trackable.
- [ ] Rust formatting, clippy, unit/integration tests, and the relevant npm wrapper command pass.
- [ ] A live smoke test using the user-provided endpoint/key succeeds locally.
- [ ] Changes are committed and pushed to `origin/master`.

## Out Of Scope

- Publishing crates.io or npm packages.
- Replacing the remote indexing service.
- Removing MCP protocol support.
- Modifying user secret stores or assistant configs beyond installing the new skill files.
