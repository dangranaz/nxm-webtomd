#!/bin/sh
# nxm-webtomd installer
#
#   curl -fsSL https://raw.githubusercontent.com/dangranaz/nxm-webtomd/main/install.sh | sh
#
# Detects your OS/arch, downloads the matching nxm-webtomd binary from the
# public GitHub release, verifies its SHA-256, and installs it to ~/.local/bin.
#
# nxm-webtomd is a single self-contained binary: there is nothing else to
# download at runtime.
#
# Supported platforms: macOS arm64 (Apple Silicon), Linux x86_64.
#
# Overrides (env vars):
#   NXM_REPO      distribution repo   (default: dangranaz/nxm-webtomd)
#   NXM_VERSION   release tag or "latest" (default: latest)
#   NXM_BIN_DIR   install dir         (default: $HOME/.local/bin)
set -eu

REPO="${NXM_REPO:-dangranaz/nxm-webtomd}"
VERSION="${NXM_VERSION:-latest}"
BIN_NAME="nxm-webtomd"
BIN_DIR="${NXM_BIN_DIR:-$HOME/.local/bin}"

say() { printf '\033[1;34m==>\033[0m %s\n' "$1"; }
err() { printf '\033[1;31mERROR:\033[0m %s\n' "$1" >&2; exit 1; }

# --- detect os/arch → release asset label ---------------------------------
os="$(uname -s)"
arch="$(uname -m)"
case "$os-$arch" in
  Darwin-arm64|Darwin-aarch64) LABEL="macos-arm64" ;;
  Linux-x86_64)                LABEL="linux-x64"   ;;
  *) err "unsupported platform: $os-$arch (supported: macOS arm64, Linux x86_64)" ;;
esac
say "platform: $LABEL"

ASSET="nxm-webtomd-${LABEL}.tar.gz"

if [ "$VERSION" = "latest" ]; then
  BASE="https://github.com/$REPO/releases/latest/download"
else
  BASE="https://github.com/$REPO/releases/download/$VERSION"
fi
URL="$BASE/$ASSET"

# --- download --------------------------------------------------------------
command -v curl >/dev/null 2>&1 || err "curl is required"
command -v tar  >/dev/null 2>&1 || err "tar is required"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

say "downloading $URL"
if ! curl -fsSL "$URL" -o "$TMP/pkg.tar.gz"; then
  err "download failed — the release/asset may not exist yet ($REPO $VERSION $ASSET)"
fi

# --- verify checksum (sidecar .sha256 published next to the asset) ---------
if curl -fsSL "$URL.sha256" -o "$TMP/pkg.sha256" 2>/dev/null; then
  expected="$(awk '{print $1}' "$TMP/pkg.sha256" | tr -d '[:space:]')"
  if command -v shasum >/dev/null 2>&1; then
    got="$(shasum -a 256 "$TMP/pkg.tar.gz" | awk '{print $1}')"
  else
    got="$(sha256sum "$TMP/pkg.tar.gz" | awk '{print $1}')"
  fi
  if [ "$expected" != "$got" ]; then
    err "checksum mismatch: expected $expected, got $got"
  fi
  say "checksum verified"
else
  say "no checksum sidecar found — skipping verification"
fi

# --- unpack + install ------------------------------------------------------
say "unpacking"
tar -C "$TMP" -xzf "$TMP/pkg.tar.gz"
[ -f "$TMP/$BIN_NAME" ] || err "binary $BIN_NAME not found in archive"

mkdir -p "$BIN_DIR"
install -m 0755 "$TMP/$BIN_NAME" "$BIN_DIR/$BIN_NAME" 2>/dev/null \
  || { cp "$TMP/$BIN_NAME" "$BIN_DIR/$BIN_NAME" && chmod 0755 "$BIN_DIR/$BIN_NAME"; }
say "installed → $BIN_DIR/$BIN_NAME"

# --- PATH hint -------------------------------------------------------------
case ":$PATH:" in
  *":$BIN_DIR:"*) : ;;
  *) printf '\033[1;33mNOTE:\033[0m add %s to your PATH:\n    export PATH="%s:$PATH"\n' "$BIN_DIR" "$BIN_DIR" ;;
esac

say "done. Configure it as an MCP server (command: $BIN_NAME) in your agent."
