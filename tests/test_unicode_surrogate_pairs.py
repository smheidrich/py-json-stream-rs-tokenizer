from io import StringIO

import pytest

from json_stream_rs_tokenizer import load


@pytest.mark.parametrize(
    "s,expected",
    [
        (r"\uD83C\uDFD4\uFE0F", "\U0001f3d4\uFE0F"),
        (r"a\uD83C\uDFD4\uFE0F", "a\U0001f3d4\uFE0F"),
        (r"\uD83C\uDFD4\uFE0Fa", "\U0001f3d4\uFE0Fa"),
    ],
)
def test_unicode_surrogate_pairs(s, expected):
    assert list(load(StringIO(f'["{s}"]'))) == [expected]


@pytest.mark.parametrize(
    "s",
    [
        r"\uD83Ca",
        r"\uD83C",
        r"\uD83C\n",
        r"\uD83C\u00e4",
    ],
)
def test_invalid_unicode_surrogate_pairs(s):
    with pytest.raises(ValueError, match=".*surrogate.*"):
        assert list(load(StringIO(f'["{s}"]')))
