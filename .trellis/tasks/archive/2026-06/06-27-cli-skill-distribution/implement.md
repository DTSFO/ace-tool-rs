# Implementation Plan

## Checklist

1. Refactor `src/main.rs` CLI parsing to support subcommands and legacy flat flags.
2. Add shared helpers in `src/main.rs` for config construction, index execution, search execution, enhance execution, and skill installation.
3. Add tests for CLI parsing/mode compatibility and install path behavior where practical.
4. Add `skills/ace-tool-rs/SKILL.md` and `skills/ace-tool-rs/agents/openai.yaml`.
5. Update `.gitignore` for runtime, local config, generated staging, and installed binary artifacts.
6. Update README and npm README with CLI subcommand and skill installation examples.
7. Build release binary and install it locally on PATH for this machine.
8. Run `ace-tool-rs install-skill --agents codex,claude,pi --force`.
9. Run verification:
   - `cargo fmt --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test`
   - relevant `node npm/ace-tool-rs/run.js --help` or direct binary help check
   - live smoke test against the provided endpoint/key
10. Review `git diff` for accidental secrets or unrelated churn.
11. Commit and push to `origin/master`.

## Risk Points

- `src/main.rs`: mode branching can accidentally change legacy behavior.
- stdout/stderr: MCP server mode must keep stdout reserved for JSON-RPC.
- skill installer: must not overwrite unrelated local skills by default.
- docs/tests: must use placeholder credentials only.

## Validation Notes

- Network-dependent live smoke is manual/local and must not be encoded into automated tests.
- Tests should not depend on home directory skill paths.
- The user-provided test key is only for local execution and must not appear in committed files.
