from io import BytesIO, StringIO

import pytest


class StringIOWithLargeCursorPositions(StringIO):
    """
    Simulates very large (> 2^64) cursor positions as occur on Windows.
    """

    def seek(self, offset, whence):
        return super().seek(offset, whence) + 2**64

    def tell(self):
        return super().tell() + 2**64


@pytest.fixture(
    params=[
        "str",
        "bytes",
        "str-unseekable",
        "bytes-unseekable",
        "str-largecursorpos",
    ]
)
def to_bytes_or_str_buf(request):
    if request.param == "str":
        return lambda s: StringIO(s)
    elif request.param == "str-unseekable":

        def make_unseekable_stringio(s: str):
            sio = StringIO(s)
            sio.seekable = lambda: False
            return sio

        return make_unseekable_stringio
    elif request.param == "str-largecursorpos":
        return lambda s: StringIOWithLargeCursorPositions(s)
    elif request.param == "bytes":
        return lambda s: BytesIO(s.encode("utf-8"))
    elif request.param == "bytes-unseekable":

        def make_unseekable_bytesio(s: str):
            bio = BytesIO(s.encode("utf-8"))
            bio.seekable = lambda: False
            return bio

        return make_unseekable_bytesio
    else:
        assert False
