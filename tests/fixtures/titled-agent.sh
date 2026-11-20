#!/bin/sh
config="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
id="0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b"
mkdir -p "$config/sessions" "$config/projects/-home-acme-api"
printf '{"type":"ai-title","aiTitle":"Fix the flaky login test"}\n' > "$config/projects/-home-acme-api/$id.jsonl"
printf '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"description":"Run the login tests"}}]}}\n' >> "$config/projects/-home-acme-api/$id.jsonl"
printf '{"status":"busy","sessionId":"%s"}' "$id" > "$config/sessions/$$.json"
echo "acme agent titled"
exec sleep 600
