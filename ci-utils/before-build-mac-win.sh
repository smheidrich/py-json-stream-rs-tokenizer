#!/bin/bash
# move target dir
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
rm -rf "$HOST_HOME_DIR/target"
mv "$HOST_HOME_DIR/cargo-target-dirs/$ver/target" "$HOST_HOME_DIR/" \
|| echo "Could not restore Cargo target dir from cache"
