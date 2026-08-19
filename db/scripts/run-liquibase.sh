#!/usr/bin/env bash
# Downloads a portable Liquibase distribution scoped to this checkout (the
# snap package is confined and can't see the system JDK), then runs it.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_DIR="$(dirname "$SCRIPT_DIR")"
LIQUIBASE_DIR="$DB_DIR/.liquibase"
LIQUIBASE_VERSION="5.0.3"
TAR_URL="https://github.com/liquibase/liquibase/releases/download/v${LIQUIBASE_VERSION}/liquibase-${LIQUIBASE_VERSION}.tar.gz"
TAR_PATH="$DB_DIR/.liquibase-download.tar.gz"

if [ ! -x "$LIQUIBASE_DIR/liquibase" ]; then
    echo "Downloading Liquibase ${LIQUIBASE_VERSION} to $LIQUIBASE_DIR ..."
    mkdir -p "$LIQUIBASE_DIR"
    if command -v curl > /dev/null; then
        curl -fL -o "$TAR_PATH" "$TAR_URL"
    elif command -v wget > /dev/null; then
        wget -O "$TAR_PATH" "$TAR_URL"
    else
        echo "Need curl or wget to download Liquibase." >&2
        exit 1
    fi
    tar xzf "$TAR_PATH" -C "$LIQUIBASE_DIR"
    rm -f "$TAR_PATH"
fi

cd "$DB_DIR"
exec "$LIQUIBASE_DIR/liquibase" "$@"
