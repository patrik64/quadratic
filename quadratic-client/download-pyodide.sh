#!/bin/bash

set -euo pipefail

# Configuration
PYODIDE_VERSION="0.27.5"
PYODIDE_URL="https://github.com/pyodide/pyodide/releases/download/${PYODIDE_VERSION}/pyodide-${PYODIDE_VERSION}.tar.bz2"
EXPECTED_TARBALL_CHECKSUM="5a866e8f40d5ef46fb1258b4488ac3edc177c7e7884ad042426bbdfb2a56dd64"
TARGET_DIR="public"
EXPECTED_DIRECTORY_CHECKSUM="72b56b07e39e07a516f2abd79a86ef502e4791bc6ce8c0b86b2c424435225a3c"

# Function to calculate pyodide directory checksum
calculate_pyodide_directory_checksum() {
    find "$TARGET_DIR/pyodide" -type f -print0 | \
    LC_ALL=C sort -z | \
    xargs -0 sha256sum | \
    sha256sum | \
    cut -d' ' -f1
}

# Check existing pyodide directory checksum
if [ -d "$TARGET_DIR/pyodide" ]; then
    computed_checksum=$(calculate_pyodide_directory_checksum)
    if [ "$computed_checksum" == "$EXPECTED_DIRECTORY_CHECKSUM" ]; then
        echo "Checksum matches for existing pyodide directory"
        exit 0
    fi

    echo "Error: Checksum mismatch for existing pyodide directory" >&2
    echo "Expected: $EXPECTED_DIRECTORY_CHECKSUM" >&2
    echo "Got:      $computed_checksum" >&2
    rm -rf "$TARGET_DIR/pyodide"
fi

# Download pyodide tarball
echo "Downloading pyodide-${PYODIDE_VERSION}.tar.bz2..."
if ! curl -L -f "$PYODIDE_URL" -o "./pyodide-${PYODIDE_VERSION}.tar.bz2"; then
    echo "Error: Failed to download pyodide" >&2
    exit 1
fi

# Verify the pyodide tarball exists
if [ ! -f "./pyodide-${PYODIDE_VERSION}.tar.bz2" ]; then
    echo "Error: Download file not found" >&2
    exit 1
fi

# Verify pyodide tarball checksum
echo "Verifying checksum..."
computed_checksum=$(sha256sum "./pyodide-${PYODIDE_VERSION}.tar.bz2" | cut -d' ' -f1)
if [ "$computed_checksum" != "$EXPECTED_TARBALL_CHECKSUM" ]; then
    echo "Error: Checksum mismatch for pyodide-${PYODIDE_VERSION}.tar.bz2" >&2
    echo "Expected: $EXPECTED_TARBALL_CHECKSUM" >&2
    echo "Got:      $computed_checksum" >&2
    rm -f "./pyodide-${PYODIDE_VERSION}.tar.bz2"
    exit 1
fi
echo "✓ Tarball checksum verified"

# Unpack pyodide tarball to the target directory
echo "Unpacking to $TARGET_DIR folder..."
if ! tar -xjf "./pyodide-${PYODIDE_VERSION}.tar.bz2" -C "$TARGET_DIR"; then
    echo "Error: Failed to extract archive" >&2
    rm -f "./pyodide-${PYODIDE_VERSION}.tar.bz2"
    exit 1
fi

# Cleanup pyodide tarball
echo "Cleaning up downloaded file..."
rm -f "./pyodide-${PYODIDE_VERSION}.tar.bz2"

# Check created pyodide directory checksum
if [ -d "$TARGET_DIR/pyodide" ]; then
    computed_checksum=$(calculate_pyodide_directory_checksum)
    if [ "$computed_checksum" != "$EXPECTED_DIRECTORY_CHECKSUM" ]; then
        echo "Checksum mismatch for pyodide directory" >&2
        echo "Expected: $EXPECTED_DIRECTORY_CHECKSUM" >&2
        echo "Got:      $computed_checksum" >&2
        exit 1
    fi
    echo "✓ Directory checksum verified"
fi

echo "Successfully completed!"
