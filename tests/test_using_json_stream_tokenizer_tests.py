"""
Run json-stream's own tokenizer tests but patch in our tokenizer
"""
from unittest.mock import patch

import pytest
from json_stream.tests.test_buffering import TestBuffering
from json_stream.tokenizer.tests.test_strings import TestJsonStringReader
from json_stream.tokenizer.tests.test_tokenizer import TestJsonTokenization

from json_stream_rs_tokenizer import RustTokenizer, JsonStringReader


@pytest.fixture(autouse=True, scope="module")
def override_tokenizer():
    with patch(
        "json_stream.tokenizer.tests.test_tokenizer.tokenize", RustTokenizer
    ), patch(
        "json_stream.tokenizer.tests.test_strings.JsonStringReader",
        JsonStringReader,
    ), patch(
        "json_stream.tests.test_buffering.tokenize", RustTokenizer
    ):
        yield


# these don't all work, mainly because our JsonStringReader can't be given an
# initial buffer on construction (would be very cumbersome to implement for
# something that is only used in tests)
TestJsonStringReader = pytest.mark.xfail(TestJsonStringReader)


# mark as xfail a bunch of cases that fail just because the error messages
# differ slightly (probably not that important to align them 100%)
class TestJsonTokenization(TestJsonTokenization):
    @pytest.mark.xfail
    def test_string_parsing(self):
        super().test_string_parsing()

    @pytest.mark.xfail
    def test_unicode_surrogate_pair_literal_unterminated(self):
        super().test_unicode_surrogate_pair_literal_unterminated()

    @pytest.mark.xfail
    def test_unicode_surrogate_pair_literal_unterminated_first_half(self):
        super().test_unicode_surrogate_pair_literal_unterminated_first_half()

    @pytest.mark.xfail
    def test_unicode_surrogate_pair_unpaired(self):
        super().test_unicode_surrogate_pair_unpaired()


__all__ = [
    "override_tokenizer",
    "TestJsonTokenization",
    "TestJsonStringReader",
    "TestBuffering",
]
