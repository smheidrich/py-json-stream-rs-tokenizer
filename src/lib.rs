/// Rust port of json-stream's tokenizer.
/// https://github.com/daggaz/json-stream
/// Copyright (c) 2020 Jamie Cockburn
/// json-stream's tokenizer was originally taken from the NAYA project.
/// https://github.com/danielyule/naya
/// Copyright (c) 2019 Daniel Yule
use crate::int::{AppropriateInt, ParseIntError};
use crate::json_string_reader::JsonStringReader;
use crate::remainder::StreamData;
use crate::suitable_stream::make_suitable_stream;
use pyclass_boxed_suitable_stream::PyClassBoxedSuitableStream;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::borrow::BorrowMut;
use std::io;
use std::num::ParseFloatError;
use std::str::FromStr;
use thiserror::Error;

mod int;
mod json_string_reader;
mod opaque_seek;
mod park_cursor;
mod py_bytes_stream;
mod py_common;
mod py_text_stream;
mod pyclass_boxed_suitable_stream;
mod read_string;
mod remainder;
mod suitable_seekable_buffered_bytes_stream;
mod suitable_seekable_buffered_text_stream;
mod suitable_stream;
mod suitable_unbuffered_bytes_stream;
mod suitable_unbuffered_text_stream;
mod suitable_unseekable_buffered_bytes_stream;
mod suitable_unseekable_buffered_text_stream;
mod utf8_char_source;

mod char_or_eof;
use crate::char_or_eof::CharOrEof;
use CharOrEof::{Char, Eof};

mod unicode_utils;
use crate::unicode_utils::UnicodeError;

use crate::suitable_stream::BufferingMode;

#[derive(Clone)]
enum TokenType {
    Operator = 0,
    String_ = 1,
    Number = 2,
    Boolean = 3,
    Null = 4,
}

#[derive(Clone)]
enum State {
    Whitespace = 0,
    Integer0 = 1,
    IntegerSign = 2,
    Integer = 3,
    IntegerExp = 4,
    IntegerExp0 = 5,
    FloatingPoint0 = 6,
    FloatingPoint = 8,
    StringEnd = 11,
    True1 = 12,
    True2 = 13,
    True3 = 14,
    False1 = 15,
    False2 = 16,
    False3 = 17,
    False4 = 18,
    Null1 = 19,
    Null2 = 20,
    Null3 = 21,
}

/// A drop-in replacement for json-stream's JSON tokenizer, written in Rust.
///
/// Args:
///   stream: Python file-like object / stream to read JSON from. Can be
///     either in text mode or in binary mode (so long as the bytes are valid
///     UTF-8).
///   buffering: Internal buffer size. -1 (the default) means to let the
///     implementation choose a buffer size. Can conflict with `correct_cursor`.
///   strings_as_files: Whether to return strings as file-like objects instead.
///   correct_cursor: *(not part of API yet, may be removed at any point)*
///     Whether it is required that the cursor is left in the correct position
///     (behind the last processed character) after park_cursor() has been
///     called. If set to False, performance for unseekable streams is
///     drastically improved at the cost of the cursor ending up in places
///     unrelated to the actual tokenization progress. For seekable streams, the
///     improvement shouldn't be noticable.
#[pyclass]
#[pyo3(text_signature = "(stream, *, buffering=-1, strings_as_files=False, correct_cursor=True)")]
pub struct RustTokenizer {
    stream: Py<PyClassBoxedSuitableStream>,
    strings_as_files: bool,
    completed: bool,
    advance: bool,
    token: String,
    state: State,
    next_state: State,
    index: i64,
    c: Option<char>,
    json_string_reader: Option<Py<JsonStringReader>>,
}

fn is_delimiter(c: CharOrEof) -> bool {
    match c {
        Char(c_) => c_.is_whitespace() || "{}[]:,".contains(c_),
        Eof => true,
    }
}

impl IntoPy<PyObject> for TokenType {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self as u32).into_py(py)
    }
}

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("{0}")]
    InvalidJson(String),
    #[error("Error due to limitation: {0}")]
    Limitation(String),
    #[error("Unknown error")]
    Unknown,
}

impl From<ParseFloatError> for ParsingError {
    fn from(e: ParseFloatError) -> ParsingError {
        ParsingError::InvalidJson(format!("error parsing float: {e}"))
    }
}

impl From<UnicodeError> for ParsingError {
    fn from(e: UnicodeError) -> ParsingError {
        ParsingError::InvalidJson(format!("error parsing unicode: {e}"))
    }
}

