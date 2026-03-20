#!/usr/bin/env bash
set -euo pipefail

# Build
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"
cargo build --release

# Determine binary path
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
  BIN="$SCRIPT_DIR/target/release/statusline.exe"
else
  BIN="$SCRIPT_DIR/target/release/statusline"
fi

if [[ ! -f "$BIN" ]]; then
  echo "ERROR: Build failed — binary not found at $BIN"
  exit 1
fi

# Find ~/.claude/settings.json
CLAUDE_DIR="${HOME}/.claude"
SETTINGS="${CLAUDE_DIR}/settings.json"

mkdir -p "$CLAUDE_DIR"

# Normalize path for JSON (forward slashes)
BIN_JSON=$(echo "$BIN" | sed 's|\\|/|g')

if [[ -f "$SETTINGS" ]]; then
  # Update or add statusLine using node (available on all platforms with Claude Code)
  node -e "
    const fs = require('fs');
    const s = JSON.parse(fs.readFileSync('$SETTINGS', 'utf8'));
    s.statusLine = { type: 'command', command: '$BIN_JSON' };
    fs.writeFileSync('$SETTINGS', JSON.stringify(s, null, 2) + '\n');
  "
else
  node -e "
    const fs = require('fs');
    const s = { statusLine: { type: 'command', command: '$BIN_JSON' } };
    fs.writeFileSync('$SETTINGS', JSON.stringify(s, null, 2) + '\n');
  "
fi

echo "Done! statusline installed: $BIN_JSON"
echo "Restart Claude Code to apply."
