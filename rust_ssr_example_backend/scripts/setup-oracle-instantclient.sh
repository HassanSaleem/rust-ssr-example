#!/usr/bin/env bash
# Idempotent installer for Oracle Instant Client (Basic Light), scoped to this
# checkout so no host-level/sudo changes are required to run the backend.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(dirname "$SCRIPT_DIR")"
INSTALL_DIR="$BACKEND_DIR/.instantclient"
ZIP_URL="https://download.oracle.com/otn_software/linux/instantclient/instantclient-basiclite-linuxx64.zip"
ZIP_PATH="$BACKEND_DIR/.instantclient-download.zip"

# Ubuntu's libaio1t64 package only ships libaio.so.1t64 (time64 SONAME
# transition), not the libaio.so.1 name Instant Client expects. Link it
# locally so no system-wide/sudo change is needed.
link_libaio_compat() {
    if ldconfig -p 2>/dev/null | grep -q ' libaio\.so\.1 '; then
        return
    fi
    local system_libaio
    system_libaio="$(ldconfig -p 2>/dev/null | grep 'libaio\.so\.1t64' | awk '{print $NF}' | head -1)"
    if [ -n "$system_libaio" ]; then
        mkdir -p "$INSTALL_DIR"
        ln -sf "$system_libaio" "$INSTALL_DIR/libaio.so.1"
        echo "Linked $INSTALL_DIR/libaio.so.1 -> $system_libaio"
    else
        echo "Warning: libaio was not found. Oracle Instant Client needs it at runtime."
        echo "Install it with: sudo apt install -y libaio1t64 || sudo apt install -y libaio1"
    fi
}

if ldconfig -p 2>/dev/null | grep -q libclntsh.so; then
    echo "Oracle Instant Client already available on the system linker path."
    link_libaio_compat
    exit 0
fi

if [ -d "$INSTALL_DIR" ] && compgen -G "$INSTALL_DIR/libclntsh.so*" > /dev/null; then
    echo "Oracle Instant Client already installed at $INSTALL_DIR"
    link_libaio_compat
    exit 0
fi

echo "Downloading Oracle Instant Client (Basic Light) to $INSTALL_DIR ..."
mkdir -p "$INSTALL_DIR"
if command -v curl > /dev/null; then
    curl -fL -o "$ZIP_PATH" "$ZIP_URL"
elif command -v wget > /dev/null; then
    wget -O "$ZIP_PATH" "$ZIP_URL"
else
    echo "Need curl or wget to download Oracle Instant Client." >&2
    exit 1
fi

echo "Extracting..."
if command -v unzip > /dev/null; then
    unzip -q -o "$ZIP_PATH" -d "$INSTALL_DIR"
else
    # zipfile.extractall() doesn't preserve symlinks (the Instant Client zip
    # uses them for versioned .so names), so recreate them manually here.
    python3 - "$ZIP_PATH" "$INSTALL_DIR" <<'PYEOF'
import sys
import zipfile
import os
import stat

zip_path, dest_dir = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(zip_path) as zf:
    for info in zf.infolist():
        mode = info.external_attr >> 16
        target_path = os.path.join(dest_dir, info.filename)
        if stat.S_ISLNK(mode):
            os.makedirs(os.path.dirname(target_path), exist_ok=True)
            link_target = zf.read(info.filename).decode()
            if os.path.lexists(target_path):
                os.remove(target_path)
            os.symlink(link_target, target_path)
        else:
            zf.extract(info, dest_dir)
PYEOF
fi
rm -f "$ZIP_PATH"

# The zip extracts into a versioned subfolder (instantclient_23_*); flatten it.
INNER_DIR="$(find "$INSTALL_DIR" -maxdepth 1 -mindepth 1 -type d -name 'instantclient_*' | head -1)"
if [ -n "$INNER_DIR" ]; then
    shopt -s dotglob
    mv "$INNER_DIR"/* "$INSTALL_DIR"/
    rmdir "$INNER_DIR"
    shopt -u dotglob
fi

link_libaio_compat

echo "Oracle Instant Client installed at $INSTALL_DIR"
