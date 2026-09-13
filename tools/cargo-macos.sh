#!/usr/bin/env bash
set -euo pipefail

if command -v cargo >/dev/null 2>&1; then
    exec cargo "$@"
fi

if command -v brew >/dev/null 2>&1; then
    rustup_bin="$(brew --prefix rustup 2>/dev/null)/bin"
    if [[ -x "$rustup_bin/cargo" ]]; then
        export PATH="$rustup_bin:$PATH"
        exec "$rustup_bin/cargo" "$@"
    fi
fi

echo "Cargo is required. Install Rust with rustup, then rerun make setup." >&2
exit 1
