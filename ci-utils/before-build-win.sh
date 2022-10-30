#!/bin/bash

# try to restore Rust home dirs from cache
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([platform()]))' \
  | tee "$HOST_HOME_DIR/outer-ver" \
)
rm -rf "$CARGO_HOME"
rm -rf "$RUSTUP_HOME"
mv "$HOST_HOME_DIR/cargo-home-dirs/$ver/.cargo" \
  "$HOST_HOME_DIR/cargo-home-dirs/$ver/.rustup" \
  "$HOST_HOME_DIR/" || true

# check if cargo avail and download if not
if ! cargo -V; then
  curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
else
  echo "Rust toolchain already installed/restored, not downloading again"
fi
rustup target add i686-pc-windows-msvc

# try to restore Rust target dir from cache
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
rm -rf "$HOST_HOME_DIR/target"
mv "$HOST_HOME_DIR/cargo-target-dirs/$ver/target" "$HOST_HOME_DIR/" \
|| echo "Could not restore Cargo target dir from cache"
