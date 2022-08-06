from .json_stream_rs_tokenizer import RustTokenizer


def load(fp, persistent=False):
    """
    Run json-stream's `load` but using the Rust tokenizer.
    """
    import json_stream

    return json_stream.load(fp, persistent, tokenizer=RustTokenizer)


def visit(fp, visitor):
    """
    Run json-stream's `visit` but using the Rust tokenizer.
    """
    import json_stream

    return json_stream.visit(fp, visitor, tokenizer=RustTokenizer)

__all__ = ["RustTokenizer", "load", "visit"]
