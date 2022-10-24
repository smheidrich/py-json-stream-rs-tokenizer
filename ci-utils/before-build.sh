#!/bin/bash
# copy file mtimes from host
shopt -s dotglob
cp -r -a --attributes-only "$HOST_PROJ_DIR"/* "$PROJ_DIR"/
# copy target dir
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
)
rm -rf "$HOST_HOME_DIR/target"
mv "$HOST_HOME_DIR/cargo-target-dirs/$ver/target" "$HOST_HOME_DIR/" \
|| echo "Could not restore Cargo target dir from cache"
echo "cargo ver:"
cargo -V
