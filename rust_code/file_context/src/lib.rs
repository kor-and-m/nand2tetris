use std::mem;

#[derive(Debug, PartialEq)]
pub struct FileDataLocation {
    pub from: usize,
    pub size: usize,
}

impl FileDataLocation {
    pub fn new(from: usize, size: usize) -> Self {
        Self { from, size }
    }
}

#[derive(Debug, PartialEq)]
pub struct FileSpan {
    pub line: usize,
    pub symbol: usize,
}

impl FileSpan {
    pub fn new(line: usize, symbol: usize) -> Self {
        Self { line, symbol }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct FileContext<Payload> {
    pub idx: usize,
    pub location: Option<FileDataLocation>,
    pub span: Option<FileSpan>,
    pub payload: Payload,
}

impl<Payload> FileContext<Payload> {
    pub fn new(
        payload: Payload,
        idx: usize,
        location: Option<FileDataLocation>,
        span: Option<FileSpan>,
    ) -> Self {
        Self {
            idx,
            location,
            span,
            payload,
        }
    }
}

impl<Payload: Default> FileContext<Payload> {
    pub fn from_old(old: &mut Self) -> Self {
        let mut context = Self::default();
        mem::swap(old, &mut context);
        context
    }
}
