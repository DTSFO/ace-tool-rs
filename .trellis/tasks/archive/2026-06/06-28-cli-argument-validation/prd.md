# Validate CLI numeric arguments and config path

## Goal

Make CLI argument validation fail early and consistently for invalid numeric configuration and explicitly missing config files, while preserving default config-file fallback and improving legacy `--index-only` usability.

## Background

- `--max-lines-per-blob 0` is currently accepted and later normalized by index chunking guards, which hides invalid user input.
- `--upload-concurrency 0` is currently accepted and can lead to ineffective or confusing indexing behavior.
- `--retrieval-timeout 0` currently fails during retrieval request execution rather than at CLI/config construction time.
- `--upload-timeout 0` currently fails as an upload batch failure rather than a parameter validation failure.
- `--config <missing>` currently falls through as an empty config. Default config-file absence should remain allowed, but an explicitly supplied missing path should be treated as invalid input.
- Legacy `--index-only` succeeds without human-readable output. Subcommand `index` already prints a summary, so legacy mode should do the same for usability.

## Requirements

1. Reproduce the reported behaviors before changing code.
2. Reject these explicitly supplied numeric values before indexing/search/enhance/MCP work starts:
   - `--max-lines-per-blob 0`
   - `--upload-concurrency 0`
   - `--upload-timeout 0`
   - `--retrieval-timeout 0`
3. Keep positive numeric values working as before.
4. Treat `--config <missing>` as an error that mentions the missing config path.
5. Preserve fallback behavior when no `--config` flag is provided and the default config file is absent.
6. Make legacy `--index-only` print the same successful summary as the `index` subcommand.
7. Update docs and bundled skill guidance for strict explicit config handling and positive numeric values.
8. Do not commit credentials, `.ace-tool/`, target artifacts, or local user config files.

## Acceptance Criteria

- [x] Reproduction notes capture current behavior for zero numeric flags, explicit missing config, and silent legacy `--index-only`.
- [x] Unit tests cover zero-value rejection for every affected numeric flag.
- [x] Unit tests cover explicit missing `--config` failure and absent default config fallback.
- [x] Smoke tests show invalid numeric values and explicit missing config exit before local indexing side effects.
- [x] Legacy `--index-only` prints a success summary on successful indexing.
- [x] README, README-zh-CN, npm README snippets, and `skills/ace-tool-rs/SKILL.md` document the relevant behavior.
- [x] `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `node npm/ace-tool-rs/run.js --help`, and local CLI smoke commands pass.
- [ ] Changes are committed and pushed to `origin/master`.

## Verification Notes

- Pre-fix reproduction showed `--max-lines-per-blob 0` and `--upload-concurrency 0` were accepted and indexed successfully, `--retrieval-timeout 0` failed during request execution, explicit missing `--config` still allowed MCP initialization, and legacy `--index-only` exited 0 with zero stdout bytes.
- Post-fix smoke showed all four zero-value flags exit 1 with `must be greater than 0` and no `.ace-tool/` side effect in a fresh project.
- Post-fix explicit missing `--config` exits 1 with `config file does not exist: <path>`.
- Post-fix legacy `--index-only` exits 0 and prints the same `Indexed ...` / `total_blobs=...` summary as the `index` subcommand.

## Out Of Scope

- Adding config-file support for numeric options.
- Changing remote API retry/backoff behavior beyond rejecting invalid CLI values earlier.
- Removing legacy top-level `--index-only`.
