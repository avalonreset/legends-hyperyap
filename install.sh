#!/bin/sh
set -eu

REPO="avalonreset/legends-hyperyap"

ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo "Error: legends-hyperyap Linux packages currently support x86_64 only. Detected: $ARCH" >&2
    exit 1
fi

if ! command -v dpkg >/dev/null 2>&1; then
    echo "Error: dpkg not found. This installer supports Debian/Ubuntu only." >&2
    echo "For other distributions, download the AppImage from:" >&2
    echo "  https://github.com/$REPO/releases/latest" >&2
    exit 1
fi

echo "Fetching latest legends-hyperyap version..."
LATEST_URL=$(curl -fsSo /dev/null -w '%{redirect_url}' \
    "https://github.com/$REPO/releases/latest")
VERSION=$(echo "$LATEST_URL" | sed 's|.*/||')

case "$VERSION" in
    v[0-9]*.[0-9]*.[0-9]*)
        ;;
    *)
        echo "Error: Invalid or unexpected version tag: $VERSION" >&2
        exit 1
        ;;
esac

echo "Latest version: $VERSION"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

DEB_NAME="HyperYap_amd64.deb"
DEB_URL="https://github.com/$REPO/releases/download/${VERSION}/${DEB_NAME}"
SUMS_URL="https://github.com/$REPO/releases/download/${VERSION}/SHA256SUMS.txt"

echo "Downloading legends-hyperyap $VERSION and SHA256SUMS.txt..."
curl -fSL -o "${TMP_DIR}/${DEB_NAME}" "$DEB_URL"
curl -fSL -o "${TMP_DIR}/SHA256SUMS.txt" "$SUMS_URL"

echo "Verifying SHA-256 digest..."
(
    cd "$TMP_DIR"
    grep " ${DEB_NAME}\$" SHA256SUMS.txt | sha256sum -c -
)

echo "Installing legends-hyperyap (requires sudo)..."
sudo dpkg -i "${TMP_DIR}/${DEB_NAME}" || sudo apt-get install -f -y

echo ""
echo "legends-hyperyap $VERSION installed successfully."
echo "Launch it from your application menu or run: hyperyap"
