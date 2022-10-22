#!/bin/bash
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
mkdir -p "$HOST_HOME_DIR/cargo-target-dirs/$ver"
rm -rf "$HOST_HOME_DIR/cargo-target-dirs/$ver" || true
mv "$PROJ_DIR"/target "$HOST_HOME_DIR/cargo-target-dirs/$ver"
