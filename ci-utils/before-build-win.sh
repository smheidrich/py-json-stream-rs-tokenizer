#!/bin/bash

# try to restore Rust home dirs from cache
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([platform()]))' \
  | tee "$HOST_HOME_DIR_UNIX/outer-ver" \
)
rm -rf "$CARGO_HOME_UNIX"
rm -rf "$RUSTUP_HOME_UNIX"
mv "$HOST_HOME_DIR_UNIX/cargo-home-dirs/$ver/.cargo" \
  "$HOST_HOME_DIR_UNIX/cargo-home-dirs/$ver/.rustup" \
  "$HOST_HOME_DIR_UNIX/" || true

# check if cargo avail and download if not
if ! { cargo -V && rustup -V; }; then
  curl --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
else
  echo "Rust toolchain already installed/restored, not downloading again"
fi
rustup target add i686-pc-windows-msvc

# install SBOM utils:
# check if cargo binstall avail and download if not
if ! cargo binstall -V; then
  curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
else
  echo "cargo binstall already installed/restored, not downloading again"
fi
# install SBOM utils
cargo binstall -y cargo-cyclonedx
cargo cyclonedx --version
# /install SBOM utils

# try to restore Rust target dir from cache
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
rm -rf "$HOST_HOME_DIR_UNIX/target"
mv "$HOST_HOME_DIR_UNIX/cargo-target-dirs/$ver/target" "$HOST_HOME_DIR_UNIX/" \
|| echo "Could not restore Cargo target dir from cache"
