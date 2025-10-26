use std::char::DecodeUtf16Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnicodeError {
    #[error("{0}")]
    InvalidUnicode(String),
    #[error("Error parsing surrogate pair: {0}")]
    InvalidSurrogatePair(String),
    #[error("Weirdness that should never happen: {0}")]
    Weirdness(String),
    #[error("Unknown error")]
    Unknown,
}

impl From<DecodeUtf16Error> for UnicodeError {
    fn from(e: DecodeUtf16Error) -> UnicodeError {
        UnicodeError::InvalidSurrogatePair(format!("couldn't parse surrogate pair: {e}"))
    }
}

#[inline]
pub fn is_surrogate(codepoint: u16) -> bool {
    (0xD800..=0xDFFF).contains(&codepoint)
}

pub fn decode_surrogate_pair(first_half: u16, second_half: u16) -> Result<char, UnicodeError> {
    match char::decode_utf16(vec![first_half, second_half]).next() {
        Some(result) => result.map_err(UnicodeError::from),
        None => Err(UnicodeError::Weirdness(format!(
            "UTF-16 decoding iterator returned nothing for surrogate pair \
            ({}, {}), not even an error",
            first_half, second_half
        ))),
    }
}
