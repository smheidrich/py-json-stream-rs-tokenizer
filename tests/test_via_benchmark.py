from json_stream_rs_tokenizer.benchmark import main


def test_via_benchmark():
    """
    Just check that it runs without errors
    """
    speedup = main(json_bytes=1e5)
    assert speedup > 1, "no speedup - make sure it's built in release mode"
