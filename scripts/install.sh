#!/usr/bin/env sh
# ctx — install script
#
#   curl -fsSL https://ctx.dev/install.sh | sh
#   CTX_VERSION=v0.2.0 curl -fsSL https://ctx.dev/install.sh | sh
#
# Installs the prebuilt ctx binary into ~/.ctx/bin (or $CTX_INSTALL_DIR),
# verifies its SHA-256 checksum, and explains PATH configuration.
# No sudo required. Will not overwrite an existing binary without warning.
set -eu

# ---- configuration ---------------------------------------------------------
REPO="${CTX_REPO:-halloffame12/CTX}"
VERSION="${CTX_VERSION:-latest}"
INSTALL_DIR="${CTX_INSTALL_DIR:-$HOME/.ctx}"
BIN_DIR="$INSTALL_DIR/bin"
BASE_URL="https://github.com/${REPO}/releases/download"

# ---- helpers ---------------------------------------------------------------
say() { printf '%s\n' "$*"; }
die() { printf 'install: %s\n' "$*" >&2; exit 1; }

# shellcheck disable=SC3057
have() { command -v "$1" >/dev/null 2>&1; }

# ---- detect os + arch ------------------------------------------------------
os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "$os" in
  linux*) os="linux" ;;
  darwin*) os="macos" ;;
  *) die "unsupported OS: $os (linux and macOS only — use install.ps1 on Windows)" ;;
esac

case "$arch" in
  x86_64|amd64) arch="x86_64" ;;
  aarch64|arm64) arch="aarch64" ;;
  *) die "unsupported architecture: $arch" ;;
esac

artifact="ctx-${os}-${arch}"
[ "$os" = "windows" ] && artifact="$artifact.exe"

# ---- resolve version -------------------------------------------------------
if [ "$VERSION" = "latest" ]; then
  if have curl; then
    VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)"
  elif have wget; then
    VERSION="$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)"
  else
    die "cannot resolve latest version — need curl or wget, or set CTX_VERSION"
  fi
  [ -n "$VERSION" ] || die "could not resolve the latest release version"
fi
VERSION="${VERSION#v}"

url="${BASE_URL}/v${VERSION}/${artifact}"
checksum_url="${BASE_URL}/v${VERSION}/checksums.txt"
target="$BIN_DIR/$artifact"

say "ctx — installing v$VERSION ($os/$arch)"
say "  from: $url"

# ---- existing binary warning ----------------------------------------------
if [ -f "$target" ]; then
  old_ver="$("$target" --version 2>/dev/null | head -1 || true)"
  if [ -n "$old_ver" ] && [ "$old_ver" = "ctx $VERSION" ]; then
    say "  ctx $VERSION already installed at $target"
    exit 0
  fi
  say "  warning: overwriting existing binary at $target ($old_ver)"
fi

# ---- download --------------------------------------------------------------
mkdir -p "$BIN_DIR"
tmp_dir="$INSTALL_DIR/.install.tmp.$$"
trap 'rm -rf "$tmp_dir"' EXIT
mkdir -p "$tmp_dir"

if have curl; then
  curl -fsSL "$url" -o "$tmp_dir/ctx"
elif have wget; then
  wget -qO "$tmp_dir/ctx" "$url"
else
  die "need curl or wget to download"
fi

# ---- checksum verification -------------------------------------------------
if have sha256sum; then
  check_cmd="sha256sum"
elif have shasum; then
  check_cmd="shasum -a 256"
else
  die "need sha256sum or shasum to verify the download"
fi

want="$( (curl -fsSL "$checksum_url" || wget -qO- "$checksum_url" 2>/dev/null) \
  | grep "  ${artifact}$" | awk '{print $1}' | head -1 )"
[ -n "$want" ] || die "could not fetch checksum for ${artifact}"
got="$( $check_cmd "$tmp_dir/ctx" | awk '{print $1}' )"
[ "$got" = "$want" ] || die "checksum mismatch for ${artifact} (got $got, want $want)"

# ---- install ---------------------------------------------------------------
[ -x "$tmp_dir/ctx" ] || chmod +x "$tmp_dir/ctx"
mv "$tmp_dir/ctx" "$target"
trap - EXIT
rm -rf "$tmp_dir"

say "  installed to $target"

# ---- PATH guidance ---------------------------------------------------------
case ":$PATH:" in
  *":$BIN_DIR:"*) : ;;
  *)
    case "$os" in
      macos)
        shell="$(basename "${SHELL:-}")"
        case "$shell" in
          zsh) rc="$HOME/.zshrc" ;;
          bash) rc="$HOME/.bash_profile" ;;
          *) rc="$HOME/.profile" ;;
        esac
        ;;
      *) rc="$HOME/.profile" ;;
    esac
    say ""
    say "  $BIN_DIR is not on your PATH. Add it with:"
    say ""
    if [ "$os" = "macos" ]; then
      say "    echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> $rc"
      say "    source $rc"
    else
      say "    echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> $rc"
      say "    export PATH=\"$BIN_DIR:\$PATH\""
    fi
    say ""
    ;;
esac

# ---- verify ----------------------------------------------------------------
ver="$("$target" --version 2>&1 | head -1)"
[ "$ver" = "ctx $VERSION" ] || die "post-install check failed: got '$ver'"
say "  ✓ $ver"
say "  next: ctx init"