pub enum JsonStreamingError {
    ParsingError(ParsingError),
    IOError(io::Error),
}

impl JsonStreamingError {
    pub fn to_py_error_at_index(self, index: isize) -> PyErr {
        match self {
            JsonStreamingError::ParsingError(e) => {
                PyValueError::new_err(format!("{e} at index {index}"))
            }
            JsonStreamingError::IOError(e) => {
                PyIOError::new_err(format!("I/O error while parsing (index {index}): {e:?}"))
            }
        }
    }
}

impl From<ParsingError> for JsonStreamingError {
    fn from(e: ParsingError) -> JsonStreamingError {
        JsonStreamingError::ParsingError(e)
    }
}

impl From<io::Error> for JsonStreamingError {
    fn from(e: io::Error) -> JsonStreamingError {
        JsonStreamingError::IOError(e)
    }
}

#[derive(Clone)]
enum Token {
    Operator(String),
    String_, // handled specially to support string streaming
    Integer(AppropriateInt),
    Float(f64),
    Boolean(bool),
    Null,
}

#[pymethods]
impl RustTokenizer {
    #[new]
    #[args("*", buffering = -1, strings_as_files = "false", correct_cursor = "true")]
    fn new(
        stream: PyObject,
        buffering: i64,
        strings_as_files: bool,
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
        Ok(RustTokenizer {
            stream: Py::new(py, stream)?,
            strings_as_files,
            completed: false,
            advance: true,
            token: String::new(),
            state: State::Whitespace,
            next_state: State::Whitespace,
            index: -1,
            c: None,
            json_string_reader: None,
        })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'_>,
    ) -> PyResult<Option<(TokenType, Option<PyObject>)>> {
        // this is just to read a possibly still unread string within JSON to its end (can happen
        // when strings_as_files is used)
        if let Some(json_string_reader) = &slf.json_string_reader {
            let index_delta = {
                let mut borrowed_json_string_reader = json_string_reader.borrow_mut(py);
                let read = borrowed_json_string_reader.read(None, py)?;
                println!("read: '{read}'");
                borrowed_json_string_reader.index
            };
            slf.index += index_delta;
            slf.json_string_reader = None;
        }
        match RustTokenizer::read_next_token(&mut slf, py) {
            Ok(maybe_tok) => Ok(match maybe_tok {
                Some(tok) => Some(RustTokenizer::token_to_py_tuple(slf, tok, py)?),
                None => None,
            }),
            Err(e) => Err({
                let index = slf.index;
                e.to_py_error_at_index(index as isize)
            }),
        }
    }

    /// Rewind the inner Python stream/file to undo readahead buffering.
    ///
    /// Required because reading char-by-char without buffering is
    /// excruciatingly slow (1 Python read() call per char!), but buffering
    /// leaves the cursor in random places that don't correspond to what has
    /// actually been processed.
    ///
    /// This method brings it back to the last char that was actually
    /// processed, so the JSON parser can call it when it sees the end of the
    /// document has been reached and thereby allow reading the stream beyond
    /// it without skipping anything.
    #[pyo3(text_signature = "($self)")]
    fn park_cursor(slf: PyRefMut<'_, Self>, py: Python<'_>) -> PyResult<()> {
        if let Err(e) = slf.stream.borrow_mut(py).park_cursor() {
            return Err(PyValueError::new_err(format!(
                "error rewinding stream to undo readahead: {e}"
            )));
        }
        Ok(())
    }
    /// Bytes/string data that have been buffered but not yet processed.
    ///
    /// The type (bytes or str) depends on the type of the data returned by
    /// the underlying Python file-like object / stream.
    ///
    /// This is provided as an alternative to park_cursor for unseekable yet
    /// buffered (for performance) streams. In such cases, the cursor will be
    /// in a "wrong" position (namely at the end of the block read ahead into
    /// the buffer) even after park_cursor() has been called, so this feature
    /// allows users to write their own workarounds by obtaining the
    /// read-ahead data.
    #[getter]
    fn remainder(slf: PyRefMut<'_, Self>, py: Python<'_>) -> StreamData {
        slf.stream.borrow(py).remainder()
    }
}

