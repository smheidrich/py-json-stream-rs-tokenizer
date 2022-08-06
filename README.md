# json-stream-rs-tokenizer

[![CI build badge](https://github.com/smheidrich/py-json-stream-rs-tokenizer/actions/workflows/build.yml/badge.svg)](https://github.com/smheidrich/py-json-stream-rs-tokenizer/actions/workflows/build.yml)
[![CI test badge](https://github.com/smheidrich/py-json-stream-rs-tokenizer/actions/workflows/test.yml/badge.svg)](https://github.com/smheidrich/py-json-stream-rs-tokenizer/actions/workflows/test.yml)
[![PyPI package and version badge](https://img.shields.io/pypi/v/json-stream-rs-tokenizer)](https://pypi.org/project/json-stream-rs-tokenizer/)
[![Supported Python versions badge](https://img.shields.io/pypi/pyversions/json-stream-rs-tokenizer)](https://pypi.org/project/json-stream-rs-tokenizer/)

A faster tokenizer for the [json-stream](https://github.com/daggaz/json-stream)
Python library.

It's actually just `json-stream`'s own tokenizer (itself adapted from the
[NAYA](https://github.com/danielyule/naya) project) ported to Rust almost
verbatim and made available as a Python module using
[PyO3](https://github.com/PyO3/pyo3).

On my machine, it **speeds up parsing by a factor of 4–10**, depending on the
nature of the data.

## Installation

```bash
pip install json-stream-rs-tokenizer
```

**Note** that in editable/develop installs, it will sometimes (?) compile the
Rust library in debug mode, which makes it run *slower* than the pure-Python
tokenizer. When in doubt, run installation commands with `--verbose` to see the
Rust compilation commands and verify that they used `--release`.

## Usage

To use this package's `RustTokenizer`, simply pass it as the `tokenizer`
argument to `json-stream`'s `load` or `visit`:

```python
from io import StringIO
from json_stream import load
from json_stream_rs_tokenizer import RustTokenizer

json_buf = StringIO('{ "a": [1,2,3,4], "b": [5,6,7] }')

# uses the Rust tokenizer to load JSON:
d = load(json_buf, tokenizer=RustTokenizer)

for k, l in d.items():
  print(f"{k}: {' '.join(str(n) for n in l)}")
```

As a perhaps slightly more convenient alternative, the package also provides
wrappers around json_stream's `load` and `visit` functions that do it for you:

```python
from json_stream_rs_tokenizer import load

d = load(StringIO('{ "a": [1,2,3,4], "b": [5,6,7] }'))

# ...
```

## Benchmarks

The package comes with a script for rudimentary benchmarks on randomly
generated JSON data. To run it, you'll need to install the optional `benchmark`
dependencies and a version of `json-stream` with
[this patch](https://github.com/daggaz/json-stream/pull/17) applied:

```bash
pip install json_stream_rs_tokenizer[benchmark]
pip install --ignore-installed \
  git+https://github.com/smheidrich/json-stream.git@util-to-convert-to-py-std-types
```

You can then run the benchmark as follows:

```bash
python -m json_stream_rs_tokenizer.benchmark
```

Run it with `--help` to see more information.

## License

MIT license. Refer to the
[LICENSE](https://github.com/smheidrich/py-json-stream-rs-tokenizer/blob/main/LICENSE)
file for details.
