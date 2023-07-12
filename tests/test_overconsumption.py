"""
Regression test for overconsumption of stream contents past the end of a doc:
https://github.com/smheidrich/py-json-stream-rs-tokenizer/issues/47
"""
from io import BytesIO, StringIO

import pytest

from json_stream_rs_tokenizer import RustTokenizer, load


@pytest.fixture()
def to_bytes_or_str(to_bytes_or_str_buf):
    xio = to_bytes_or_str_buf("")
    if isinstance(xio, StringIO):
        return lambda s: s
    elif isinstance(xio, BytesIO):
        return lambda s: s.encode("utf-8")
    else:
        assert False


# this test requires a version of json-stream that supports park_cursor()
@pytest.mark.xfail(reason="json-stream chicken and egg problem")
@pytest.mark.parametrize(
    "s,expected_cursor_pos",
    [
        ('{ "a": 1 } { "b": 2 }', 10),
        ('{"a": 1} { "b": 2 }', 8),
        ('{"a":1} { "b": 2 }', 7),
        ('{ "a":1, "b": 2, "c": 3, "d": 4, "xyz": 99999 } { "b": 2 }', 47),
        ('{ "a": [1, 2, 3, 4, 5 ], "d": 4, "xyz": 99999 } { "b": 2 }', 47),
    ],
)
def test_overconsumption_load_ends_at_doc_end(
    s, expected_cursor_pos, to_bytes_or_str_buf
):
    buf = to_bytes_or_str_buf(s)
    list(load(buf))
    assert buf.tell() == expected_cursor_pos


@pytest.mark.parametrize(
    "s,expected_str_cursor_pos,expected_bytes_cursor_pos",
    [
        ('{ "a": 1 } | { "b": 2 }', 10, 10),
        ('{"a": 1} | { "b": 2 }', 8, 8),
        ('{"a":1} | { "b": 2 }', 7, 7),
        ('{ "a":1, "b": 2, "c": 3, "d": 4, "xyz": 9 } | { "b": 2 }', 43, 43),
        ('{ "æ": [1, 2, 3, 4, 5 ], "ð": 4, "xyz": 9 } | { "b": 2 }', 43, 45),
    ],
)
def test_overconsumption_park_cursor_skip_3_chars_and_continue(
    s, expected_str_cursor_pos, expected_bytes_cursor_pos, to_bytes_or_str_buf
):
    """
    Principal regression test for overconsumption.
    """
    buf = to_bytes_or_str_buf(s)
    tokenizer = RustTokenizer(buf)
    for kind, val in tokenizer:
        if val == "}":
            break
    tokenizer.park_cursor()
    if isinstance(buf, StringIO):
        assert buf.tell() == expected_str_cursor_pos
    elif isinstance(buf, BytesIO):
        assert buf.tell() == expected_bytes_cursor_pos
    else:
        assert False, "what"
    buf.read(3)  # skip ahead 3 chars
    assert "".join(str(val) for kind, val in tokenizer) == "{b:2}"


@pytest.mark.parametrize(
    "s",
    [
        ('{ "a": 1 } | { "b": 2 }'),
        ('{"a": 1} | { "b": 2 }'),
        ('{"a":1} | { "b": 2 }'),
        ('{ "a":1, "b": 2, "c": 3, "d": 4, "xyz": 9 } | { "b": 2 }'),
        ('{ "æ": [1, 2, 3, 4, 5 ], "ð": 4, "xyz": 9 } | { "b": 2 }'),
    ],
)
def test_correct_cursor_disabled(s, to_bytes_or_str_buf, to_bytes_or_str):
    """
    Test that overconsumption occurs when cursor correctness is not requested.

    In that case, we do readahead buffering regardless of seekability to
    maximize performance.

    Also tests that the remainder is correct in this case, which allows people
    to build their own workarounds for the issue in Python if they so choose.
    """
    buf = to_bytes_or_str_buf(s)
    tokenizer = RustTokenizer(buf, correct_cursor=False)
    for kind, val in tokenizer:
        if val == "}":
            break
    assert len(buf.read()) == 0  # EOF => readahead buf read everything
    assert tokenizer.remainder == to_bytes_or_str(s.split("}", maxsplit=1)[1])
