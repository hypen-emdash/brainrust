use std::io;

#[derive(Debug)]
pub enum Error {
    ProgramComplete,
    UnmatchedOpenBracket(usize),
    UnmatchedCloseBracket(usize),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
