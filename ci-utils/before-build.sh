#!/bin/bash
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
rm -rf "$PROJ_DIR/target"
mv "$HOST_HOME_DIR/cargo-target-dirs/$ver/target" "$PROJ_DIR/" \
|| echo "Could not restore Cargo target dir from cache"
