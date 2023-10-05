from json_stream_rs_tokenizer import RustTokenizer

class InfiniteJsonFile:
  start = "["
  elem = "1234, \n"

  def __init__(self):
    self.remainder = self.start
    self.size = 0

  def read(self, size):
    if size == 0:
      return self.start[:0]
    s = self.remainder + self.elem * (size // len(self.elem) + 1)
    self.remainder = s[size:]
    s = s[:size]
    self.size += len(s)
    return s

  def seek(self, offset, whence):
    match whence:
      case 0:
        if offset == 0:
          self.remainder = self.start
        else:
          self.remainder = self.elem[(offset-1) % len(self.elem):]
        self.size = offset
      case 1:
        assert offset == 0
      case 2:
        raise NotImplementedError("seek from end not impl")
    return self.size

  def seekable(self):
    return True

# f = InfiniteJsonFile()
# for x in range(60):
  # print(f.read(8), end="")

f = InfiniteJsonFile()
N = 100000
with open("temp.json", "w") as o:
  for i in range(N+1):
    if i % (N/10) == 0:
      print(f"{i}/{N} ({i*10000}/{N*10000} bytes)")
    o.write(f.read(10000))

with open("temp.json") as f:
  size = 0
  r = RustTokenizer(f)
  for i, tok in enumerate(r):
    if i % 1000000 == 0:
      print(f"{i} ({size} bytes)")
    size += len(next(r))
