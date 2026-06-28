---
name: ace-tool-rs
description: Use the local ace-tool-rs CLI for ACE semantic codebase search, project indexing, prompt enhancement, and MCP compatibility. Use when exploring unfamiliar code, locating implementations by behavior, preparing codebase context, or when the user asks to use ACE/ace-tool-rs.
---

# ace-tool-rs

Use this skill when semantic codebase context is useful and the local `ace-tool-rs` command is available.

## When To Use

- You do not know which files contain the implementation.
- The user asks where a behavior, workflow, or architectural concept lives.
- You need high-level context before editing a repository.
- The user explicitly asks to use ACE, ace-tool-rs, semantic search, or prompt enhancement.

## When Not To Use

- You need every occurrence of a known identifier or exact string. Use `rg`.
- You already know the exact file to read.
- The task is about git history; use git commands.
- The user asks only to run a local shell command unrelated to codebase context.

## CLI Commands

Check the CLI:

```bash
ace-tool-rs --help
```

The CLI normally reads ACE credentials from:

```text
~/.config/ace-tool-rs/config.toml
```

Use `--base-url` and `--token` only when you need to override that config for a
single command. If you pass `--config <path>` explicitly, the file must exist;
the implicit default config path is the only missing config path that falls back
to environment variables.

Index a project:

```bash
ace-tool-rs index --project-root "$PWD"
```

The project root must already exist. Indexing a valid project writes local
runtime state under `.ace-tool/` and may update the project root `.gitignore`.

Search a project:

```bash
ace-tool-rs search \
  --project-root "$PWD" \
  --query "Where is the authentication flow implemented?"
```

Enhance a prompt:

```bash
ace-tool-rs enhance \
  --prompt "Add a login page" \
  --conversation-history "User: initial request"
```

Run as an MCP server when an MCP client is configured:

```bash
ace-tool-rs mcp --transport lsp --no-webbrowser-enhance-prompt
```

MCP server mode defaults `enhance_prompt` to direct no-browser responses when no
prompt UI flags are supplied. Passing `--no-webbrowser-enhance-prompt` keeps MCP
configs explicit and compatible with older installed versions.

## Query Guidance

Write natural-language behavioral queries, optionally with keywords:

```text
Find where the server handles chunked file upload merging. Keywords: upload chunk merge
```

Use exact search instead for symbols:

```bash
rg "function_name"
```

Treat semantic search snippets as locator hints. Before editing or citing exact
implementation details, verify the relevant local source files with `rg`, `sed`,
or the normal file-reading tools.

## Safety Rules

- Do not print or commit tokens. Keep config files in user-owned secret storage.
- Do not add real credentials to docs, examples, tests, task notes, or shell history files.
- Treat `.ace-tool/` as local runtime state, not source-controlled content.
- Pass positive integers for numeric overrides such as `--max-lines-per-blob`,
  `--upload-timeout`, `--upload-concurrency`, and `--retrieval-timeout`.
- If `ace-tool-rs search` returns an error, report the exact error and fall back to ordinary repository inspection.
