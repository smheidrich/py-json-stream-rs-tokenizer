"""
Run json-stream's own tokenizer tests but patch in our tokenizer
"""
from unittest.mock import patch

import pytest

from json_stream.tests.test_tokenizer import TestJsonTokenization
from json_stream_rs_tokenizer import RustTokenizer


@pytest.fixture(autouse=True, scope="module")
def override_tokenizer():
    with patch("json_stream.tests.test_tokenizer.tokenize", RustTokenizer):
        yield


__all__ = ["override_tokenizer", "TestJsonTokenization"]
