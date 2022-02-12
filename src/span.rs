use std::ops::Range;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(Range { start, end }: Range<usize>) -> Self {
        Self { start, end }
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}
