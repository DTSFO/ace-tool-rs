# Implementation Plan

1. Reproduce current behavior with installed or debug CLI:
   - zero numeric flags,
   - explicit missing config file,
   - legacy `--index-only` successful no-output behavior.
2. Read backend specs before editing.
3. Add positive-value validation in `Config::new`.
4. Change `load_file_config` so explicit missing `--config` errors while implicit default missing config falls back.
5. Change legacy `--index-only` to print summary.
6. Add focused tests in `tests/config_test.rs` and/or `src/main.rs` unit tests.
7. Update README, README-zh-CN, npm READMEs, and `skills/ace-tool-rs/SKILL.md`.
8. Run verification:
   - `cargo fmt --check`
   - `cargo test`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `node npm/ace-tool-rs/run.js --help`
   - local smoke commands for invalid numerics, missing config, and legacy `--index-only` output.
9. Scan for live endpoint/token strings outside ignored local state.
10. Commit, push, archive task, and record journal.

## Risk Points

- Do not break default config fallback for users relying on environment variables.
- Do not move validation into only one CLI subcommand; all normal ACE config construction paths need it.
- Do not print diagnostics to stdout in MCP mode.
