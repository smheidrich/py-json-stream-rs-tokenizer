#!/bin/bash

# save cargo target dir cache
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
mkdir -p "$HOST_HOME_DIR/cargo-target-dirs/$ver"
rm -rf "$HOST_HOME_DIR/cargo-target-dirs/$ver/target"
mv "$CARGO_TARGET_DIR" "$HOST_HOME_DIR/cargo-target-dirs/$ver/"

# save cargo home cache (will happen more than once on linux but that's fine)
ver=$(cat "$HOST_HOME_DIR/outer-ver")
[ -n "$ver" ] || { echo "error loading outer version"; exit 1; }
#rm -rf "$HOST_HOME_DIR/cargo-home-dirs/$ver"
mkdir -p "$HOST_HOME_DIR/cargo-home-dirs/$ver"
rsync -viau "$CARGO_HOME_UNIX" "$RUSTUP_HOME_UNIX" "$HOST_HOME_DIR_UNIX/cargo-home-dirs/$ver/"
