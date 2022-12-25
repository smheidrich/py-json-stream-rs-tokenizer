"""
Regression test for overconsumption of stream contents past the end of a doc:
https://github.com/smheidrich/py-json-stream-rs-tokenizer/issues/47
"""
from io import BytesIO, StringIO

import pytest

from json_stream_rs_tokenizer import RustTokenizer, load


@pytest.fixture(params=["str", "bytes", "str-unseekable", "bytes-unseekable"])
def to_bytes_or_str_buf(request):
    if request.param == "str":
        return lambda s: StringIO(s)
    elif request.param == "str-unseekable":

        def make_unseekable_stringio(s: str):
            sio = StringIO(s)
            sio.seekable = lambda: False
            return sio

        return make_unseekable_stringio
    elif request.param == "bytes":
        return lambda s: BytesIO(s.encode("utf-8"))
    elif request.param == "bytes-unseekable":

        def make_unseekable_bytesio(s: str):
            bio = BytesIO(s.encode("utf-8"))
            bio.seekable = lambda: False
            return bio

        return make_unseekable_bytesio
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
def test_overconsumes_when_correct_cursor_disabled(s, to_bytes_or_str_buf):
    """
    Test that overconsumption occurs when cursor correctness is not requested.

    In that case, we do readahead buffering regardless of seekability to
    maximize performance.
    """
    buf = to_bytes_or_str_buf(s)
    tokenizer = RustTokenizer(buf, False)
    for kind, val in tokenizer:
        if val == "}":
            break
    tokenizer.park_cursor()
    assert len(buf.read()) == 0  # EOF => readahead buf read everything
