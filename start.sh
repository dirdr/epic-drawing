#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$SCRIPT_DIR/backend"
FRONTEND_DIR="$SCRIPT_DIR/frontend"
WASM_OUT_DIR="$FRONTEND_DIR/src/lib/wasm"

command -v wasm-pack >/dev/null 2>&1 || {
	echo "error: wasm-pack is not installed (https://rustwasm.github.io/wasm-pack/installer/)" >&2
	exit 1
}
command -v pnpm >/dev/null 2>&1 || {
	echo "error: pnpm is not installed (https://pnpm.io/installation)" >&2
	exit 1
}

echo "==> Building backend to wasm"
(cd "$BACKEND_DIR" && wasm-pack build --target web --release --out-dir "$WASM_OUT_DIR")

echo "==> Installing frontend dependencies"
(cd "$FRONTEND_DIR" && pnpm install --frozen-lockfile)

echo "==> Starting frontend dev server"
(cd "$FRONTEND_DIR" && exec pnpm run dev "$@")
