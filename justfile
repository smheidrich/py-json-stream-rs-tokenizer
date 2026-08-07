licenses:
  # Requires [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) to
  # be installed:
  cargo about generate about.hbs > LICENSES-RUST-DEPS.html
  # From:
  # https://github.com/rust-lang/rust/issues/67014#issuecomment-3066977642
  cp \
    $(rustc --print sysroot)/share/doc/rust/COPYRIGHT-library.html \
    LICENSES-RUST.html
