__all__ = [
    "load",
    "visit",
    "rust_tokenizer_or_raise",
    "ExtensionException",
    "ExtensionUnavailable",
    "RequestedFeatureUnavailable",
]

try:
    from .json_stream_rs_tokenizer import RustTokenizer

    __all__.append("RustTokenizer")
except ImportError:
    pass

try:
    from .json_stream_rs_tokenizer import supports_bigint

    __all__.append("supports_bigint")
except ImportError:
    pass


class ExtensionException(Exception):
    pass


class ExtensionUnavailable(ExtensionException):
    pass


class RequestedFeatureUnavailable(ExtensionException):
    pass


def rust_tokenizer_or_raise(requires_bigint=True):
    try:
        tokenizer = RustTokenizer
        if requires_bigint and not supports_bigint():
            raise RequestedFeatureUnavailable(
                "Rust tokenizer lacks requested support for arbitrary-size "
                "integers on your platform, most likely because you're using "
                "PyPy or the extension was built with Py_LIMITED_API."
            )
    except NameError as e:
        raise ExtensionUnavailable(
            "Rust tokenizer unavailable, most likely because no prebuilt "
            "wheel was available for your platform and building from source "
            "failed."
        ) from e
    return tokenizer


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