impl RustTokenizer {
    fn read_next_token(
        slf: &mut Self,
        py: Python<'_>,
    ) -> Result<Option<Token>, JsonStreamingError> {
        let mut now_token;
        loop {
            if slf.advance {
                match slf.stream.borrow_mut(py).read_char()? {
                    Some(r) => slf.c = Some(r),
                    None => slf.c = None,
                }
                slf.index += 1;
            }
            match slf.c {
                Some(c) => {
                    now_token = RustTokenizer::process_char(slf.borrow_mut(), Char(c))?;
                    slf.state = slf.next_state.clone();
                    if slf.completed {
                        slf.completed = false;
                        slf.token = String::new();
                        return Ok(now_token.clone());
                    }
                }
                None => {
                    slf.advance = false;
                    break;
                }
            }
        }
        now_token = RustTokenizer::process_char(slf.borrow_mut(), Eof)?;
        if slf.completed {
            match now_token {
                Some(now_token) => {
                    // these are just to ensure in the next iteration we'll end
                    // up in the slf.completed = false branch and quit:
                    slf.completed = false;
                    slf.state = State::Whitespace;
                    // final token
                    return Ok(Some(now_token));
                }
                None => {
                    return Ok(None);
                }
            }
        } else {
            return Ok(None);
        }
    }

