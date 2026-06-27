# Persistence and Index Data Guidelines

This project does not use a database, ORM, migrations, or SQL schema. Persistent backend state is a project-local `.ace-tool` directory containing the index cache and optional HTTP logs. Treat this file as guidance for local persistence and index data formats.

## Storage Locations

- Index cache path is resolved by `src/utils/project_detector.rs::get_index_file_path`.
- The default index file is `.ace-tool/index.bin` under the indexed project root.
- Optional HTTP request logs are written to `.ace-tool/http_requests.log` by `src/http_logger.rs`.
- `.ace-tool` is included in default exclude patterns in `src/config.rs` and is added to `.gitignore` by project helper code.

Reference files:
- `src/utils/project_detector.rs`
- `src/index/manager.rs`
- `src/http_logger.rs`
- `tests/index_test.rs`
- `tests/utils_test.rs`

## Index Format

The current index format is binary bincode, not JSON. `src/index/manager.rs` defines:

- `CURRENT_INDEX_VERSION: u32 = 2`
- `MAX_INDEX_BYTES: u64 = 256 * 1024 * 1024`
- `IndexData { version, config_hash, entries }`
- `FileEntry { mtime_secs, mtime_nanos, size, blob_hashes }`

`IndexManager::save_index` serializes `IndexData` with a bincode size limit, writes to `index.bin.tmp`, and renames to `index.bin`. On Windows it removes the existing target before rename. Keep this atomic write pattern for any future index-cache writes.

`IndexManager::load_index` is intentionally tolerant. It returns an empty index and logs when:

- the file is missing,
- metadata or read fails,
- the file is larger than `MAX_INDEX_BYTES`,
- bincode deserialization fails,
- the stored `version` or `config_hash` does not match current expectations.

Do not make cache load failures fatal unless the product behavior explicitly changes. Search can rebuild the index.

## Configuration Fingerprints

`calculate_config_hash` currently hashes only `max_lines_per_blob`, because that setting affects blob splitting and hash calculation. If a new setting changes which blobs are generated or how hashes are calculated, include it in the config hash and add tests like `tests/index_test.rs::test_config_hash_changes_with_max_lines`.

Avoid adding unrelated runtime settings to the hash. A config hash mismatch forces a rebuild, so only index-affecting settings belong there.

## File Discovery and Ignore Rules

Index file discovery combines:

- built-in text extensions and filenames from `src/config.rs`,
- built-in exclude patterns from `src/config.rs`,
- `.gitignore` and `.aceignore` rules parsed by `ignore::gitignore`,
- `walkdir` traversal with `follow_links(false)`.

The standalone path collector in `src/index/manager.rs` is used inside `tokio::task::spawn_blocking` so directory walking and filesystem metadata work do not block the async runtime. Keep heavy filesystem scanning in blocking tasks.

Reference tests:
- `tests/config_test.rs::test_default_text_extensions_contains_common_types`
- `tests/config_test.rs::test_default_exclude_patterns_contains_common_dirs`
- `tests/index_test.rs::test_collect_files_excludes_binary_extensions`
- `tests/index_test.rs::test_collect_files_excludes_directories`

## Blob Identity and Chunking

Blob identity is SHA-256 over both the normalized relative path and blob content:

```rust
hasher.update(path.as_bytes());
hasher.update(content.as_bytes());
```

This means a rename changes blob identity even if content is unchanged. Preserve that behavior unless the server-side contract changes.

Large text files are split by line count using `max_lines_per_blob`, defaulting to 800 when the configured value is zero. Chunk paths use:

```text
<relative_path>#chunk<index>of<count>
```

Reference tests:
- `tests/index_test.rs::test_calculate_blob_name`
- `tests/index_test.rs::test_split_file_content_small_file`
- `tests/index_test.rs::test_split_file_content_large_file`

## Mtime Cache Rules

The cache hit logic in `process_file_standalone` uses:

- `mtime_secs`, `mtime_nanos`, and file size on high-precision filesystems.
- `mtime_secs` and size plus a content hash verification when `mtime_nanos == 0`.

On transient metadata/read errors, the code preserves the old cache entry when possible. If the file is genuinely deleted (`ErrorKind::NotFound`), the entry is not preserved because the new index is rebuilt from current results rather than extended.

Do not replace this with a simple "extend old index" approach. `index_project` builds a fresh `IndexData` from processed files so deleted files are removed.

## Content Safety

Before indexing content:

- enforce `MAX_BLOB_SIZE` before and after reading,
- decode as UTF-8, GBK, GB18030, or Windows-1252 before falling back to lossy UTF-8,
- skip binary-like content,
- strip unsafe control characters while preserving newline, carriage return, and tab,
- normalize stored relative paths to forward slashes.

Reference functions:
- `IndexManager::read_file_with_encoding`
- `IndexManager::decode_bytes_with_encoding`
- `IndexManager::is_binary_content`
- `IndexManager::sanitize_content`
- `normalize_relative_path` in `src/utils/path_normalizer.rs`

## Upload and Search Data Flow

`IndexManager::index_project` follows this sequence:

1. Collect indexable file paths in a blocking task.
2. Load the old index.
3. Process files in parallel with `rayon` inside a blocking task.
4. Build a new index from processed results.
5. Upload only new blobs with `AdaptiveStrategy`.
6. Save the new index.
7. Return `success`, `partial`, or `error` with `IndexStats`.

`IndexManager::search_context` always calls `index_project` before reading blob hashes and sending `/agents/codebase-retrieval`.

If a new feature needs indexed blobs, reuse `IndexManager` instead of reading `.ace-tool/index.bin` directly.

## Current Compatibility Note

`PromptEnhancer::load_blob_names` in `src/enhancer/prompt_enhancer.rs` still attempts to read the index file as JSON text. The current index writer stores bincode. Treat direct index reading outside `IndexManager` as legacy behavior and avoid copying it into new code.

## Anti-Patterns

- Do not introduce SQL, migrations, or ORM abstractions for current index state.
- Do not store generated index files outside `.ace-tool` without updating `project_detector`, ignore patterns, docs, and tests.
- Do not parse `.gitignore` or `.aceignore` manually; use the existing `ignore` crate path.
- Do not let cache deserialization failures crash indexing.
- Do not assume stored paths use platform-specific separators.
