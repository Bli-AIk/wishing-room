#!/usr/bin/env bash
# tokei_check.sh — local lint for file size / module style / clippy exception hygiene
# Usage:
#   ./tokei_check.sh [max_total_lines] [max_code_lines] [search_dir]
#
# Defaults:
#   max_total_lines = 800
#   max_code_lines  = 500
#   search_dir      = .

set -euo pipefail

if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    CYAN=''
    BOLD=''
    RESET=''
fi

MAX_TOTAL_LINES="${1:-800}"
MAX_CODE_LINES="${2:-500}"
SEARCH_DIR="${3:-.}"

errors=0

mod_files=$(find "$SEARCH_DIR" \
    -path '*/target' -prune -o \
    -path '*/examples' -prune -o \
    -name 'mod.rs' -type f -print 2>/dev/null || true)
if [ -n "$mod_files" ]; then
    echo -e "${RED}${BOLD}Error:${RESET} Found mod.rs files. Use Rust 2018+ module naming instead:"
    while IFS= read -r file; do
        [ -z "$file" ] && continue
        echo -e "  ${YELLOW}$file${RESET}"
    done <<< "$mod_files"
    errors=1
fi

rust_files=$(find "$SEARCH_DIR" \
    -path '*/target' -prune -o \
    -path '*/examples' -prune -o \
    -name '*.rs' -type f -print 2>/dev/null || true)
if [ -n "$rust_files" ]; then
    while IFS= read -r file; do
        [ -z "$file" ] && continue
        lines=$(wc -l < "$file")
        if [ "$lines" -gt "$MAX_TOTAL_LINES" ]; then
            echo -e "${RED}${BOLD}Error:${RESET} ${YELLOW}$file${RESET} has ${CYAN}$lines${RESET} total lines (max ${CYAN}$MAX_TOTAL_LINES${RESET})"
            errors=1
        fi
    done <<< "$rust_files"
fi

tokei_report=$(tokei "$SEARCH_DIR" --output json --files 2>/dev/null || true)
if [ -n "$tokei_report" ]; then
    over_code_limit=$(printf '%s' "$tokei_report" | jq -r --argjson max "$MAX_CODE_LINES" '
        .Rust.reports[]?
        | select((.name | contains("/target/") | not) and (.name | contains("/examples/") | not))
        | select(.stats.code > $max)
        | "\(.name)\t\(.stats.code)"
    ' 2>/dev/null || true)
    if [ -n "$over_code_limit" ]; then
        while IFS=$'\t' read -r file code_lines; do
            [ -z "$file" ] && continue
            echo -e "${RED}${BOLD}Error:${RESET} ${YELLOW}$file${RESET} has ${CYAN}$code_lines${RESET} lines of code via tokei (max ${CYAN}$MAX_CODE_LINES${RESET})"
            errors=1
        done <<< "$over_code_limit"
    fi
fi

allow_hits=$(grep -rn 'allow(clippy::' "$SEARCH_DIR" --include="*.rs" --exclude-dir=target 2>/dev/null || true)
if [ -n "$allow_hits" ]; then
    echo -e "${RED}${BOLD}Error:${RESET} Found allow(clippy::...). Use clippy.toml for global config or #[expect] for individual cases:"
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        echo -e "  ${YELLOW}$line${RESET}"
    done <<< "$allow_hits"
    errors=1
fi

expect_no_reason=$(grep -rl '#\[expect(clippy::' "$SEARCH_DIR" --include="*.rs" --exclude-dir=target 2>/dev/null | \
    xargs -r awk '
    prev_expect && FILENAME != prev_file {
        print prev_loc ": " prev_line
        prev_expect = 0
    }
    prev_expect {
        if (/\/\/ reason:/) { prev_expect = 0; next }
        print prev_loc ": " prev_line
        prev_expect = 0
    }
    /#\[expect\(clippy::/ {
        if (/\/\/ reason:/) next
        prev_expect = 1; prev_file = FILENAME; prev_loc = FILENAME ":" FNR; prev_line = $0
    }
    END { if (prev_expect) print prev_loc ": " prev_line }
    ' 2>/dev/null || true)
if [ -n "$expect_no_reason" ]; then
    echo -e "${RED}${BOLD}Error:${RESET} Found #[expect(clippy::...)] without // reason: comment:"
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        echo -e "  ${YELLOW}$line${RESET}"
    done <<< "$expect_no_reason"
    errors=1
fi

if [ "$errors" -ne 0 ]; then
    exit 1
fi

echo -e "${GREEN}${BOLD}Tokei OK:${RESET} All Rust files under ${CYAN}$MAX_TOTAL_LINES${RESET} total lines and ${CYAN}$MAX_CODE_LINES${RESET} lines of code, no mod.rs found."
echo -e "${GREEN}${BOLD}Lint OK:${RESET} No #[allow(clippy::...)] found, all #[expect] have reasons."
