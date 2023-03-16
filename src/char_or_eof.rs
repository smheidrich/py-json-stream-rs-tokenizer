use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// Either a single character or EOF.
///
/// No different from `Option<char>`, but making it explicit that `None` represents `EOF`
/// specifically (as opposed to just a generic absence of a character) to allow for nicer
/// formatting in error messages and such.
#[derive(Clone, Copy)]
pub enum CharOrEof {
    Char(char),
    Eof,
}

impl From<Option<char>> for CharOrEof {
    fn from(value: Option<char>) -> CharOrEof {
        match value {
            Some(c) => CharOrEof::Char(c),
            None => CharOrEof::Eof,
        }
    }
}

impl From<char> for CharOrEof {
    fn from(value: char) -> CharOrEof {
        CharOrEof::Char(value)
    }
}

impl Debug for CharOrEof {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            CharOrEof::Char(c) => Debug::fmt(c, f),
            CharOrEof::Eof => {
                write!(f, "EOF")
            }
        }
    }
}

impl Display for CharOrEof {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            CharOrEof::Char(c) => Display::fmt(c, f),
            CharOrEof::Eof => Ok(()),
        }
    }
}
