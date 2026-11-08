#!/bin/sh
set -eu

REPO="USER/REPO"
BIN="leuko"
SHIMS="npm bun yarn pnpm"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

os=$(uname -s)
arch=$(uname -m)

case "$os" in
  Linux)  os_part="unknown-linux-gnu" ;;
  Darwin) os_part="apple-darwin" ;;
  *) echo "Unsupported OS: $os" >&2; exit 1 ;;
esac

case "$arch" in
  x86_64|amd64)  arch_part="x86_64" ;;
  arm64|aarch64) arch_part="aarch64" ;;
  *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
esac

target="${arch_part}-${os_part}"
base="https://github.com/${REPO}/releases/latest/download"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# --- Download and verify ---
echo "Downloading ${BIN} for ${target}..."
curl -fsSL "${base}/${BIN}-${target}.tar.gz"        -o "$tmp/${BIN}.tar.gz"
curl -fsSL "${base}/${BIN}-${target}.tar.gz.sha256" -o "$tmp/${BIN}.sha256"

expected=$(cut -d' ' -f1 "$tmp/${BIN}.sha256")
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmp/${BIN}.tar.gz" | cut -d' ' -f1)
else
  actual=$(shasum -a 256 "$tmp/${BIN}.tar.gz" | cut -d' ' -f1)
fi
[ "$expected" = "$actual" ] || { echo "Checksum mismatch!" >&2; exit 1; }

# --- Install binary ---
tar xzf "$tmp/${BIN}.tar.gz" -C "$tmp"
mkdir -p "$INSTALL_DIR"
install -m 755 "$tmp/${BIN}" "$INSTALL_DIR/${BIN}"
echo "Installed $INSTALL_DIR/${BIN}"

# --- Create shims (relative symlinks, never clobber foreign files) ---
for name in $SHIMS; do
  link="$INSTALL_DIR/$name"
  if [ -L "$link" ]; then
    case "$(readlink "$link")" in
      "$BIN"|"$INSTALL_DIR/$BIN") continue ;;   # already ours
    esac
    echo "Skipping $link (existing symlink points elsewhere)" >&2
  elif [ -e "$link" ]; then
    echo "Skipping $link (a real file already exists)" >&2
  else
    ln -s "$BIN" "$link"
    echo "Linked $name -> $BIN"
  fi
done

# --- Put INSTALL_DIR first in PATH (persisted in the shell rc file) ---
marker="# added by leuko installer"
line="export PATH=\"$INSTALL_DIR:\$PATH\""
case "${SHELL:-}" in
  */zsh)  rc="$HOME/.zshrc" ;;
  */bash) if [ "$os" = Darwin ]; then rc="$HOME/.bash_profile"; else rc="$HOME/.bashrc"; fi ;;
  */fish) rc="$HOME/.config/fish/config.fish"
          line="fish_add_path --move --prepend \"$INSTALL_DIR\"" ;;
  *)      rc="$HOME/.profile" ;;
esac

mkdir -p "$(dirname "$rc")"
if ! grep -qF "$marker" "$rc" 2>/dev/null; then
  printf '\n%s\n%s\n' "$marker" "$line" >> "$rc"
  echo "Added $INSTALL_DIR to PATH in $rc"
fi

echo "Done. Restart your terminal (or run: source $rc) to start using the shims."
