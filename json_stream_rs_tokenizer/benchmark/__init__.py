from pathlib import Path
import random
from tempfile import TemporaryDirectory

from contexttimer import Timer
import json_stream as js
import json_stream_rs_tokenizer as jsrs
from tqdm import tqdm

from .random_json_generator import RandomJsonGenerator


def shuffled(l):
    return random.sample(l, k=len(l))


def main():
    try:
        js.to_standard_types
    except NameError:
        print(
            "benchmarks currently require a patched version of json-stream "
            "available at https://github.com/daggaz/json-stream/pull/17"
        )
        exit(1)
    with TemporaryDirectory() as tmp_dir:
        random_json_file_path = Path(tmp_dir) / "random.json"
        print("generating random json...")
        random_json_str = RandomJsonGenerator().random_list(target_len=100)
        random_json_size = len(random_json_str.encode("utf-8"))
        print(
            f"generated random json {random_json_file_path} "
            f"with size {random_json_size:.3e} bytes"
        )
        random_json_file_path.write_text(random_json_str)
        results = {"python": {}, "rust": {}}
        for tokenizer_type, load_fn in shuffled(
            [("python", js.load), ("rust", jsrs.load)]
        ):
            print(f"running with {tokenizer_type} tokenizer")
            with random_json_file_path.open() as f:
                l = load_fn(f)
                with Timer() as t:
                    parsed = [js.to_standard_types(x) for x in tqdm(l)]
                print(f"{tokenizer_type} time: {t.elapsed:.2f} s")
                results[tokenizer_type]["elapsed"] = t.elapsed
                results[tokenizer_type]["parsed"] = parsed
        assert (
            results["python"]["parsed"] == results["rust"]["parsed"]
        ), "BUG: Rust and Py results differ!"
        speedup = results["python"]["elapsed"]/results["rust"]["elapsed"]
        print(f"speedup: {speedup:.2f}")
