use std::ops::{Add, Range};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const EOF: Span = Span { start: 0, end: 0 };

    pub fn new(Range { start, end }: Range<usize>) -> Self {
        Self { start, end }
    }

    pub fn range(&self) -> Option<Range<usize>> {
        (*self != Self::EOF).then(|| self.start..self.end)
    }
}

impl Add for Span {
    type Output = Span;

    fn add(self, rhs: Span) -> Self::Output {
        Span::new(self.start..rhs.end)
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}

pub struct WithSpan<T> {
    pub value: T,
    pub span: Span,
}

impl<T> WithSpan<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

impl<T> Spanned for WithSpan<T> {
    fn span(&self) -> Span {
        self.span
    }
}
