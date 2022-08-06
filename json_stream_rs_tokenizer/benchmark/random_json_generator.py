from contextlib import nullcontext
import json
import random
from tqdm import tqdm


DEFAULT_MAX_BYTES = 2e6


class RandomJsonGenerator:
    def __init__(self, max_depth=8, ensure_ascii=False):
        self.max_depth = max_depth
        self.ensure_ascii = ensure_ascii

    def random_list(self, max_bytes=DEFAULT_MAX_BYTES, target_len=None):
        return self._dumps(self._random_list(0, max_bytes, target_len)[0])

    def random_dict(self, max_bytes=DEFAULT_MAX_BYTES, target_len=None):
        return self._dumps(self._random_dict(0, max_bytes)[0])

    def _random_list(
        self, depth=0, max_bytes=DEFAULT_MAX_BYTES, target_len=None
    ):
        l = []
        bytes_here = 2
        if depth >= self.max_depth or bytes_here >= max_bytes:
            return l, bytes_here
        n_elems = target_len or 1 + int(random.random() * 10)
        elem_types = [
            self._random_list,
            self._random_dict,
            self._random_str,
        ] + ([self._random_int, self._random_float] if max_bytes < 10 else [])
        t = tqdm() if depth == 0 else nullcontext()
        t.total = max_bytes
        with t:
            while bytes_here < max_bytes:
                if bytes_here > 2:  # i.e. not first
                    bytes_here += 2  # ", "
                elems_left = n_elems-len(l)
                bytes_left = max_bytes-bytes_here
                elem, elem_bytes = random.choice(elem_types)(
                    depth + 1, bytes_left / (elems_left or -1)
                )
                bytes_here += elem_bytes
                l.append(elem)
                if depth == 0:
                    t.n = bytes_here
                    t.refresh()
            if depth == 0:
                t.n = max_bytes
                t.refresh()
        return l, bytes_here

    def _random_dict(self, depth=0, max_bytes=DEFAULT_MAX_BYTES):
        d = {}
        bytes_here = 2
        if depth >= self.max_depth or bytes_here >= max_bytes:
            return d, bytes_here
        n_elems = 1 + int(random.random() * 10)
        elem_types = [
            self._random_list,
            self._random_dict,
            self._random_str,
        ] + ([self._random_int, self._random_float] if max_bytes < 10 else [])
        t = tqdm() if depth == 0 else nullcontext()
        t.total = max_bytes
        with t:
            while bytes_here < max_bytes:
                key, key_bytes = self._random_str(
                    depth + 1, max(max_bytes / n_elems, 4)
                )
                if key in d:
                    continue
                if bytes_here > 2:  # i.e. not first
                    bytes_here += 2  # ", "
                bytes_here += key_bytes + 2
                value, value_bytes = random.choice(elem_types)(
                    depth + 1, max_bytes / n_elems
                )
                d[key] = value
                bytes_here += value_bytes
                if depth == 0:
                    t.n = bytes_here
                    t.refresh()
            if depth == 0:
                t.n = max_bytes
                t.refresh()
        return d, bytes_here

    def _random_str(self, depth=0, max_bytes=DEFAULT_MAX_BYTES):
        nonsingle = "\b\t\nµæðøæſ"
        choices = (
            " 0123456789"
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            "abcdefghijklmnopqrstuvwxyz" + nonsingle
        )
        nonsingle = set(nonsingle)
        if max_bytes <= 2:
            return "", 2
        l = []
        bytes_here = 2
        while bytes_here < max_bytes:
            c = random.choice(choices)
            if c not in nonsingle:  # just to speed up
                bytes_here += 1
            else:
                bytes_here += len(self._dumps(c).encode("utf-8")) - 2
            l.append(c)
        return "".join(l), bytes_here

    def _random_int(self, depth=0, max_bytes=DEFAULT_MAX_BYTES):
        n = int(self._random_float(depth, max_bytes, 100000)[0])
        return n, len(str(n))

    def _random_float(self, depth=0, max_bytes=DEFAULT_MAX_BYTES, mult=None):
        if mult is None:
            mult = float(f"1e{int(random.random()*20)}")
        f = random.random() * mult
        return f, len(self._dumps(f))

    def _dumps(self, x):
        return json.dumps(x, ensure_ascii=self.ensure_ascii)
