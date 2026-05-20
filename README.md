# Leuko

This binary is meant to capture commands installing packages to package managers and audit them before allowing you to install them.
A prevention against supply chain attacks.

## Installation

cargo build --release

### Install the binary
cp target/release/leuko ~/.local/bin/leuko

### Symlink npm → leuko
ln -sf ~/.local/bin/leuko ~/.local/bin/npm

### Ensure this dir is first in PATH
export PATH="$HOME/.local/bin:$PATH"

## Developer Notes

Build for windows

`rustup target add x86_64-pc-windows-gnu`

`sudo apt install mingw-w64`

`cargo build --target x86_64-unknown-linux-gnu`
