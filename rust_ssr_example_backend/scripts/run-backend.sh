#!/usr/bin/env bash
# Ensures the Oracle Instant Client is present, then runs the backend with
# LD_LIBRARY_PATH pointed at it. Extra args are forwarded to `cargo run`.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"
INSTANT_CLIENT_DIR="$BACKEND_DIR/.instantclient"

"$SCRIPT_DIR/setup-oracle-instantclient.sh"

if [ -d "$INSTANT_CLIENT_DIR" ]; then
    export LD_LIBRARY_PATH="$INSTANT_CLIENT_DIR:${LD_LIBRARY_PATH:-}"
fi

cd "$BACKEND_DIR"
exec cargo run "$@"
