use std::io;

/// Variant of `BufReadCharsExt` trait that can be used as an interface.
///
/// `BufReadCharsExt` has the issue that it has `BufRead` as its supertrait
/// *and* provides a blanket implementation for it, which together mean that
/// you can't provide your own implementations of its methods, ever, even with
/// newtype hacks or things like that, because to implement `BufReadCharsExt`
/// you must implement `BufRead` and if you implement `BufRead` you can't
/// implement `BufReadCharsExt`'s methods anymore because your implementations
/// will conflict with the blanket ones. So it's totally unsuitable as an
/// "interface-like" trait that other people can use in their function
/// signatures to signify "I want something that can give me individual chars,
/// nothing more".
///
/// This trait here, meanwhile, is explicitly meant to be interface-like.
pub trait Utf8CharSource {
    fn read_char(&mut self) -> io::Result<Option<char>>;
}
