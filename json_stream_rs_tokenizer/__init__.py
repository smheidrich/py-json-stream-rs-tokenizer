__all__ = [
    "load",
    "visit",
    "rust_tokenizer_or_raise",
    "ExtensionException",
    "ExtensionUnavailable",
    "RequestedFeatureUnavailable",
]


class TokenType:
    Operator = 0
    String_ = 1
    Number = 2
    Boolean = 3
    Null = 4


try:
    from .json_stream_rs_tokenizer import (
        RustTokenizer as _RustTokenizer,
        supports_bigint as _supports_bigint,
    )

    # included only for backwards-compatibility - to the outside world, bigint
    # is now always supported via fallback to conversion in Python
    def supports_bigint():
        return True

    if _supports_bigint():
        RustTokenizer = _RustTokenizer
    else:

        def RustTokenizer(f):
            """
            Rust tokenizer (fallback wrapper for integer conversion)
            """
            # x = (token_type, value) but {un&re}packing worsens performance
            for x in _RustTokenizer(f):
                if x[0] == TokenType.Number and isinstance(x[1], str):
                    # fallback required for large integers
                    yield (x[0], int(x[1]))
                else:
                    yield x

    __all__.extend(["RustTokenizer", "supports_bigint"])
except ImportError:
    pass


class ExtensionException(Exception):
    pass


class ExtensionUnavailable(ExtensionException):
    pass


class RequestedFeatureUnavailable(ExtensionException):
    pass


def rust_tokenizer_or_raise(requires_bigint=True):
    """
    Args:
        requires_bigint: Deprecated, has no effect as arbitrary-size
        integers are now always supported via fallback to conversion in Python.

    Raises:
        ExtensionUnavailable: If the Rust extension is not available.
    """
    try:
        return RustTokenizer
    except NameError as e:
        raise ExtensionUnavailable(
            "Rust tokenizer unavailable, most likely because no prebuilt "
            "wheel was available for your platform and building from source "
            "failed."
        ) from e


def load(fp, persistent=False):
    """
    Run json-stream's `load` but using the Rust tokenizer.
    """
    import json_stream

    return json_stream.load(fp, persistent, tokenizer=rust_tokenizer_or_raise())


def visit(fp, visitor):
    """
    Run json-stream's `visit` but using the Rust tokenizer.
    """
    import json_stream

    return json_stream.visit(fp, visitor, tokenizer=rust_tokenizer_or_raise())
