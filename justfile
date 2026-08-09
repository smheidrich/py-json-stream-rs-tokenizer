licenses:
  # Requires [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) to
  # be installed:
  cargo about generate about.hbs > LICENSES-RUST-DEPS.html
  # From:
  # https://github.com/rust-lang/rust/issues/67014#issuecomment-3066977642
  cp \
    $(rustc --print sysroot)/share/doc/rust/COPYRIGHT-library.html \
    LICENSES-RUST.html

# Note: We put one SBOM file for all target archs into the repo instead of
# generating it separately for each target arch during CI, because the latter
# requires the `cargo-cyclonedx` binaries' glibc version and the one used in
# the `manylinux` container to match, which they didn't the last time I tried.
sbom:
  # Requires
  # [`cargo-cyclonedx`](https://github.com/CycloneDX/cyclonedx-rust-cargo)
  # to be installed:
  cargo cyclonedx --target all
