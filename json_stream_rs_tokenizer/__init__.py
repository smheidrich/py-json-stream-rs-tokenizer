__all__ = [
    "load",
    "visit",
    "rust_tokenizer_or_raise",
    "ExtensionException",
    "ExtensionUnavailable",
    "RequestedFeatureUnavailable",
    "JsonStringReader",
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
        JsonStringReader,
    )

    # included only for backwards-compatibility - to the outside world, bigint
    # is now always supported via fallback to conversion in Python
    def supports_bigint():
        return True

    if _supports_bigint():
        RustTokenizer = _RustTokenizer
    else:

        class RustTokenizer:
            """
            Rust tokenizer (fallback wrapper for integer conversion)
            """

            def __init__(self, *args, **kwargs):
                self.inner = _RustTokenizer(*args, **kwargs)

            def __iter__(self):
                return self

            def __next__(self):
                # x = (token_type, value) but {un&re}packing worsens perf
                x = self.inner.__next__()
                if x[0] == TokenType.Number and isinstance(x[1], str):
                    # fallback required for large integers
                    return (x[0], int(x[1]))
                else:
                    return x

            @property
            def remainder(self):
                return self.inner.remainder

            def park_cursor(self):
                self.inner.park_cursor()

    __all__.extend(["RustTokenizer", "supports_bigint"])
except ImportError:
    pass


class ExtensionException(Exception):
    pass


class ExtensionUnavailable(ExtensionException):
    pass


class RequestedFeatureUnavailable(ExtensionException):
    pass


def rust_tokenizer_or_raise(requires_bigint=True, **kwargs):
    """
    Args:
        requires_bigint: Deprecated, has no effect as arbitrary-size
            integers are now always supported via fallback to conversion in
            Python.
        kwargs: Keyword arguments *excluding the `stream` argument* with which
            you're planning to instantiate the tokenizer. Facilitates checking
            if any of them (or their specific values) aren't known or supported
            yet.

    Raises:
        ExtensionUnavailable: If the Rust extension is not available.
        RequestedFeatureUnavailable: If a requested feature is not available.
    """
    supported_kwargs = {"buffering", "strings_as_files"}
    unsupported = kwargs.keys() - supported_kwargs
    if unsupported:
        raise RequestedFeatureUnavailable(
            "some requested features are unknown in this version of "
            f"json-stream-rs-tokenizer: {unsupported}"
        )
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

    return json_stream.load(
        fp, persistent, tokenizer=rust_tokenizer_or_raise()
    )


def visit(fp, visitor):
    """
    Run json-stream's `visit` but using the Rust tokenizer.
    """
    import json_stream

    return json_stream.visit(fp, visitor, tokenizer=rust_tokenizer_or_raise())