    fn token_to_py_tuple<'a>(
        mut slf: PyRefMut<'_, Self>,
        tok: Token,
        py: Python<'_>,
    ) -> PyResult<(TokenType, Option<PyObject>)> {
        Ok(match tok {
            Token::Operator(s) => (TokenType::Operator, Some(s.into_py(py))),
            Token::String_ => {
                let json_string_reader = Py::new(
                    py,
                    JsonStringReader::from_existing_py_pyclass_boxed_suitable_stream(
                        slf.stream.clone_ref(py),
                    ),
                )?;
                if slf.strings_as_files {
                    slf.json_string_reader = Some(json_string_reader.clone_ref(py));
                    (TokenType::String_, Some(json_string_reader.into_py(py)))
                } else {
                    let mut borrowed_json_string_reader = json_string_reader.borrow_mut(py);
                    let r = (
                        TokenType::String_,
                        Some(borrowed_json_string_reader.read(None, py)?.into_py(py)),
                    );
                    slf.index += borrowed_json_string_reader.index;
                    r
                }
            }
            Token::Integer(n) => (TokenType::Number, Some(n.into_py(py))),
            Token::Float(f) => (TokenType::Number, Some(f.into_py(py))),
            Token::Boolean(b) => (TokenType::Boolean, Some(b.into_py(py))),
            Token::Null => (TokenType::Null, None),
        })
    }

    fn process_char<'a>(slf: &mut Self, c: CharOrEof) -> Result<Option<Token>, ParsingError> {
        slf.advance = true;
        slf.next_state = slf.state.clone();
        let mut now_token = None;
        let mut add_char = false;

        match slf.state {
            State::Whitespace => match c {
                Char('{') => {
                    slf.completed = true;
                    now_token = Some(Token::Operator("{".to_owned()));
                }
                Char('}') => {
                    slf.completed = true;
                    now_token = Some(Token::Operator("}".to_owned()));
                }
                Char('[') => {
                    slf.completed = true;
                    now_token = Some(Token::Operator("[".to_owned()));
                }
                Char(']') => {
                    slf.completed = true;
                    now_token = Some(Token::Operator("]".to_owned()));
                }
                Char(',') => {
                    slf.completed = true;
                    now_token = Some(Token::Operator(",".to_owned()));
                }
                Char(':') => {
                    slf.completed = true;
                    now_token = Some(Token::Operator(":".to_owned()));
                }
                Char('"') => {
                    slf.next_state = State::StringEnd;
                    slf.completed = true;
                    now_token = Some(Token::String_);
                }
                Char('1'..='9') => {
                    slf.next_state = State::Integer;
                    add_char = true;
                }
                Char('0') => {
                    slf.next_state = State::Integer0;
                    add_char = true;
                }
                Char('-') => {
                    slf.next_state = State::IntegerSign;
                    add_char = true;
                }
                Char('f') => {
                    slf.next_state = State::False1;
                }
                Char('t') => {
                    slf.next_state = State::True1;
                }
                Char('n') => {
                    slf.next_state = State::Null1;
                }
                Char(c_) => {
                    if !c_.is_whitespace() {
                        return Err(ParsingError::InvalidJson(format!(
                            "Invalid JSON character: {c:?}"
                        )));
                    }
                }
                Eof => (),
            },
            State::Integer => match c {
                Char('0'..='9') => {
                    add_char = true;
                }
                Char('.') => {
                    slf.next_state = State::FloatingPoint0;
                    add_char = true;
                }
                Char('e' | 'E') => {
                    slf.next_state = State::IntegerExp0;
                    add_char = true;
                }
                _ if is_delimiter(c) => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    match AppropriateInt::from_str(&slf.token) {
                        Ok(parsed_num) => {
                            now_token = Some(Token::Integer(parsed_num));
                        }
                        Err(ParseIntError::General(e)) => {
                            return Err(ParsingError::InvalidJson(format!(
                                "Could not parse integer: {e}"
                            )));
                        }
                        Err(ParseIntError::TooLargeOrSmall) => {
                            return Err(ParsingError::Limitation(format!(
                                "Incapable of parsing integer due to platform constraint"
                            )));
                        }
                    }
                    slf.advance = false;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "A number must contain only digits.  Got {c:?}"
                    )));
                }
            },
            State::Integer0 => match c {
                Char('.') => {
                    slf.next_state = State::FloatingPoint0;
                    add_char = true;
                }
                Char('e' | 'E') => {
                    slf.next_state = State::IntegerExp0;
                    add_char = true;
                }
                _ if is_delimiter(c) => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token = Some(Token::Integer(AppropriateInt::Normal(0)));
                    slf.advance = false;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "A 0 must be followed by a '.' | a 'e'.  Got {c:?}"
                    )));
                }
            },
            State::IntegerSign => match c {
                Char('0') => {
                    slf.next_state = State::Integer0;
                    add_char = true;
                }
                Char('1'..='9') => {
                    slf.next_state = State::Integer;
                    add_char = true;
                }
                c_ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "A - must be followed by a digit.  Got {c_:?}"
                    )));
                }
            },
            State::IntegerExp0 => match c {
                Char('+' | '-' | '0'..='9') => {
                    slf.next_state = State::IntegerExp;
                    add_char = true;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "An e in a number must be followed by a '+', '-' | digit.  Got {c:?}"
                    )));
                }
            },
            State::IntegerExp => match c {
                Char('0'..='9') => {
                    add_char = true;
                }
                _ if is_delimiter(c) => {
                    slf.completed = true;
                    now_token = Some(Token::Float(slf.token.parse::<f64>()?));
                    slf.next_state = State::Whitespace;
                    slf.advance = false;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "A number exponent must consist only of digits.  Got {c:?}"
                    )));
                }
            },
            State::FloatingPoint => match c {
                Char('0'..='9') => {
                    add_char = true;
                }
                Char('e' | 'E') => {
                    slf.next_state = State::IntegerExp0;
                    add_char = true;
                }
                _ if is_delimiter(c) => {
                    slf.completed = true;
                    now_token = Some(Token::Float(slf.token.parse::<f64>()?));
                    slf.next_state = State::Whitespace;
                    slf.advance = false;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "A number must include only digits"
                    )));
                }
            },
            State::FloatingPoint0 => match c {
                Char('0'..='9') => {
                    slf.next_state = State::FloatingPoint;
                    add_char = true;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "A number with a decimal point must be followed by a fractional part"
                    )));
                }
            },
            State::False1 => match c {
                Char('a') => {
                    slf.next_state = State::False2;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::False2 => match c {
                Char('l') => {
                    slf.next_state = State::False3;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::False3 => match c {
                Char('s') => {
                    slf.next_state = State::False4;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::False4 => match c {
                Char('e') => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token = Some(Token::Boolean(false));
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::True1 => match c {
                Char('r') => {
                    slf.next_state = State::True2;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::True2 => match c {
                Char('u') => {
                    slf.next_state = State::True3;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::True3 => match c {
                Char('e') => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token = Some(Token::Boolean(true));
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::Null1 => match c {
                Char('u') => {
                    slf.next_state = State::Null2;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::Null2 => match c {
                Char('l') => {
                    slf.next_state = State::Null3;
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::Null3 => match c {
                Char('l') => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token = Some(Token::Null);
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::StringEnd => {
                if is_delimiter(c) {
                    slf.advance = false;
                    slf.next_state = State::Whitespace;
                } else {
                    return Err(ParsingError::InvalidJson(format!(
                        "Expected whitespace | an operator after strin.  Got {c:?}"
                    )));
                }
            }
        }

        if add_char {
            if let Char(c_) = c {
                slf.token.push(c_);
            }
        };

        Ok(now_token)
    }
}

/// supports_bigint()
/// --
///
/// Returns whether the current installation supports arbitrary-size integers.
#[pyfunction]
fn supports_bigint() -> PyResult<bool> {
    Ok(int::supports_bigint())
}

#[pymodule]
fn json_stream_rs_tokenizer(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<RustTokenizer>()?;
    m.add_class::<JsonStringReader>()?;
    m.add_wrapped(wrap_pyfunction!(supports_bigint))?;

    Ok(())
}
