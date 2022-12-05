"""
Regression test for overconsumption of stream contents past the end of a doc:
https://github.com/smheidrich/py-json-stream-rs-tokenizer/issues/47
"""
from io import StringIO

import pytest

from json_stream_rs_tokenizer import load


@pytest.mark.parametrize(
    "s,expected_cursor_pos",
    [
        ('{ "a": 1 } { "b": 2 }', 10),
        ('{"a": 1} { "b": 2 }', 8),
        ('{"a":1} { "b": 2 }', 7),
        ('{ "a":1, "b": 2, "c": 3, "d": 4, "xyz": 99999 } { "b": 2 }', 47),
    ],
)
def test_overconsumption_multiple_documents(s, expected_cursor_pos):
    buf = StringIO(s)
    list(load(buf))
    assert buf.tell() == expected_cursor_pos
