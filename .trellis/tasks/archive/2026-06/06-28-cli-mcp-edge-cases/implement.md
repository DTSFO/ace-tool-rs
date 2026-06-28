# Implementation Plan

1. Reproduce the reported nonexistent `index --project-root` behavior using a temp path and record observed side effects.
2. Read relevant backend specs before editing.
3. Implement `validate_project_root` in `src/main.rs` and call it at the top of `run_index`.
4. Add focused tests for missing and file project roots.
5. Adjust MCP startup defaults so `mcp` mode uses no-browser enhancement unless the user explicitly configured prompt UI behavior.
6. Update README, README-zh-CN, npm README snippets, and bundled skill guidance for unattended MCP, search caveats, and indexing side effects.
7. Run verification:
   - `cargo fmt --check`
   - `cargo test`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `node npm/ace-tool-rs/run.js --help`
   - local smoke for missing project root and valid config-file search.
8. Scan for live endpoint/token strings outside local ignored files.
9. Commit and push to `origin/master`.

## Risk Points

- Do not move path validation into code paths that should accept missing local state such as `.ace-tool/`.
- Do not break one-shot interactive `enhance` behavior.
- Keep stdout safe for MCP mode; diagnostics remain stderr/logs.
