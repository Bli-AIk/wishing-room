#!/usr/bin/env bash
set -euo pipefail

sample="${1:-Theater}"
port="${TALED_PERF_PORT:-8155}"
web_root="target/dx/taled-editor/release/web/public"
playwright_root="${TALED_PERF_NODE_ROOT:-/tmp/taled-perf-node}"
playwright_bin="${playwright_root}/node_modules/.bin/playwright"

dx build --platform web -r -p taled-editor >/tmp/taled-perf-build.log 2>&1

mkdir -p "${playwright_root}"
if [[ ! -x "${playwright_bin}" ]]; then
  (
    cd "${playwright_root}"
    if [[ ! -f package.json ]]; then
      npm init -y >/tmp/taled-perf-npm-init.log 2>&1
    fi
    npm install --no-save @playwright/test >/tmp/taled-perf-npm-install.log 2>&1
  )
fi

python3 -m http.server "${port}" --directory "${web_root}" >/tmp/taled-perf-server.log 2>&1 &
server_pid="$!"
cleanup() {
  kill "${server_pid}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

python3 - <<PY
import sys, time, urllib.request
url = "http://127.0.0.1:${port}/"
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
for _ in range(60):
    try:
        with opener.open(url, timeout=1) as response:
            if response.status == 200:
                sys.exit(0)
    except Exception:
        time.sleep(1)
raise SystemExit("perf server not ready")
PY

export NO_PROXY="127.0.0.1,localhost"
export no_proxy="127.0.0.1,localhost"
export NODE_PATH="${playwright_root}/node_modules"
TALED_PERF_BASE_URL="http://127.0.0.1:${port}" \
TALED_PERF_SAMPLE="${sample}" \
"${playwright_bin}" test scripts/perf_probe.spec.js --reporter=line --workers=1
