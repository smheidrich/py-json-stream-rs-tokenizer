from contextlib import nullcontext
from io import BytesIO, StringIO

import pytest

from json_stream_rs_tokenizer import RustTokenizer


@pytest.mark.parametrize(
    "buffering,expected_str_cursor_pos,expected_bytes_cursor_pos",
    [
        (1, 7, 10),  # unbuffered
        (2000, 14, 17),  # large buffer
        (-1, 14, 17),  # don't care => should choose large buf
    ],
)
def test_buffering_cursor_pos(
    buffering,
    expected_str_cursor_pos,
    expected_bytes_cursor_pos,
    to_bytes_or_str_buf,
):
    """
    Test that buffering setting is respected by checking cursor pos.
    """
    buf = to_bytes_or_str_buf('[ "äöµ", "a" ]')
    tokenizer = RustTokenizer(buf, buffering=buffering, correct_cursor=False)
    for kind, val in tokenizer:
        if val == "äöµ":
            break
    else:
        assert False, "didn't find expected list elem for some reason"
    if isinstance(buf, StringIO):
        # for text streams, tell() and seek() positions are opaque numbers
        # => compare by checking equality to position after reading N chars
        pos = buf.tell()
        buf.seek(0, 0)  # go back to start
        buf.read(expected_str_cursor_pos)  # read N chars
        expected_str_cursor_opaque_pos = buf.tell()
        assert pos == expected_str_cursor_opaque_pos
    elif isinstance(buf, BytesIO):
        assert buf.tell() == expected_bytes_cursor_pos
    else:
        assert False, "what"


@pytest.mark.parametrize(
    "buffering",
    [
        1,  # unbuffered
        2000,  # large buffer
        -1,  # don't care => should choose large buf
    ],
)
def test_buffering_cursor_pos_with_correct_cursor_enforcement(
    buffering, to_bytes_or_str_buf
):
    """
    Test that buffering setting plays nicely with correct_cursor.

    Could have gone into either the overconsumption tests or here, but here
    should be fine.
    """
    buf = to_bytes_or_str_buf('[ "äöµ", "a" ]')
    raises_ctx = (
        pytest.raises(ValueError)
        if not buf.seekable() and buffering > 1
        else nullcontext()
    )
    with raises_ctx:
        tokenizer = RustTokenizer(
            buf, buffering=buffering, correct_cursor=True
        )
    if raises_ctx != nullcontext:
        return  # nothing else to check
    for kind, val in tokenizer:
        if val == "äöµ":
            break
    else:
        assert False, "didn't find expected list elem for some reason"
    if isinstance(buf, StringIO):
        # for text streams, tell() and seek() positions are opaque numbers
        # => compare by checking equality to position after reading N chars
        pos = buf.tell()
        buf.seek(0, 0)  # go back to start
        buf.read(7)  # read N chars
        expected_str_cursor_opaque_pos = buf.tell()
        assert pos == expected_str_cursor_opaque_pos
    elif isinstance(buf, BytesIO):
        assert buf.tell() == 10
    else:
        assert False, "what"
