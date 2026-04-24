#!/bin/sh
set -eu

REPO="avalonreset/hyperyap"

ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo "Error: HyperYap Linux packages currently support x86_64 only. Detected: $ARCH" >&2
    exit 1
fi

if ! command -v dpkg >/dev/null 2>&1; then
    echo "Error: dpkg not found. This installer supports Debian/Ubuntu only." >&2
    echo "For other distributions, download the AppImage from:" >&2
    echo "  https://github.com/$REPO/releases/latest" >&2
    exit 1
fi

echo "Fetching latest HyperYap version..."
LATEST_URL=$(curl -fsSo /dev/null -w '%{redirect_url}' \
    "https://github.com/$REPO/releases/latest")
VERSION=$(echo "$LATEST_URL" | sed 's|.*/||')

if [ -z "$VERSION" ]; then
    echo "Error: Could not determine latest version." >&2
    exit 1
fi

echo "Latest version: $VERSION"

DEB_URL="https://github.com/$REPO/releases/download/${VERSION}/HyperYap_amd64.deb"
TMP_DEB="/tmp/hyperyap_${VERSION}_amd64.deb"

echo "Downloading HyperYap $VERSION..."
curl -fSL -o "$TMP_DEB" "$DEB_URL"

echo "Installing HyperYap (requires sudo)..."
sudo dpkg -i "$TMP_DEB" || sudo apt-get install -f -y

rm -f "$TMP_DEB"

echo ""
echo "HyperYap $VERSION installed successfully."
echo "Launch it from your application menu or run: hyperyap"
