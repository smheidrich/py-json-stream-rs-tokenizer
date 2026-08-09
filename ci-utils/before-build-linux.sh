#!/bin/bash

# install libatomic (required for newer rustup versions to work on older
# CentOS-based manylinux images - may have to be removed again once
# cibuildwheel updates the default manylinux image used from 2014 to sth.
# newer)
yum install -y libatomic

# copy file mtimes from host
shopt -s dotglob
cp -r -a --attributes-only "$HOST_PROJ_DIR"/* "$PROJ_DIR"/

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
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
else
  echo "Rust toolchain already installed/restored, not downloading again"
fi

# install SBOM utils:
# check if cargo binstall avail and download if not
if ! cargo binstall -V; then
  curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
else
  echo "cargo binstall already installed/restored, not downloading again"
fi
# install SBOM utils (we use musl because GNU has some glibc version issues...)
cargo binstall -y --targets x86_64-unknown-linux-musl cargo-cyclonedx
cargo cyclonedx --version
# /install SBOM utils

# try to restore Rust target dir from cache
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
rm -rf "$HOST_HOME_DIR/target"
mv "$HOST_HOME_DIR/cargo-target-dirs/$ver/target" "$HOST_HOME_DIR/" \
|| echo "Could not restore Cargo target dir from cache"
