use crate::Spanned;

use std::io::{self, Write};

pub struct ErrorReporter<'a, W> {
    pub writer: W,
    pub source: &'a str,
}

pub trait Report<W>: Spanned {
    fn report(&self, reporter: &mut ErrorReporter<'_, W>) -> io::Result<()>;
}

impl<'a, W: Write> ErrorReporter<'a, W> {
    pub fn new(writer: W, input: &'a str) -> Self {
        Self {
            writer,
            source: input,
        }
    }

    fn report(&mut self, err: impl Report<W>) -> Result<(), io::Error> {
        write!(self.writer, "[error]: ")?;
        err.report(&mut self)?;
        write!(self.writer, "{}\n", self.source)?;
        write!(self.writer, "{:space$}^ \n", "", space = err.span().start)
    }

    pub fn exit(&mut self, err: impl Report<W>) -> ! {
        self.report(err).expect("failed to write to stdout");
        std::process::exit(1)
    }
}
