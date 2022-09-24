from io import StringIO
from json_stream_rs_tokenizer import load


def test_integers():
    assert list(load(StringIO("[123]"))) == [123]
    assert list(load(StringIO("[123e3]"))) == [123e3]
    assert list(load(StringIO("[123E3]"))) == [123E3]
    assert list(load(StringIO("[-123E3]"))) == [-123E3]
