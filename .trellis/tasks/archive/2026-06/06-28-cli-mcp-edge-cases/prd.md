# Reproduce and fix CLI/MCP edge cases

## Goal

Reproduce the reported CLI/MCP edge cases, fix behavior that is clearly incorrect, and document operational caveats that are inherent to the current remote retrieval/indexing design.

## Background

- `ace-tool-rs index --project-root <nonexistent>` currently appears to create the missing project directory, `.ace-tool/`, and `.gitignore`, then returns success such as `Indexed 1 blobs`. This is a parameter validation bug.
- `enhance_prompt` through MCP can hang in unattended clients when the server is started without `--no-webbrowser-enhance-prompt`, because the default local browser review flow waits for user interaction.
- Some normal semantic search snippets can look summarized or stale compared with local files, while canary/exact queries are accurate. Search results should be treated as navigation clues, not source-of-truth code.
- Indexing writes `.ace-tool/` and can update `.gitignore`. That side effect should be explicit in docs/skill guidance.

## Requirements

1. Reproduce the nonexistent `--project-root` indexing behavior before changing it.
2. Make `index --project-root <path>` fail before any local side effects when `<path>` does not exist or is not a directory.
3. Preserve valid indexing behavior for existing directories and legacy `--index-only`.
4. Optimize MCP unattended usage so `enhance_prompt` does not unexpectedly wait on the browser review UI in MCP server mode.
5. Keep one-shot CLI `enhance` behavior compatible unless the user explicitly opts out with `--no-webbrowser-enhance-prompt`.
6. Update user-facing docs and bundled skill instructions:
   - MCP examples should use `--no-webbrowser-enhance-prompt` for unattended clients.
   - Search results are locator hints; verify final implementation against local files.
   - `index` writes `.ace-tool/` and may update `.gitignore`.
7. Do not commit live endpoint/token values or generated `.ace-tool/` state.

## Acceptance Criteria

- [x] A pre-fix or current reproduction demonstrates the nonexistent project-root behavior.
- [x] `ace-tool-rs index --project-root <missing>` exits non-zero and does not create the missing directory.
- [x] A focused automated test covers missing and file-as-project-root validation for the `index` path.
- [x] MCP server config defaults to direct prompt enhancement or docs/examples make the unattended requirement explicit enough that MCP `enhance_prompt` avoids browser wait.
- [x] Docs and `skills/ace-tool-rs/SKILL.md` mention search-result verification and index side effects.
- [x] `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `node npm/ace-tool-rs/run.js --help`, and a local smoke command pass.
- [ ] Changes are committed and pushed to `origin/master`.

## Verification Notes

- Pre-fix reproduction with installed CLI: `index --project-root /tmp/.../missing-project` exited 0, printed `Indexed 1 blobs`, and created the missing directory, `.ace-tool/`, and `.gitignore`.
- Post-fix smoke with debug and installed release binaries: missing project root exits 1 with `project root does not exist: ...` and leaves the missing path absent.
- MCP line-transport smoke without `--no-webbrowser-enhance-prompt` returned `enhance_prompt` JSON-RPC output quickly, confirming MCP no-browser default behavior.

## Out Of Scope

- Changing remote retrieval/model behavior for summarized or stale snippets.
- Removing `.ace-tool/` as the local index/cache state directory.
- Preventing `.gitignore` updates for valid indexing runs.
- Changing third-party provider APIs.
