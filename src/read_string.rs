use std::io;

pub trait ReadString {
    fn read_string(&mut self, size: usize) -> io::Result<String>;
}
