from io import StringIO
import re

import pytest

from json_stream_rs_tokenizer import load


def test_free_charater():
    with pytest.raises(
        ValueError,
        match=re.escape(
            "Error while parsing at index 0: invalid JSON: "
            "Invalid JSON character: 'a'"
        ),
    ):
        list(load(StringIO("a")))


def test_letter_in_number():
    with pytest.raises(
        ValueError,
        match=re.escape(
            "Error while parsing at index 4: invalid JSON: "
            "A number must contain only digits.  Got 'a'"
        ),
    ):
        list(load(StringIO("[123a]")))


def test_invalid_character_code():
    with pytest.raises(
        ValueError,
        match=re.escape(
            "Error while parsing at index 3: invalid JSON: "
            "Invalid character code: 'z'"
        ),
    ):
        list(load(StringIO(r'"\uz"')))
