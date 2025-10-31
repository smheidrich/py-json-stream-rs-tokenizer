# Changelog

## 0.4.31

- **Interface changes:**
  - Some exception messages related to BigInt support in PyPy may have changed.
    - This should not affect the vast majority of users and whether exact
      exception messages are part of the interface is up for debate, so it's
      not considered a breaking change.
- **Packaging improvements:**
  - The artifacts published to PyPI now come with
    [attestations](https://docs.pypi.org/attestations/).
- **Fixed bugs:**
  - Using json-stream-rs-tokenizer with Python 3.14 would segfault. This should
    no longer happen.
- **Python/library compatibility changes:**
  - Added wheels for Python 3.14.
  - Dropped support for Python 3.7 (all variants) and PyPy versions other than
    3.11.
    - This is not considered a breaking change, and hence doesn't necessitate a
      (pseudo)-major version release, because it should be "invisible" to
      people using now-unsupported Python versions: Package installers will
      simply not resolve to json-strema-rs-tokenizer 0.4.31.
  - To avoid issues like the segfaults when used with Python 3.14 in the
    future, json-stream-rs-tokenizer now specifies an upper bound for the
    supported Python versions (currently `<3.15`) which will only be raised
    after testing that a new Python version works fine.

## 0.4.30 and below

*I only started writing a changelog starting at 0.4.31. For older versions,
please have a look at the commit history and the `v`-prefixed tags attached to
commits.*
