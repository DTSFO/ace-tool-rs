# ace-tool-rs

CLI, assistant skill, and MCP server for codebase indexing, semantic search, and prompt enhancement.

## Installation

```bash
# Install globally
npm install -g ace-tool-rs

# Or run directly with npx
npx ace-tool-rs --help
```

## How It Works

This package uses platform-specific optional dependencies to provide pre-built binaries. When you install `ace-tool-rs`, npm automatically downloads the correct binary for your platform.

### Supported Platforms

| Platform | Architecture | Package |
|----------|--------------|---------|
| macOS    | x64, ARM64   | `@ace-tool-rs/darwin-universal` |
| Linux    | x64          | `@ace-tool-rs/linux-x64` |
| Linux    | ARM64        | `@ace-tool-rs/linux-arm64` |
| Windows  | x64          | `@ace-tool-rs/win32-x64` |
| Windows  | ARM64        | `@ace-tool-rs/win32-arm64` |

## Usage

```bash
ace-tool-rs --help
mkdir -p ~/.config/ace-tool-rs
$EDITOR ~/.config/ace-tool-rs/config.toml
ace-tool-rs mcp --transport lsp --no-webbrowser-enhance-prompt
ace-tool-rs index --project-root /path/to/project
ace-tool-rs search --project-root /path/to/project --query "Where is auth handled?"
ace-tool-rs enhance --prompt "Add request logging"
ace-tool-rs install-skill --agents codex,claude,pi
```

`index` requires an existing project directory. For valid projects it writes
`.ace-tool/` and may update the root `.gitignore`. Treat semantic search output
as locator guidance and verify exact implementation details in local files.
Explicit `--config <path>` values must point to an existing TOML file, and
numeric override flags such as `--upload-timeout` must be positive integers.

## Troubleshooting

### Binary not found

If the platform-specific package failed to install, you can install it manually:

```bash
# For Linux x64
npm install @ace-tool-rs/linux-x64

# For macOS
npm install @ace-tool-rs/darwin-universal

# For Windows x64
npm install @ace-tool-rs/win32-x64
```

### Alternative installation

If you have Rust installed, you can build from source:

```bash
cargo install ace-tool-rs
```

## License

GPL-3.0-only

For commercial use, please contact missdeer@gmail.com for licensing options.

## Verifying Downloads

Each GitHub release includes a `SHA256SUMS` file for integrity verification:

```bash
# Download the binary and checksum file
curl -LO https://github.com/missdeer/ace-tool-rs/releases/latest/download/ace-tool-rs_Linux_x86_64.tar.gz
curl -LO https://github.com/missdeer/ace-tool-rs/releases/latest/download/SHA256SUMS

# Verify the checksum
sha256sum -c SHA256SUMS --ignore-missing
```
