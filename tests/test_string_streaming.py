import pytest

from json_stream_rs_tokenizer import RustTokenizer
from json_stream.tokenizer import TokenType


@pytest.mark.parametrize(
    "buffering",
    [
        1,  # unbuffered
        2000,  # large buffer
        -1,  # don't care => should choose large buf
    ],
)
def test_basic_read(buffering, to_bytes_or_str_buf):
    buf = to_bytes_or_str_buf('[ "Hello, World!", "a" ]')
    tokenizer = RustTokenizer(
        buf, buffering=buffering, correct_cursor=False, strings_as_files=True
    )
    assert next(tokenizer) == (TokenType.OPERATOR, "[")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert val.read() == "Hello, World!"
    assert next(tokenizer) == (TokenType.OPERATOR, ",")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert val.read() == "a"
    assert next(tokenizer) == (TokenType.OPERATOR, "]")
    with pytest.raises(StopIteration):
        next(tokenizer)


@pytest.mark.parametrize(
    "buffering",
    [
        1,  # unbuffered
        2000,  # large buffer
        -1,  # don't care => should choose large buf
    ],
)
def test_partial_read_and_skip(buffering, to_bytes_or_str_buf):
    buf = to_bytes_or_str_buf('[ "Hello, World!", "a" ]')
    tokenizer = RustTokenizer(
        buf, buffering=buffering, correct_cursor=False, strings_as_files=True
    )
    assert next(tokenizer) == (TokenType.OPERATOR, "[")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert val.read(5) == "Hello"
    assert next(tokenizer) == (TokenType.OPERATOR, ",")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert val.read() == "a"
    assert next(tokenizer) == (TokenType.OPERATOR, "]")
    with pytest.raises(StopIteration):
        next(tokenizer)


@pytest.mark.parametrize(
    "buffering",
    [
        1,  # unbuffered
        2000,  # large buffer
        -1,  # don't care => should choose large buf
    ],
)
def test_partial_read_and_read_rest(buffering, to_bytes_or_str_buf):
    buf = to_bytes_or_str_buf('[ "Hello, World!", "a" ]')
    tokenizer = RustTokenizer(
        buf, buffering=buffering, correct_cursor=False, strings_as_files=True
    )
    assert next(tokenizer) == (TokenType.OPERATOR, "[")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert val.read(5) == "Hello"
    assert val.read() == ", World!"
    assert next(tokenizer) == (TokenType.OPERATOR, ",")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert val.read() == "a"
    assert next(tokenizer) == (TokenType.OPERATOR, "]")
    with pytest.raises(StopIteration):
        next(tokenizer)


@pytest.mark.parametrize(
    "buffering",
    [
        1,  # unbuffered
        2000,  # large buffer
        -1,  # don't care => should choose large buf
    ],
)
def test_read_lines(buffering, to_bytes_or_str_buf):
    buf = to_bytes_or_str_buf('[ "Hello\nWorld!", "a" ]')
    tokenizer = RustTokenizer(
        buf, buffering=buffering, correct_cursor=False, strings_as_files=True
    )
    assert next(tokenizer) == (TokenType.OPERATOR, "[")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert list(val) == ["Hello\n", "World!"]
    assert next(tokenizer) == (TokenType.OPERATOR, ",")
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert val.read() == "a"
    assert next(tokenizer) == (TokenType.OPERATOR, "]")
    with pytest.raises(StopIteration):
        next(tokenizer)


# less extensive tests for other methods:


def test_readline(to_bytes_or_str_buf):
    buf = to_bytes_or_str_buf('"Hello\nWorld!"')
    tokenizer = RustTokenizer(buf, strings_as_files=True)
    kind, val = next(tokenizer)
    assert kind == TokenType.STRING
    assert list([val.readline(), val.readline()]) == ["Hello\n", "World!"]
