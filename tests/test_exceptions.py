from io import StringIO
import re

import pytest

from json_stream_rs_tokenizer import load


def test_free_charater():
    with pytest.raises(
        ValueError,
        match=re.escape("Invalid JSON character: 'a' at index 0"),
    ):
        list(load(StringIO("a")))


def test_letter_in_number():
    with pytest.raises(
        ValueError,
        match=re.escape(
            "A number must contain only digits.  Got 'a' at index 4"
        ),
    ):
        list(load(StringIO("[123a]")))


def test_invalid_number_starting_with_zero():
    with pytest.raises(
        ValueError,
        match=re.escape(
            "A 0 must be followed by a '.' | a 'e'.  Got '1' at index 1"
        ),
    ):
        list(load(StringIO(r"01")))


def test_invalid_character_code():
    with pytest.raises(
        ValueError,
        match=re.escape("Invalid character code: 'z' at index 3"),
    ):
        list(load(StringIO(r'"\uz"')))
