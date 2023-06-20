import json
import random
from functools import partial
from pathlib import Path
from tempfile import TemporaryDirectory

import json_stream as js
from contexttimer import Timer
from json_stream.tokenizer import tokenize as pure_python_tokenizer
from json_stream_to_standard_types import to_standard_types
from tqdm import tqdm

import json_stream_rs_tokenizer as jsrs

from .random_json_generator import RandomJsonGenerator


def shuffled(l):
    return random.sample(l, k=len(l))


def main(json_bytes=2e6):
    with TemporaryDirectory() as tmp_dir:
        random_json_file_path = Path(tmp_dir) / "random.json"
        print("generating random json...")
        random_json_str = RandomJsonGenerator().random_list(
            max_bytes=json_bytes, target_len=100
        )
        random_json_size = len(random_json_str.encode("utf-8"))
        print(
            f"generated random json {random_json_file_path} "
            f"with size {random_json_size:.3e} bytes"
        )
        random_json_file_path.write_text(random_json_str, encoding="utf-8")
        results = {"python": {}, "rust": {}, "non-streaming": {}}
        for tokenizer_type, load_fn in shuffled(
            [
                ("python", partial(js.load, tokenizer=pure_python_tokenizer)),
                ("rust", partial(js.load, tokenizer=jsrs.RustTokenizer)),
                ("non-streaming", json.load),
            ]
        ):
            print(f"running with {tokenizer_type} tokenizer")
            with Timer() as t:
                with random_json_file_path.open(encoding="utf-8") as f:
                    l = load_fn(f)
                    parsed = [to_standard_types(x) for x in tqdm(l, total=100)]
            print(f"{tokenizer_type} time: {t.elapsed:.2f} s")
            results[tokenizer_type]["elapsed"] = t.elapsed
            results[tokenizer_type]["parsed"] = parsed
        assert (
            results["python"]["parsed"] == results["rust"]["parsed"]
        ), "BUG: Rust and Py results differ!"
        assert (
            results["non-streaming"]["parsed"] == results["rust"]["parsed"]
        ), "BUG: non-streaming and streaming results differ!"
        speedup = results["python"]["elapsed"] / results["rust"]["elapsed"]
        print(f"speedup: {speedup:.2f}")
    return speedup
