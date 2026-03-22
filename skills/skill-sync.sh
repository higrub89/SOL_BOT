#!/bin/bash
set -euo pipefail

SKILLS_DIR="$(dirname "$0")"
AGENTS_MD="$(dirname "$SKILLS_DIR")/AGENTS.md"

TEMP_FILE=$(mktemp)

echo "| Action | Skill |" > "$TEMP_FILE"
echo "|--------|-------|" >> "$TEMP_FILE"

# Find all SKILL.md
for skill_file in "$SKILLS_DIR"/*/SKILL.md; do
    if [ ! -f "$skill_file" ]; then continue; fi
    
    # Parse YAML frontmatter
    name=$(awk '/^name:/{print $2}' "$skill_file")
    
    # If no name, skip
    if [ -z "$name" ]; then continue; fi
    
    # Get auto-invokes block
    awk '/^auto-invokes:/{flag=1; next} /^---/{if(flag) {flag=0; exit}} flag {if ($1 == "-") {sub(/^- /, ""); print $0}}' "$skill_file" | while read -r action; do
        if [ ! -z "$action" ]; then
            echo "| $action | \`$name\` |" >> "$TEMP_FILE"
        fi
    done
done

# Sort alphabetically by Action
(head -n 2 "$TEMP_FILE" && tail -n +3 "$TEMP_FILE" | sort -f) > "${TEMP_FILE}.sorted"

# Replace table in AGENTS.md between marker comments
awk -v table="$(cat "${TEMP_FILE}.sorted")" '
BEGIN { p = 1 }
/^<!-- AUTO-INVOKE-START -->/ { print; print table; p = 0; next }
/^<!-- AUTO-INVOKE-END -->/ { p = 1 }
p { print }
' "$AGENTS_MD" > "${AGENTS_MD}.tmp"

mv "${AGENTS_MD}.tmp" "$AGENTS_MD"
rm "$TEMP_FILE" "${TEMP_FILE}.sorted"

echo "✅ AGENTS.md updated with latest Auto-invoke triggers"
