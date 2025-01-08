"""
Test compatibility with json-stream's support for giving iterables to `load()`.
"""
import json_stream
import pytest


@pytest.mark.parametrize("chunk_size", [1, 2, 3, 4, 10])
def test_chunk_boundary_inside_utf8_char(chunk_size: int) -> None:
    """
    Test that chunk boundaries inside UTF-8 chars are handled correctly.

    Regression test for https://github.com/daggaz/json-stream/issues/59.
    """
    inner_str = "——"
    document_str = f'"{inner_str}"'
    document_bytes = document_str.encode("utf-8")

    iterable = (
        document_bytes[i : i + chunk_size]
        for i in range(0, len(document_bytes), chunk_size)
    )

    parsed = json_stream.load(iterable)

    assert parsed == inner_str
