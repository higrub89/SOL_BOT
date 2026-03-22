#!/usr/bin/env bash
# Configures skills for AI coding assistants (agentskills.io standard)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🔧 Setting up Agent Skills for bot_trading..."

tools=(
  ".claude/skills"
  ".codex/skills"
  ".github/skills"
  ".gemini/skills"
)

for tool_dir in "${tools[@]}"; do
  full_path="$ROOT_DIR/$tool_dir"
  mkdir -p "$(dirname "$full_path")"
  if [ -L "$full_path" ]; then
    rm "$full_path"
  fi
  ln -s "$SCRIPT_DIR" "$full_path"
  echo "  ✓ $tool_dir → skills/"
done

echo ""
echo "✅ Skills configured. Restart your AI coding assistant to load them."
echo "   Gemini CLI: enable 'experimental.skills' in settings."
