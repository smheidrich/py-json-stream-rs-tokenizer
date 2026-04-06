# Changelog

## 0.5.1

- **New features:**
  - Added support for parsing `NaN` and `Infinity`/`-Infinity` as
    `float("nan")` and `float("inf")`/`float("-inf")`, respectively (same as
    Python's own `json` module).
- **Compatibility changes:**
  - Dropped 32-bit Windows support for now (might be reintroduced later, see
    [#153](https://github.com/smheidrich/py-json-stream-rs-tokenizer/issues/153))

## 0.5.0

- **Potentially breaking changes:**
  - Changed the default of `correct_cursor` from `True` to `False`.

    Previously, `correct_cursor` defaulting to `True` meant that the tokenizer
    would take care of always leaving the stream cursor of *unseekable* streams
    (and only those!) at a position matching the extent to which it has parsed
    that stream into tokens, and no further. This came at the cost of severely
    degraded performance, because it meant reading the stream byte-by-byte
    instead of the much more performant option of buffering ahead in large
    chunks.

    For *seekable* streams, nothing changes, because for those,
    `correct_cursor=True` always used buffering (and still does) and doesn't
    actually leave the cursor in the "correct" position (matching the
    tokenization extent) by default: The only effect of `correct_cursor=True`
    for seekable streams was and is to "remember" the position up to which the
    stream has been tokenized, and only a call of `RustTokenizer.park_cursor()`
    causes the cursor to be "reset" back to this position.

    If you've been relying on the previous behavior for unseekable streams,
    you can recover it by explicitly instantiating `RustTokenizer` with
    `correct_cursor=True`. As `json-stream` is normally what instantiates the
    tokenizer (the `tokenizer` parameter of its `load` function actually takes
    a tokenizer "factory"), you can cause _it_ to instantiate it with
    `correct_cursor` set using `functools.partial`:

    ```python
    from functools import partial
    from json_stream import load

    load(..., tokenizer=partial(RustTokenizer, correct_cursor=True))
    ```

## 0.4.32

- Added type stub files (written by hand so may contain errors)

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
