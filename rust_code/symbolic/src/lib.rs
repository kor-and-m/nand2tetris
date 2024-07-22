pub trait SymbolicElem<'a>: Sized {
    fn write_symbols(&self, buff: &mut [u8]) -> usize;
}
