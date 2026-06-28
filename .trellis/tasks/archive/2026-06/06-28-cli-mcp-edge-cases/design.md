# Design

## Boundaries

- `src/main.rs` owns CLI parsing, mode selection, and top-level path validation before constructing `IndexManager`.
- `src/index/manager.rs` owns indexing internals and may create `.ace-tool/` for a valid project root.
- `src/tools/search_context.rs` already validates project paths for MCP search before constructing an index manager.
- Documentation and `skills/ace-tool-rs/SKILL.md` own operational guidance for MCP unattended use and retrieval caveats.

## Path Validation

Add a small helper in `src/main.rs` for index command project roots:

```rust
fn validate_project_root(project_root: &Path) -> Result<()>
```

Contract:

- Missing path -> `project root does not exist: <path>`.
- Non-directory path -> `project root is not a directory: <path>`.
- Valid directory -> `Ok(())`.

Call the helper in `run_index` before `IndexManager::new`, so missing paths do not cause `.ace-tool/` or `.gitignore` side effects.

## MCP Enhance Unattended Behavior

MCP server mode is normally machine-to-machine. The browser review UI is useful for one-shot CLI use but surprising for MCP tools. The least disruptive change is:

- When running `mcp`, default `no_webbrowser_enhance_prompt` to `true` if the user did not explicitly pass the prompt UI flag.
- Keep `enhance` and legacy `--enhance-prompt` default behavior unchanged.

This preserves interactive CLI behavior while making MCP safe for unattended agents.

## Docs and Skill Guidance

Update examples and caveats:

- MCP examples include or explain `--no-webbrowser-enhance-prompt`.
- `index` side effects: `.ace-tool/` and `.gitignore`.
- Search snippets are retrieval/model output and must be verified against local files before editing or citing exact implementation details.

## Compatibility

- Existing configs that already pass `--no-webbrowser-enhance-prompt` continue working.
- Existing MCP configs that did not pass it become less interactive but more reliable for agents.
- CLI `enhance` users still get the browser review flow by default.
- Missing project-root validation only rejects paths that cannot be valid project roots.
