#!/bin/bash
ver=$( \
  python3 -c \
  'from platform import *; print("-".join([python_implementation(), python_version(), platform()]))' \
);
mv "$HOST_HOME_DIR/cargo-target-dirs/$ver" "$PROJ_DIR/target" || true
