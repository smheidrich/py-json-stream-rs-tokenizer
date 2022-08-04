# json-stream-rs-tokenizer

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
pip install git+https://github.com/smheidrich/py-json-stream-rs-tokenizer.git
```

**Note** that in editable installs, it will sometimes (?) compile the Rust
library in debug mode, which makes it run *slower* than the pure-Python
tokenizer. When in doubt, run installation commands with `--verbose` to see the
Rust compilation commands and verify that they used `--release`.

## Usage

Because `json-stream` currently has no mechanism to provide a custom tokenizer
(which I would prefer), this package provides its own wrappres around
json_stream's `load` and `visit` functions that monkeypatch it in before
running them:

```python
from io import StringIO
import json_stream_rs_tokenizer import load

# uses the Rust tokenizer to load JSON:
d = load(StringIO('{ "a": [1,2,3,4], "b": [5,6,7] }'))

for k, l in d.items():
  print(f"{k}: {' '.join(str(n) for n in l)}")
```

The patching is undone when the function returns.

Due to patching being a global state mutation, using `json-stream-rs-tokenizer`
in this way is generally *not thread-safe*. As an alternative, you can patch it
in manually using `json_stream_rs_tokenizer.patch()`, which should be safe if
you do it before you spawn any threads, and then just call the original (but
now patched) `json_stream.load` and `json_stream.visit` functions.

## License

MIT license. See the [LICENSE](LICENSE) file for details.
