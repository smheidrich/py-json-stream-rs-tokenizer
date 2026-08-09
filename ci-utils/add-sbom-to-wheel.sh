#!/bin/bash
# Add software bills of materials (SBOM) to wheel.
# See https://peps.python.org/pep-0770/
set -euo pipefail
shopt -s nullglob

wheel_path=$1

cargo cyclonedx
sbom_filename=(*.cdx.xml)

dist_info_dirname="$(
  unzip -l "$wheel_path" \
  | awk '{print $4}' \
  | grep dist-info \
  | head -n 1 \
  | sed 's%/.*%%'
)"
sboms_dir_path="$dist_info_dirname/sboms"
mkdir -p "$sboms_dir_path"
mv "$sbom_filename" "$sboms_dir_path"

zip "$wheel_path" "$sboms_dir_path/$sbom_filename"
