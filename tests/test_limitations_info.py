from json_stream_rs_tokenizer import supports_bigint


def test_supports_bigint():
    assert supports_bigint() is True
