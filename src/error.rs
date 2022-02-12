use crate::{Span, Spanned};

use std::fmt;
use std::io::{self, Write};

pub trait Report<W>: Spanned {
    fn report(&self, reporter: &mut Reporter<'_, W>) -> io::Result<()>;
}

pub struct Reporter<'a, W> {
    pub out: W,
    pub source: &'a str,
}

impl<'a, W> Reporter<'a, W>
where
    W: Write,
{
    pub fn new(out: W, source: &'a str) -> Self {
        Self { out, source }
    }

    fn report(&mut self, err: impl Report<W>) -> Result<(), io::Error> {
        write!(self.out, "[error]: ")?;
        err.report(self)?;
        writeln!(self.out, "\n{}", self.source)?;
        write!(self.out, "{:space$}^ \n", "", space = err.span().start)
    }

    pub fn exit(&mut self, err: impl Report<W>) -> ! {
        self.report(err).expect("failed to write to stdout");
        std::process::exit(1)
    }
}

impl<E, W> From<E> for Box<dyn Report<W>>
where
    E: Report<W> + fmt::Debug + 'static,
{
    fn from(err: E) -> Self {
        Box::new(err)
    }
}

impl<W> Spanned for Box<dyn Report<W>> {
    fn span(&self) -> Span {
        (**self).span()
    }
}

impl<W> Report<W> for Box<dyn Report<W>> {
    fn report(&self, reporter: &mut Reporter<'_, W>) -> io::Result<()> {
        (**self).report(reporter)
    }
}
