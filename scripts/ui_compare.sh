#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT/dev/stage-2/generated/ui-review"
TOOLS_DIR="/tmp/wishing-ui-tools"
PORT="${PORT:-8133}"
BASE_URL="http://127.0.0.1:${PORT}"
THRESHOLD="${THRESHOLD:-0.12}"

mkdir -p "$OUT_DIR"

if [ ! -f "$TOOLS_DIR/package.json" ]; then
  mkdir -p "$TOOLS_DIR"
  (
    cd "$TOOLS_DIR"
    npm init -y >/dev/null 2>&1
    npm install playwright-core@1.52.0 >/dev/null
  )
elif [ ! -d "$TOOLS_DIR/node_modules/playwright-core" ]; then
  (
    cd "$TOOLS_DIR"
    npm install playwright-core@1.52.0 >/dev/null
  )
fi

cleanup() {
  if [ -n "${SERVER_PID:-}" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

(
  cd "$ROOT"
  dx serve --platform web --port "$PORT" -p taled-editor
) >"$OUT_DIR/server.log" 2>&1 &
SERVER_PID=$!

for _ in $(seq 1 60); do
  if curl -fsS "$BASE_URL" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

screens=(dashboard editor tilesets layers objects settings)
references=(
  "unnamed (5) (3).jpg"
  "unnamed (5) (1).jpg"
  "unnamed (5) (2).jpg"
  "unnamed (5) (4).jpg"
  "unnamed (5) (5).jpg"
  "unnamed (5) (6).jpg"
)

status=0
for index in "${!screens[@]}"; do
  screen="${screens[$index]}"
  reference_name="${references[$index]}"
  runtime_png="$OUT_DIR/${screen}.png"
  diff_png="$OUT_DIR/${screen}-diff.png"
  composite_png="$OUT_DIR/${screen}-compare.png"
  report_json="$OUT_DIR/${screen}.json"
  reference_path="$ROOT/dev/stage-2/references/ui-images/${reference_name}"

  NODE_PATH="$TOOLS_DIR/node_modules" \
    node "$ROOT/scripts/ui_capture_review.js" \
    --base-url "$BASE_URL" \
    --screen "$screen" \
    --out "$runtime_png"

  if ! python3 "$ROOT/scripts/ui_compare.py" \
    "$screen" \
    "$runtime_png" \
    "$reference_path" \
    "$diff_png" \
    "$composite_png" \
    "$report_json" \
    "$THRESHOLD"; then
    status=1
  fi
done

exit "$status"
