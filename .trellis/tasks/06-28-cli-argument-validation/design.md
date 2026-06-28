# Design

## Boundaries

- `src/main.rs` owns CLI argument structs, credential/config-file loading, mode dispatch, and top-level command output.
- `src/config.rs` owns normalized runtime `Config` and should reject invalid normalized option values that can also come from future non-CLI sources.
- Indexing internals should not need to defend user-facing zero values except for compatibility with older code paths already present.

## Numeric Validation Contract

Use early validation in `Config::new` so all modes that construct a normal ACE config share the same contract:

- `max_lines_per_blob`, `upload_timeout`, `upload_concurrency`, and `retrieval_timeout` must be positive when supplied.
- Omitted values keep the existing defaults/adaptive behavior.
- Error messages should name the CLI-facing flag, for example `--upload-timeout must be greater than 0`.

This keeps validation consistent across `mcp`, `index`, `search`, `enhance`, and legacy modes.

## Config Path Contract

`load_file_config` should distinguish two cases:

- Explicit `--config <path>`:
  - Missing path is an error.
  - Existing malformed/unreadable file continues to be an error with path context.
- Implicit default config path:
  - Missing path remains allowed and falls back to environment variables.

This preserves backward compatibility for users who have not created a config file while making explicit user input strict.

## Legacy Index Output

`run_index` already supports summary printing through `print_summary`. Legacy `--index-only` should call `run_index(..., true)` so successful runs are visible. Error and partial handling remain unchanged.

## Documentation

Update user-facing docs and the installed skill source to say:

- Explicit `--config` must point to an existing TOML file; the default config may be absent.
- Numeric override flags must be positive integers.
- Legacy `--index-only` prints an indexing summary.

## Compatibility

- Positive numeric flags are unchanged.
- Default config fallback remains unchanged.
- Scripts or MCP configs that explicitly point to a missing config file will now fail fast; this is intentional because the explicit path is likely a configuration mistake.
