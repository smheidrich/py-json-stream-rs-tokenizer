#!/bin/bash

# try to restore from cache

# note that while the version information in $ver is more specific than we need
# here (really we just need to know which container image we're in), it's fine
# because the unimportant information remains the same for the same image
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
  | tee "$PROJ_DIR/outer-ver" \
)
rm -rf "$CARGO_HOME"
rm -rf "$RUSTUP_HOME"
mv "$HOST_HOME_DIR/cargo-home-dirs/$ver/.cargo" \
  "$HOST_HOME_DIR/cargo-home-dirs/$ver/.rustup" \
  "$HOST_HOME_DIR/" || true

# check if cargo avail and download if not
if ! cargo -V; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
else
  echo "Rust toolchain already installed/restored, not downloading again"
fi
