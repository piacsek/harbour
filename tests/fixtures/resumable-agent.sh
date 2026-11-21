#!/bin/sh
config="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
id="0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b"
mkdir -p "$config/sessions" "$config/projects/-home-acme-api"
printf '{"status":"idle","sessionId":"%s"}' "$id" > "$config/sessions/$$.json"
resumed=""
previous=""
for argument do
  [ "$previous" = "--resume" ] && resumed=$argument
  previous=$argument
done
if [ -n "$resumed" ]; then
  echo "acme agent resumed $resumed"
  echo "acme agent args:[$*]"
else
  printf '{"type":"ai-title","aiTitle":"Fix the flaky login test"}\n' > "$config/projects/-home-acme-api/$id.jsonl"
  echo "acme agent started"
fi
exec sleep 600
