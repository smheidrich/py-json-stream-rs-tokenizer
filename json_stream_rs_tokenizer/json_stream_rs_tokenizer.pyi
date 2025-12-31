"""
Manually written type hints stub file until PyO3 supports stub generation.

See https://pyo3.rs/v0.27.1/python-typing-hints.html
"""
from typing import Any, IO, final

@final
class RustTokenizer:
  # TODO: buffering default is actually -1 but Mypy insists on it being
  #       ellipsis...
  def __new__(
    cls, stream: IO[Any], *, buffering: int = ..., correct_cursor: bool = False
  ) -> RustTokenizer: ...

  def park_cursor(self) -> None: ...

  @property
  def remainder(self) -> str | bytes: ...

def supports_bigint() -> bool: ...

__all__ = [
  "RustTokenizer",
  "supports_bigint",
]
