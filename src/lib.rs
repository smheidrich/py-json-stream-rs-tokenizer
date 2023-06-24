/// Rust port of json-stream's tokenizer.
/// https://github.com/daggaz/json-stream
/// Copyright (c) 2020 Jamie Cockburn
/// json-stream's tokenizer was originally taken from the NAYA project.
/// https://github.com/danielyule/naya
/// Copyright (c) 2019 Daniel Yule
use compact_str::CompactString;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3_file::PyFileLikeObject;
use std::borrow::BorrowMut;
use std::io::BufRead;
use std::io::BufReader;
use std::num::ParseFloatError;
use thiserror::Error;
use utf8_chars::BufReadCharsExt;

mod int;
use crate::int::{AppropriateInt, ParseIntError};
use std::str::FromStr;

mod char_or_eof;
use crate::char_or_eof::CharOrEof;
use CharOrEof::{Char, Eof};

mod unicode_utils;
use crate::unicode_utils::{is_surrogate, decode_surrogate_pair, UnicodeError};

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
    String_ = 9,
    StringEscape = 10,
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
    Unicode = 22,
    UnicodeSurrogateStart = 23,
    UnicodeSurrogateStringEscape = 24,
    UnicodeSurrogate = 25,
}

#[pyclass]
struct RustTokenizer {
    stream: Box<dyn BufRead + Send>,
    completed: bool,
    advance: bool,
    token: String,
    state: State,
    next_state: State,
    index: i64,
    c: Option<char>,
    unicode_buffer: CompactString,
    prev_charcode: Option<u16>, // first half of a Unicode surrogate pair
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

#[pymethods]
impl RustTokenizer {
    #[new]
    fn new(stream: PyObject) -> PyResult<Self> {
        Ok(RustTokenizer {
            stream: Box::new(BufReader::new(PyFileLikeObject::with_requirements(
                stream, true, false, false,
            )?)),
            completed: false,
            advance: true,
            token: String::new(),
            state: State::Whitespace,
            next_state: State::Whitespace,
            index: -1,
            c: None,
            unicode_buffer: CompactString::with_capacity(4),
            prev_charcode: None,
        })
    }
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'_>,
    ) -> PyResult<Option<(TokenType, Option<PyObject>)>> {
        let mut now_token;
        loop {
            if slf.advance {
                match slf.stream.chars().next() {
                    Some(r) => match r {
                        Ok(r) => slf.c = Some(r),
                        Err(e) => {
                            let index = slf.index;
                            return Err(PyIOError::new_err(format!(
                                "I/O error while parsing (index {index}): {e:?}"
                            )));
                        }
                    },
                    None => slf.c = None,
                }
                slf.index += 1;
            }
            match slf.c {
                Some(c) => {
                    match RustTokenizer::process_char(slf.borrow_mut(), py, Char(c)) {
                        Ok(tok) => {
                            now_token = tok;
                            slf.state = slf.next_state.clone();
                        }
                        Err(e) => {
                            let index = slf.index;
                            return Err(PyValueError::new_err(format!("{e} at index {index}")));
                        }
                    }
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
        match RustTokenizer::process_char(slf.borrow_mut(), py, Eof) {
            Ok(tok) => {
                now_token = tok;
            }
            Err(e) => {
                let index = slf.index;
                return Err(PyValueError::new_err(format!("{e} at index {index}")));
            }
        }
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
}

impl RustTokenizer {
    fn process_char<'a>(
        slf: &mut Self,
        py: Python<'_>,
        c: CharOrEof,
    ) -> Result<Option<(TokenType, Option<PyObject>)>, ParsingError> {
        slf.advance = true;
        slf.next_state = slf.state.clone();
        let mut now_token = None;
        let mut add_char = false;
        let mut c = c;

        match slf.state {
            State::Whitespace => match c {
                Char('{') => {
                    slf.completed = true;
                    now_token = Some((TokenType::Operator, Some("{".into_py(py))));
                }
                Char('}') => {
                    slf.completed = true;
                    now_token = Some((TokenType::Operator, Some("}".into_py(py))));
                }
                Char('[') => {
                    slf.completed = true;
                    now_token = Some((TokenType::Operator, Some("[".into_py(py))));
                }
                Char(']') => {
                    slf.completed = true;
                    now_token = Some((TokenType::Operator, Some("]".into_py(py))));
                }
                Char(',') => {
                    slf.completed = true;
                    now_token = Some((TokenType::Operator, Some(",".into_py(py))));
                }
                Char(':') => {
                    slf.completed = true;
                    now_token = Some((TokenType::Operator, Some(":".into_py(py))));
                }
                Char('"') => {
                    slf.next_state = State::String_;
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
                },
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
                            now_token = Some((TokenType::Number, Some(parsed_num.into_py(py))));
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
                    now_token = Some((TokenType::Number, Some(0.into_py(py))));
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
                    now_token = Some((
                        TokenType::Number,
                        Some(slf.token.parse::<f64>()?.into_py(py)),
                    ));
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
                    now_token = Some((
                        TokenType::Number,
                        Some(slf.token.parse::<f64>()?.into_py(py)),
                    ));
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
                    now_token = Some((TokenType::Boolean, Some(false.into_py(py))));
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
                    now_token = Some((TokenType::Boolean, Some(true.into_py(py))));
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
                    now_token = Some((TokenType::Null, None));
                }
                _ => {
                    return Err(ParsingError::InvalidJson(format!(
                        "Invalid JSON character: {c:?}"
                    )));
                }
            },
            State::String_ => match c {
                Char('\"') => {
                    slf.completed = true;
                    now_token = Some((TokenType::String_, Some(slf.token.clone().into_py(py))));
                    slf.next_state = State::StringEnd;
                }
                Char('\\') => {
                    slf.next_state = State::StringEscape;
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
            State::StringEscape => {
                slf.next_state = State::String_;
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
                        slf.next_state = State::Unicode;
                        slf.unicode_buffer = CompactString::with_capacity(4);
                    }
                    _ => {
                        return Err(ParsingError::InvalidJson(format!(
                            "Invalid string escape: {c}"
                        )));
                    }
                }
            }
            State::Unicode => {
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
                            slf.next_state = State::String_;
                        }
                        None if is_surrogate(charcode) => {
                            slf.prev_charcode = Some(charcode);
                            slf.next_state = State::UnicodeSurrogateStart;
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
            State::UnicodeSurrogateStart => {
                match c {
                    Char('\\') => {
                        slf.next_state = State::UnicodeSurrogateStringEscape;
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
                }
            }
            State::UnicodeSurrogateStringEscape => {
                match c {
                    Char('u') => {
                        slf.unicode_buffer = CompactString::with_capacity(4);
                        slf.next_state = State::UnicodeSurrogate;
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
                }
            }
            State::UnicodeSurrogate => {
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
                    c = Char(
                        decode_surrogate_pair(prev_charcode, charcode)
                        .map_err(|_| ParsingError::InvalidJson(format!(
                            "Error decoding UTF-16 surrogate pair \
                            \\u{prev_charcode:x}\\u{charcode:x}"
                        )))?
                    );
                    slf.prev_charcode = None;
                    slf.next_state = State::String_;
                    add_char = true;
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
    m.add_wrapped(wrap_pyfunction!(supports_bigint))?;

    Ok(())
}
