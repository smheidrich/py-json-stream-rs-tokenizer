from unittest.mock import patch as mock_patch
from contextlib import contextmanager


@contextmanager
def patched():
    """
    Context manager that patches json-stream to use the Rust tokenizer.

    The patch will be undone upon leaving the context.
    """
    with patched_loader(), patched_visitor():
        yield


@contextmanager
def patched_loader():
    """
    Like `patched`, but only patches `load` to use the Rust tokenizer.
    """
    from .json_stream_rs_tokenizer import RustTokenizer

    with mock_patch("json_stream.loader.tokenize", new=RustTokenizer):
        yield


@contextmanager
def patched_visitor():
    """
    Like `patched`, but only patches `visit` to use the Rust tokenizer.
    """
    from .json_stream_rs_tokenizer import RustTokenizer

    with mock_patch("json_stream.visitor.tokenize", new=RustTokenizer):
        yield


def patch():
    """
    Non-context-manager version of `patched`.

    Actually just calls `patched()` without `with` so it never exits (so if you
    want to undo the patch explicitly, just call the returned object's
    `__exit__`), but the name makes more sense for a non-context-manager
    operation.
    """
    return patched()


def load(fp, persistent=False):
    """
    Run json-stream's `load` but using the Rust tokenizer.
    """
    import json_stream

    with patched_loader():
        return json_stream.load(fp, persistent)


def visit(fp, visitor):
    """
    Run json-stream's `visit` but using the Rust tokenizer.
    """
    import json_stream

    with patched_loader():
        return json_stream.visit(fp, visitor)
