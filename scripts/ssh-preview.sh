#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
START_PORT="${WISHING_PREVIEW_PORT:-3210}"
SSH_TARGET_HINT="${SSH_TARGET_HINT:-<server ip>}"

pick_port() {
    if ! command -v ss >/dev/null 2>&1; then
        printf '%s\n' "$START_PORT"
        return 0
    fi

    local port
    for port in $(seq "$START_PORT" $((START_PORT + 49))); do
        if ! ss -tlnH "( sport = :${port} )" 2>/dev/null | grep -q .; then
            printf '%s\n' "$port"
            return 0
        fi
    done

    echo "no free preview port found" >&2
    return 1
}

PORT="$(pick_port)"

cat <<EOF
Taled SSH Preview

1. On your local machine, open an SSH tunnel:
   ssh -L ${PORT}:127.0.0.1:${PORT} ${SSH_TARGET_HINT}

2. Then open this URL in your local browser:
   http://127.0.0.1:${PORT}

This command keeps the preview running. Press Ctrl+C here to stop it.
EOF

if [[ "${WISHING_PREVIEW_DRY_RUN:-0}" == "1" ]]; then
    exit 0
fi

cd "${ROOT_DIR}"
exec dx serve \
    --package taled-editor \
    --platform web \
    --addr 127.0.0.1 \
    --port "${PORT}" \
    --open false \
    --interactive false
