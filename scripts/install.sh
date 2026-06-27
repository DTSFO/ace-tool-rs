#!/usr/bin/env bash

set -euo pipefail

BINARY_NAME="ace-tool-rs"
INSTALL_DIR="${ACE_TOOL_INSTALL_DIR:-$HOME/.local/bin}"
SKILL_AGENTS="${ACE_TOOL_SKILL_AGENTS:-codex,claude,pi}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_SKILL_DIR="${ACE_TOOL_SKILL_SOURCE:-$REPO_ROOT/skills/ace-tool-rs}"

usage() {
	cat >&2 <<EOF
Usage: scripts/install.sh [path-to-ace-tool-rs]

Installs ace-tool-rs to:
  ${INSTALL_DIR}

Then installs the bundled skill for:
  ${SKILL_AGENTS}

Environment overrides:
  ACE_TOOL_INSTALL_DIR      Binary install directory (default: \$HOME/.local/bin)
  ACE_TOOL_SKILL_AGENTS     Comma-separated agents (default: codex,claude,pi)
  ACE_TOOL_SKILL_SOURCE     Skill source directory (default: repo skills/ace-tool-rs)
  ACE_TOOL_INSTALL_FORCE=1  Replace existing ace-tool-rs skill directories
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
	usage
	exit 0
fi

resolve_binary() {
	if [[ -n "${1:-}" ]]; then
		if [[ ! -x "$1" ]]; then
			echo "Error: binary is not executable: $1" >&2
			exit 1
		fi
		printf '%s\n' "$1"
		return
	fi

	local release_binary="$REPO_ROOT/target/release/$BINARY_NAME"
	if [[ ! -x "$release_binary" ]]; then
		echo "Release binary not found; building with cargo..." >&2
		(cd "$REPO_ROOT" && cargo build --release)
	fi

	if [[ ! -x "$release_binary" ]]; then
		echo "Error: failed to build $release_binary" >&2
		exit 1
	fi

	printf '%s\n' "$release_binary"
}

if [[ ! -d "$SOURCE_SKILL_DIR" || ! -f "$SOURCE_SKILL_DIR/SKILL.md" ]]; then
	echo "Error: skill source not found: $SOURCE_SKILL_DIR" >&2
	exit 1
fi

BINARY_PATH="$(resolve_binary "${1:-}")"

mkdir -p "$INSTALL_DIR"
cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

INSTALL_ARGS=(install-skill --agents "$SKILL_AGENTS" --source "$SOURCE_SKILL_DIR")
if [[ "${ACE_TOOL_INSTALL_FORCE:-}" == "1" ]]; then
	INSTALL_ARGS+=(--force)
fi

"$INSTALL_DIR/$BINARY_NAME" "${INSTALL_ARGS[@]}"

echo "Installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME" >&2
case ":$PATH:" in
*":$INSTALL_DIR:"*) ;;
*) echo "Note: add $INSTALL_DIR to PATH if ace-tool-rs is not found." >&2 ;;
esac
