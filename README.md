> **NOTE:** `json-stream-rs-tokenizer` is now automatically used by
> `json-stream`, so unless you find a bug, you can ignore this package's
> existence!

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

### Implicit

Starting at its 2.0 release, **`json-stream` depends on and uses
`json-stream-rs-tokenizer` by default**, so you don't need to install it
explicitly anymore.

### Explicit

If you use an older `json-stream` version (which you have no reason to do) or
need to install `json-stream-rs-tokenizer` explicitly for another reason, you
can do:

```bash
pip install json-stream-rs-tokenizer
```

The library will be installed as a prebuilt wheel if one is available for your
platform. Otherwise, pip will try to build it from the source distribution,
which requires a Rust toolchain to be installed and available to succeed.

Note that if the build from source fails, the package installation will be
considered successfully completed anyway, but `RustTokenizer` (see below) won't
be available for import. This is so that packages (specifically, `json-stream`)
can depend on the library but fall back to their own implementation if neither
a prebuilt wheel is available nor the build succeeds.

You can increase the installation command's verbosity with `-v` (repeated for
even more information, e.g. `-vv`) to see error messages when the build from
source fails.

**Note** that if the Rust library is compiled in debug mode, it will run
*slower* than the pure-Python tokenizer. The setuptools configuration should
make sure this doesn't happen even when installing in development mode, but
when in doubt, run installation commands with `-v` to see the Rust compilation
commands and verify that they used `--release`.

## Usage

### Implicit

As described above, `json-stream-rs-tokenizer` is now used by `json-stream` by
default, so you don't have to do anything special to use it. `json-stream` will
fall back to its pure-Python tokenizer when `json-stream-rs-tokenizer` was not
successfully installed, however.

### Explicit

For older versions of `json-stream`, or if you want to *ensure* the Rust
tokenizer is used no matter what, simply pass this package's `RustTokenizer` as
the `tokenizer` argument to `json-stream`'s `load` or `visit`:

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

Note that the import of `RustTokenizer` will fail if the Rust extension is not
available (i.e., when no prebuilt wheels were available and the installation
from the source distribution failed).

## Limitations

- For PyPy, the speedup is only 1.0-1.5x (much lower than that for CPython).
  This has yet to be
  [investigated](https://github.com/smheidrich/py-json-stream-rs-tokenizer/issues/33).
- In builds that don't support PyO3's
  [`num-bigint` extension](https://pyo3.rs/main/doc/pyo3/num_bigint/)
  (currently only PyPy builds and manual ones against Python's limited C API
  (`Py_LIMITED_API`)), conversion of large integers is performed in Python
  rather than in Rust, at a very small runtime cost.

## Benchmarks

The package comes with a script for rudimentary benchmarks on randomly
generated JSON data. To run it, you'll need to install the optional `benchmark`
dependencies and a version of `json-stream` with
[this patch](https://github.com/daggaz/json-stream/pull/17) applied:

```bash
pip install 'json_stream_rs_tokenizer[benchmark]'
pip install --ignore-installed \
  'git+https://github.com/smheidrich/json-stream.git@util-to-convert-to-py-std-types'
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
