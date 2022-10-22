#!/bin/bash

# save cargo target dir cache
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
mkdir -p "$HOST_HOME_DIR/cargo-target-dirs/$ver"
rm -rf "$HOST_HOME_DIR/cargo-target-dirs/$ver/target"
mv "$PROJ_DIR/target" "$HOST_HOME_DIR/cargo-target-dirs/$ver/"

# save cargo home cache (will happen more than once on linux but that's fine)
ver=$(cat "$PROJ_DIR/outer-ver")
[ -n "$ver" ] || { echo "error loading outer version"; exit 1; }
rm -rf "$HOST_HOME_DIR/cargo-home-dirs/$ver"
mkdir -p "$HOST_HOME_DIR/cargo-home-dirs/$ver"
cp -r "$HOME/.cargo" "$HOST_HOME_DIR/cargo-home-dirs/$ver/"
