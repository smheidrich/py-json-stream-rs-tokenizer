__all__ = ["load", "visit", "rust_tokenizer_or_raise", "ExtensionUnavailable"]

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


class ExtensionUnavailable(Exception):
    pass


def rust_tokenizer_or_raise():
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
