## Installation

cargo build --release

### Install the binary

cp target/release/leuko ~/.local/bin/leuko

### Symlink npm → leuko

Add as many things as you would like leuko to control.
This only effects leukos abilities and does not manipulate your personal environment.

`ln -sf ~/.local/bin/leuko ~/.local/bin/npm`

`ln -sf ~/.local/bin/leuko ~/.local/bin/bun`

### Ensure this dir is first in PATH

export PATH="$HOME/.local/bin:$PATH"

## Developer Notes

Build for windows

`rustup target add x86_64-pc-windows-gnu`

`sudo apt install mingw-w64`

`cargo build --target x86_64-unknown-linux-gnu`

#### Testing

No verbosity on tests to visualize outputs

`cargo test -q --lib TEST -- --nocapture`
