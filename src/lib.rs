/// Rust port of json-stream's tokenizer.
/// https://github.com/daggaz/json-stream
/// Copyright (c) 2020 Jamie Cockburn
/// json-stream's tokenizer was originally taken from the NAYA project.
/// https://github.com/danielyule/naya
/// Copyright (c) 2019 Daniel Yule
use num_bigint::BigInt;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3_file::PyFileLikeObject;
use std::borrow::BorrowMut;
use std::io::BufRead;
use std::io::BufReader;
use std::str::FromStr;
use utf8_chars::BufReadCharsExt;

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
    Unicode1 = 22,
    Unicode2 = 23,
    Unicode3 = 24,
    Unicode4 = 25,
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
    charcode: u32,
}

fn is_delimiter(c: char) -> bool {
    c.is_whitespace() || "{}[]:,".contains(c)
}

impl IntoPy<PyObject> for TokenType {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self as u32).into_py(py)
    }
}

#[pymethods]
impl RustTokenizer {
    #[new]
    fn new(stream: PyObject) -> PyResult<Self> {
        Ok(RustTokenizer {
            stream: Box::new(BufReader::new(
                PyFileLikeObject::with_requirements(
                    stream, true, false, false,
                )?,
            )),
            completed: false,
            advance: true,
            token: String::new(),
            state: State::Whitespace,
            next_state: State::Whitespace,
            index: -1,
            c: None,
            charcode: 0,
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
                    match RustTokenizer::process_char(slf.borrow_mut(), py, c) {
                        Ok(tok) => {
                            now_token = tok;
                            slf.state = slf.next_state.clone();
                        }
                        Err(e) => {
                            let index = slf.index;
                            return Err(PyValueError::new_err(format!(
                                "Error while parsing at index {index}: {e:?}"
                            )));
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
        let now_token = RustTokenizer::process_char(slf.borrow_mut(), py, ' ')?;
        if slf.completed {
            match now_token {
                Some(now_token) => {
                    return Ok(Some(now_token));
                }
                None => {
                    return Ok(None);
                }
            }
        } else {
            return Err(PyValueError::new_err(format!(
                "Unexpected end of stream"
            )));
        }
    }
}

impl RustTokenizer {
    fn process_char<'a>(
        slf: &mut Self,
        py: Python<'_>,
        c: char,
    ) -> Result<Option<(TokenType, Option<PyObject>)>, PyErr> {
        slf.advance = true;
        slf.next_state = slf.state.clone();
        let mut now_token = None;
        let mut add_char = false;
        let mut c = c;

        match slf.state {
            State::Whitespace => match c {
                '{' => {
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Operator, Some("{".into_py(py))));
                }
                '}' => {
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Operator, Some("}".into_py(py))));
                }
                '[' => {
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Operator, Some("[".into_py(py))));
                }
                ']' => {
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Operator, Some("]".into_py(py))));
                }
                ',' => {
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Operator, Some(",".into_py(py))));
                }
                ':' => {
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Operator, Some(":".into_py(py))));
                }
                '"' => {
                    slf.next_state = State::String_;
                }
                '1'..='9' => {
                    slf.next_state = State::Integer;
                    add_char = true;
                }
                '0' => {
                    slf.next_state = State::Integer0;
                    add_char = true;
                }
                '-' => {
                    slf.next_state = State::IntegerSign;
                    add_char = true;
                }
                'f' => {
                    slf.next_state = State::False1;
                }
                't' => {
                    slf.next_state = State::True1;
                }
                'n' => {
                    slf.next_state = State::Null1;
                }
                _ => {
                    if !c.is_whitespace() {
                        return Err(PyValueError::new_err(format!(
                            "Invalid JSON character: {c:?}"
                        )));
                    }
                }
            },
            State::Integer => match c {
                '0'..='9' => {
                    add_char = true;
                }
                '.' => {
                    slf.next_state = State::FloatingPoint0;
                    add_char = true;
                }
                'e' | 'E' => {
                    slf.next_state = State::IntegerExp0;
                    add_char = true;
                }
                _ if is_delimiter(c) => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token = Some((
                        TokenType::Number,
                        match slf.token.parse::<i64>() {
                            Ok(parsed_num) => {
                                Some(parsed_num.into_py(py))
                            },
                            Err(_) => {
                                Some(
                                    match BigInt::from_str(&slf.token) {
                                        Ok(parsed_num) => parsed_num.into_py(py),
                                        Err(e) => {
                                            return Err(PyValueError::new_err(format!("Error parsing integer: {e:?}")));
                                        }
                                    }
                                )
                            }
                        },
                    ));
                    slf.advance = false;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "A number must contain only digits.  Got '{c:?}'"
                    )));
                }
            },
            State::Integer0 => match c {
                '.' => {
                    slf.next_state = State::FloatingPoint0;
                    add_char = true;
                }
                'e' | 'E' => {
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
                    return Err(PyValueError::new_err(format!(
                        "A 0 must be followed by a '.' | a 'e'.  Got '{c:?}'"
                    )));
                }
            },
            State::IntegerSign => match c {
                '0' => {
                    slf.next_state = State::Integer0;
                    add_char = true;
                }
                '1'..='9' => {
                    slf.next_state = State::Integer;
                    add_char = true;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "A - must be followed by a digit.  Got '{c:?}'"
                    )));
                }
            },
            State::IntegerExp0 => match c {
                '+' | '-' | '0'..='9' => {
                    slf.next_state = State::IntegerExp;
                    add_char = true;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "An e in a number must be followed by a '+', '-' | digit.  Got '{c:?}'"
                    )));
                }
            },
            State::IntegerExp => match c {
                '0'..='9' => {
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
                    return Err(PyValueError::new_err(format!(
                        "A number exponent must consist only of digits.  Got '{c:?}'"
                    )));
                }
            },
            State::FloatingPoint => match c {
                '0'..='9' => {
                    add_char = true;
                }
                'e' | 'E' => {
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
                    return Err(PyValueError::new_err(format!(
                        "A number must include only digits"
                    )));
                }
            },
            State::FloatingPoint0 => match c {
                '0'..='9' => {
                    slf.next_state = State::FloatingPoint;
                    add_char = true;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "A number with a decimal point must be followed by a fractional part"
                    )));
                }
            },
            State::False1 => match c {
                'a' => {
                    slf.next_state = State::False2;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::False2 => match c {
                'l' => {
                    slf.next_state = State::False3;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::False3 => match c {
                's' => {
                    slf.next_state = State::False4;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::False4 => match c {
                'e' => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Boolean, Some(false.into_py(py))));
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::True1 => match c {
                'r' => {
                    slf.next_state = State::True2;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::True2 => match c {
                'u' => {
                    slf.next_state = State::True3;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::True3 => match c {
                'e' => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token =
                        Some((TokenType::Boolean, Some(true.into_py(py))));
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::Null1 => match c {
                'u' => {
                    slf.next_state = State::Null2;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::Null2 => match c {
                'l' => {
                    slf.next_state = State::Null3;
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::Null3 => match c {
                'l' => {
                    slf.next_state = State::Whitespace;
                    slf.completed = true;
                    now_token = Some((TokenType::Null, None));
                }
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid JSON character: '{c:?}'"
                    )));
                }
            },
            State::String_ => match c {
                '\"' => {
                    slf.completed = true;
                    now_token = Some((
                        TokenType::String_,
                        Some(slf.token.clone().into_py(py)),
                    ));
                    slf.next_state = State::StringEnd;
                }
                '\\' => {
                    slf.next_state = State::StringEscape;
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
                    return Err(PyValueError::new_err(format!(
                        "Expected whitespace | an operator after strin.  Got '{c:?}'"
                    )));
                }
            }
            State::StringEscape => {
                slf.next_state = State::String_;
                match c {
                    '\\' | '\"' => {
                        add_char = true;
                    }
                    'b' => {
                        c = 8u8 as char;
                        add_char = true;
                    }
                    'f' => {
                        c = 12u8 as char;
                        add_char = true;
                    }
                    'n' => {
                        c = '\n';
                        add_char = true;
                    }
                    't' => {
                        c = '\t';
                        add_char = true;
                    }
                    'r' => {
                        c = '\r';
                        add_char = true;
                    }
                    '/' => {
                        c = '/';
                        add_char = true;
                    }
                    'u' => {
                        slf.next_state = State::Unicode1;
                        slf.charcode = 0;
                    }
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid string escape: {c:?}"
                        )));
                    }
                }
            }
            State::Unicode1 => {
                match c {
                    '0'..='9' => {
                        slf.charcode = (c as u32 - 48) * 4096;
                    }
                    'a'..='f' => {
                        slf.charcode = (c as u32 - 87) * 4096;
                    }
                    'A'..='F' => {
                        slf.charcode = (c as u32 - 55) * 4096;
                    }
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid character code: {c:?}"
                        )));
                    }
                }
                slf.next_state = State::Unicode2;
            }
            State::Unicode2 => {
                match c {
                    '0'..='9' => {
                        slf.charcode += (c as u32 - 48) * 256;
                    }
                    'a'..='f' => {
                        slf.charcode += (c as u32 - 87) * 256;
                    }
                    'A'..='F' => {
                        slf.charcode += (c as u32 - 55) * 256;
                    }
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid character code: {c:?}"
                        )));
                    }
                }
                slf.next_state = State::Unicode3;
            }
            State::Unicode3 => {
                match c {
                    '0'..='9' => {
                        slf.charcode += (c as u32 - 48) * 16;
                    }
                    'a'..='f' => {
                        slf.charcode += (c as u32 - 87) * 16;
                    }
                    'A'..='F' => {
                        slf.charcode += (c as u32 - 55) * 16;
                    }
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid character code: {c:?}"
                        )));
                    }
                }
                slf.next_state = State::Unicode4;
            }
            State::Unicode4 => {
                match c {
                    '0'..='9' => {
                        slf.charcode += c as u32 - 48;
                    }
                    'a'..='f' => {
                        slf.charcode += c as u32 - 87;
                    }
                    'A'..='F' => {
                        slf.charcode += c as u32 - 55;
                    }
                    _ => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid character code: {c:?}"
                        )));
                    }
                }
                slf.next_state = State::String_;
                match char::from_u32(slf.charcode) {
                    Some(unicode_char) => {
                        c = unicode_char;
                    }
                    None => {
                        let charcode = slf.charcode;
                        return Err(PyValueError::new_err(format!(
                            "No unicode character for code: {charcode}"
                        )));
                    }
                }
                add_char = true;
            }
        }

        if add_char {
            slf.token.push(c);
        };

        Ok(now_token)
    }
}

#[pymodule]
fn json_stream_rs_tokenizer(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<RustTokenizer>()?;
    Ok(())
}
