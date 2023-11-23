use crate::pyclass_boxed_suitable_stream::PyClassBoxedSuitableStream;
use crate::suitable_stream::make_suitable_stream;
use crate::unicode_utils::{decode_surrogate_pair, is_surrogate};
use crate::{BufferingMode, CharOrEof, JsonStreamingError, ParsingError};
use compact_str::CompactString;
use pyo3::prelude::*;
use std::io;
use CharOrEof::{Char, Eof};

#[derive(Clone)]
enum StringState {
    String_ = 9,
    StringEscape = 10,
    Unicode = 22,
    UnicodeSurrogateStart = 23,
    UnicodeSurrogateStringEscape = 24,
    UnicodeSurrogate = 25,
}

/// A streaming parser for the contents of strings within JSON.
///
/// Should not normally be instantiated by the user directly.
///
/// Args:
///   stream: Python file-like object / stream to read the JSON string contents
///     from. Can be either in text mode or in binary mode (so long as the bytes
///     are valid UTF-8).
///   buffering: Internal buffer size. -1 (the default) means to let the
///     implementation choose a buffer size. Can conflict with `correct_cursor`.
///   correct_cursor: *(not part of API yet, may be removed at any point)*
///     Whether it is required that the cursor is left in the correct position
///     (behind the last processed character) after park_cursor() has been
///     called. If set to False, performance for unseekable streams is
///     drastically improved at the cost of the cursor ending up in places
///     unrelated to the actual tokenization progress. For seekable streams, the
///     improvement shouldn't be noticable.
#[pyclass]
#[pyo3(text_signature = "(stream, *, buffering=-1, correct_cursor=True)")]
pub struct JsonStringReader {
    stream: Py<PyClassBoxedSuitableStream>,
    completed: bool,
    state: StringState,
    pub index: i64,
    unicode_buffer: CompactString,
    prev_charcode: Option<u16>, // first half of a Unicode surrogate pair
}

#[pymethods]
impl JsonStringReader {
    #[new]
    #[args("*", buffering = -1, strings_as_files = "false", correct_cursor = "true")]
    fn new(
        stream: PyObject,
        buffering: i64,
        correct_cursor: bool,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let buffering_mode = if buffering < 0 {
            BufferingMode::DontCare
        } else if buffering == 0 || buffering == 1 {
            BufferingMode::Unbuffered
        } else {
            BufferingMode::BufferedWithSize(buffering.try_into().unwrap())
        };
        let stream = PyClassBoxedSuitableStream::new(make_suitable_stream(
            stream,
            buffering_mode,
            correct_cursor,
        )?);
        Ok(JsonStringReader {
            stream: Py::new(py, stream)?,
            completed: false,
            state: StringState::String_,
            index: 0,
            unicode_buffer: CompactString::with_capacity(4),
            prev_charcode: None,
        })
    }

    #[args(size = -1, "/")]
    #[pyo3(text_signature = "($self, size=-1, /)")]
    pub fn read(&mut self, size: Option<isize>, py: Python<'_>) -> PyResult<String> {
        // normalize size arg
        let max_n_chars: Option<usize> = match size {
            None => None,
            Some(size) if size < 0 => None,
            Some(size) if size == 0 => return Ok("".to_owned()),
            Some(size) => Some(size as usize),
        };
        // /normalize
        self.read_string_contents(max_n_chars, py).map_err(|e| {
            let index = self.index;
            e.to_py_error_at_index(index as isize)
        })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(slf: PyRefMut<'_, Self>, py: Python<'_>) -> PyResult<Option<String>> {
        JsonStringReader::readline(slf, None, py)
    }

    fn readline(mut slf: PyRefMut<'_, Self>, size: Option<isize>, py: Python<'_>) -> PyResult<Option<String>> {
        // normalize size arg
        let max_n_chars: Option<usize> = match size {
            None => None,
            Some(size) if size < 0 => None,
            Some(size) if size == 0 => return Ok(Some("".to_owned())),
            Some(size) => Some(size as usize),
        };
        // /normalize
        JsonStringReader::read_until_newline(&mut slf, max_n_chars, py).map_err(|e| {
            let index = slf.index;
            e.to_py_error_at_index(index as isize)
        })
    }
}

impl JsonStringReader {
    pub fn from_existing_py_pyclass_boxed_suitable_stream(
        stream: Py<PyClassBoxedSuitableStream>,
    ) -> Self {
        Self {
            stream,
            completed: false,
            state: StringState::String_,
            index: 0,
            unicode_buffer: CompactString::with_capacity(4),
            prev_charcode: None,
        }
    }

    fn read_string_contents<'a>(
        &mut self,
        max_n_chars: Option<usize>,
        py: Python<'_>,
    ) -> Result<String, JsonStreamingError> {
        if self.completed {
            return Ok(String::new());
        }
        let mut s = String::new();
        while max_n_chars.map_or(true, |n| s.len() < n) {
            match Self::read_and_process_until_1_char(self, py)? {
                Char(c_out) => s.push(c_out),
                Eof => {
                    self.completed = true;
                    break;
                }
            }
        }
        Ok(s)
    }

    fn read_until_newline(
        &mut self,
        max_n_chars: Option<usize>,
        py: Python<'_>,
    ) -> Result<Option<String>, JsonStreamingError> {
        if self.completed {
            return Ok(None);
        }
        let mut s = String::new();
        while max_n_chars.map_or(true, |n| s.len() < n) {
            match Self::read_and_process_until_1_char(self, py)? {
                Char(c_out) => {
                    s.push(c_out);
                    if c_out == '\n' {
                        break;
                    };
                }
                Eof => {
                    self.completed = true;
                    break;
                }
            }
        }
        Ok(Some(s))
    }

