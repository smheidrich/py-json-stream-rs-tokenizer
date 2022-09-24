from io import StringIO
from json_stream_rs_tokenizer import load


def test_large_integers():
    assert list(load(StringIO(f"[{2**63}]"))) == [2**63]
    assert list(load(StringIO(f"[{10**200}]"))) == [10**200]
    assert list(load(StringIO(f"[{-10**200}]"))) == [-10**200]
