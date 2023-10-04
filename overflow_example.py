# from: https://github.com/smheidrich/py-json-stream-rs-tokenizer/issues/93#issuecomment-1744756519
import json
import json_stream
from json_stream import streamable_list, streamable_dict
from random import choice
from string import ascii_lowercase
from tqdm import tqdm


def main():
    print("Generating ...")
    with open("foo.json", "w") as f:
        json.dump(
            obj=streamable_list(
                tqdm(
                    iterable=random_json(
                        n_lists=100,
                        n_dicts=100,
                        n_items_per_dict=100,
                    ),
                    total=100,
                )
            ),
            fp=f,
            indent=4,
        )

    print("Reading ...")
    with open("foo.json", "r") as f:
        foo = json_stream.load(f)
        for bars in foo:
            for bar in bars:
                for k, v in bar.items():
                    print(k, v)

    print("Done.")


def random_json(
    *,
    n_lists: int,
    n_dicts: int,
    n_items_per_dict: int,
):
    for _ in range(n_lists):
        yield streamable_list(
            random_dicts(n_dicts=n_dicts, n_items_per_dict=n_items_per_dict)
        )


def random_dicts(
    *,
    n_dicts: int,
    n_items_per_dict: int,
):
    for _ in range(n_dicts):
        yield streamable_dict(random_dict_items(n_items_per_dict=n_items_per_dict))


def random_dict_items(
    *,
    n_items_per_dict: int,
):
    for _ in range(n_items_per_dict):
        yield random_string(), random_value()


def random_value():
    return choice(
        [
            random_int,
            random_float,
            random_string,
            random_bool,
            lambda: None,
        ]
    )()


def random_string():
    return "".join(choice(ascii_lowercase) for _ in range(10))


def random_int():
    return choice([-1, 0, 1]) * choice(range(100))


def random_float():
    return choice([-1, 1]) * choice(range(100)) / 10


def random_bool():
    return choice([True, False])


if __name__ == "__main__":
    main()