    fn read_and_process_until_1_char(
        self: &mut Self,
        py: Python<'_>,
    ) -> Result<CharOrEof, JsonStreamingError> {
        loop {
            let c = match self
                .stream
                .borrow_mut(py)
                .read_char()
                .map_err(|e| <io::Error as Into<JsonStreamingError>>::into(e))?
            {
                Some(c) => Char(c),
                None => Eof,
            };
            self.index += 1;
            if let Some(char_or_eof_out) = Self::process_char(self, c)? {
                return Ok(char_or_eof_out);
            }
        }
    }

    /// Returning `Eof` here means end of string, not end of file (which would return an error).
    fn process_char(slf: &mut Self, c: CharOrEof) -> Result<Option<CharOrEof>, ParsingError> {
        let mut add_char = false;
        let mut c = c;

        match slf.state {
            StringState::String_ => match c {
                Char('\"') => {
                    c = Eof;
                    add_char = true;
                }
                Char('\\') => {
                    slf.state = StringState::StringEscape;
                }
                Eof => {
                    return Err(ParsingError::InvalidJson(
                        "Unterminated string at end of file".to_string(),
                    ));
                }
                _ => {
                    add_char = true;
                }
            },
            StringState::StringEscape => {
                slf.state = StringState::String_;
                match c {
                    Char('\\' | '\"') => {
                        add_char = true;
                    }
                    Char('b') => {
                        c = Char(8u8 as char);
                        add_char = true;
                    }
                    Char('f') => {
                        c = Char(12u8 as char);
                        add_char = true;
                    }
                    Char('n') => {
                        c = Char('\n');
                        add_char = true;
                    }
                    Char('t') => {
                        c = Char('\t');
                        add_char = true;
                    }
                    Char('r') => {
                        c = Char('\r');
                        add_char = true;
                    }
                    Char('/') => {
                        c = Char('/');
                        add_char = true;
                    }
                    Char('u') => {
                        slf.state = StringState::Unicode;
                        slf.unicode_buffer = CompactString::with_capacity(4);
                    }
                    _ => {
                        return Err(ParsingError::InvalidJson(format!(
                            "Invalid string escape: {c}"
                        )));
                    }
                }
            }
            StringState::Unicode => {
                match c {
                    Char(c) => {
                        slf.unicode_buffer.push(c);
                    }
                    Eof => {
                        return Err(ParsingError::InvalidJson(format!(
                            "Unterminated unicode literal at end of file"
                        )));
                    }
                }
                if slf.unicode_buffer.len() == 4 {
                    let Ok(charcode) = u16::from_str_radix(
                        slf.unicode_buffer.as_str(), 16
                    ) else {
                        let unicode_buffer = slf.unicode_buffer.as_str();
                        return Err(ParsingError::InvalidJson(format!(
                            "Invalid unicode literal: \\u{unicode_buffer}"
                        )));
                    };
                    match char::from_u32(charcode as u32) {
                        Some(unicode_char) => {
                            c = Char(unicode_char);
                            add_char = true;
                            slf.state = StringState::String_;
                        }
                        None if is_surrogate(charcode) => {
                            slf.prev_charcode = Some(charcode);
                            slf.state = StringState::UnicodeSurrogateStart;
                        }
                        None => {
                            // should never happen
                            return Err(ParsingError::InvalidJson(format!(
                                "No unicode character for code: {charcode}"
                            )));
                        }
                    }
                }
            }
            StringState::UnicodeSurrogateStart => match c {
                Char('\\') => {
                    slf.state = StringState::UnicodeSurrogateStringEscape;
                }
                Char(_) => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Unpaired UTF-16 surrogate"
                    )));
                }
                Eof => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Unpaired UTF-16 surrogate at end of file"
                    )));
                }
            },
            StringState::UnicodeSurrogateStringEscape => match c {
                Char('u') => {
                    slf.unicode_buffer = CompactString::with_capacity(4);
                    slf.state = StringState::UnicodeSurrogate;
                }
                Char(_) => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Unpaired UTF-16 surrogate"
                    )));
                }
                Eof => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Unpaired UTF-16 surrogate at end of file"
                    )));
                }
            },
            StringState::UnicodeSurrogate => {
                match c {
                    Char(c) => {
                        slf.unicode_buffer.push(c);
                    }
                    Eof => {
                        return Err(ParsingError::InvalidJson(format!(
                            "Unterminated unicode literal at end of file"
                        )));
                    }
                }
                if slf.unicode_buffer.len() == 4 {
                    let Ok(charcode) = u16::from_str_radix(
                        slf.unicode_buffer.as_str(), 16
                    ) else {
                        let unicode_buffer = slf.unicode_buffer.as_str();
                        return Err(ParsingError::InvalidJson(format!(
                            "Invalid unicode literal: \\u{unicode_buffer}"
                        )));
                    };
                    if !is_surrogate(charcode) {
                        return Err(ParsingError::InvalidJson(format!(
                            "Second half of UTF-16 surrogate pair is not a surrogate!"
                        )));
                    }
                    let Some(prev_charcode) = slf.prev_charcode else {
                        return Err(ParsingError::InvalidJson(format!(
                            "This should never happen, please report it as a bug..."
                        )));
                    };
                    c = Char(decode_surrogate_pair(prev_charcode, charcode).map_err(|_| {
                        ParsingError::InvalidJson(format!(
                            "Error decoding UTF-16 surrogate pair \
                            \\u{prev_charcode:x}\\u{charcode:x}"
                        ))
                    })?);
                    slf.prev_charcode = None;
                    slf.state = StringState::String_;
                    add_char = true;
                }
            }
        }

        Ok(if add_char { Some(c) } else { None })
    }
}
