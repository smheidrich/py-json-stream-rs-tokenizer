use std::io;

/// It is an error to do arithmetic on this number.
#[derive(Copy, Clone)]
pub struct OpaqueSeekPos(pub u64);

#[derive(Copy, Clone)]
pub enum OpaqueSeekFrom {
    Start(OpaqueSeekPos),
    #[allow(dead_code)] // to be honest I don't understand why this is dead code if it's public...
    End,
    Current,
}

/// A trait for "opaque" seeks like those encountered in Python's text IO.
///
/// "Opaque" here refers to the positions returned by and given to seek(): You
/// may only seek to positions that were returned by a previous seek() call.
/// You may not interpret such positions or differences between them as
/// signifying anything like bytes, characters, or whatever. Seeking to
/// other position e.g. by adding numbers to one or making one up results in
/// undefined behavior. So don't do that.
pub trait OpaqueSeek {
    fn seek(&mut self, pos: OpaqueSeekFrom) -> io::Result<OpaqueSeekPos>;
}
