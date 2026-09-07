//! A deliberately specified UTF-8 CSV dialect for the film.
//! See DIALECT.md. The input source and application schema live outside Decoder.

use std::fmt;
use std::io::{self, Read};

pub type Record = Vec<String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Start,
    Bare,
    Quoted,
    QuoteSeen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    QuoteInBareField,
    CharacterAfterQuote,
    ExpectedLf,
    UnclosedQuote,
    InvalidUtf8,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvError {
    /// Zero-based source byte position.
    /// InvalidUtf8 points to the field's start.
    pub offset: usize,
    pub kind: ErrorKind,
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at byte {}", self.kind, self.offset)
    }
}
impl std::error::Error for CsvError {}

#[derive(Debug)]
pub struct Decoder {
    state: State,
    field: Vec<u8>,
    row: Record,
    offset: usize,
    field_start: usize,
    started: bool,
    after_cr: bool,
    closed: bool,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            state: State::Start,
            field: Vec::new(),
            row: Vec::new(),
            offset: 0,
            field_start: 0,
            started: false,
            after_cr: false,
            closed: false,
        }
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn field_bytes(&self) -> &[u8] {
        &self.field
    }
    pub fn completed_fields(&self) -> &[String] {
        &self.row
    }
    pub fn awaiting_lf(&self) -> bool {
        self.after_cr
    }

    fn error(&mut self, kind: ErrorKind, offset: usize) -> CsvError {
        self.closed = true;
        CsvError { kind, offset }
    }

    fn end_field(&mut self) -> Result<(), CsvError> {
        let bytes = std::mem::take(&mut self.field);
        let text = String::from_utf8(bytes).map_err(|_| {
            self.error(ErrorKind::InvalidUtf8, self.field_start)
        })?;
        self.row.push(text);
        self.state = State::Start;
        self.field_start = self.offset;
        Ok(())
    }

    fn end_record(&mut self) -> Record {
        self.started = false;
        self.field_start = self.offset;
        std::mem::take(&mut self.row)
    }

    /// Consume one source byte, possibly producing one complete record.
    /// A chunk boundary requires NO call to finish: just keep this Decoder.
    pub fn push(
        &mut self,
        byte: u8,
    ) -> Result<Option<Record>, CsvError> {
        if self.closed {
            return Err(CsvError {
                offset: self.offset,
                kind: ErrorKind::Closed,
            });
        }
        let at = self.offset;
        self.offset += 1;
        if self.after_cr {
            if byte != b'\n' {
                return Err(self.error(ErrorKind::ExpectedLf, at));
            }
            self.after_cr = false;
            return Ok(Some(self.end_record()));
        }
        self.started = true;
        match self.state {
            State::Quoted => {
                if byte == b'"' {
                    self.state = State::QuoteSeen;
                } else {
                    self.field.push(byte);
                }
            }
            State::QuoteSeen if byte == b'"' => {
                self.field.push(b'"');
                self.state = State::Quoted;
            }
            State::Start if byte == b'"' => {
                self.state = State::Quoted
            }
            State::Bare if byte == b'"' => {
                return Err(
                    self.error(ErrorKind::QuoteInBareField, at)
                );
            }
            _ => match byte {
                b',' => self.end_field()?,
                b'\n' => {
                    self.end_field()?;
                    return Ok(Some(self.end_record()));
                }
                b'\r' => {
                    self.end_field()?;
                    self.after_cr = true;
                }
                _ if self.state == State::QuoteSeen => {
                    return Err(self
                        .error(ErrorKind::CharacterAfterQuote, at));
                }
                _ => {
                    self.field.push(byte);
                    self.state = State::Bare;
                }
            },
        }
        Ok(None)
    }

    /// Signal FINAL end of input, once. May yield the last unterminated record.
    pub fn finish(&mut self) -> Result<Option<Record>, CsvError> {
        if self.closed {
            return Err(CsvError {
                offset: self.offset,
                kind: ErrorKind::Closed,
            });
        }
        self.closed = true;
        if self.after_cr {
            return Err(
                self.error(ErrorKind::ExpectedLf, self.offset)
            );
        }
        if self.state == State::Quoted {
            return Err(
                self.error(ErrorKind::UnclosedQuote, self.offset)
            );
        }
        if !self.started {
            return Ok(None);
        }
        self.end_field()?;
        Ok(Some(self.end_record()))
    }
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Csv(CsvError),
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Csv(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Io(e) => e,
            Self::Csv(e) => e,
        })
    }
}

/// A blocking Read adapter. It keeps unread buffer bytes between next() calls.
pub struct Records<R> {
    input: R,
    decoder: Decoder,
    buffer: [u8; 4096],
    pos: usize,
    len: usize,
    done: bool,
}

impl<R: Read> Records<R> {
    pub fn new(input: R) -> Self {
        Self {
            input,
            decoder: Decoder::new(),
            buffer: [0; 4096],
            pos: 0,
            len: 0,
            done: false,
        }
    }
}

impl<R: Read> Iterator for Records<R> {
    type Item = Result<Record, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            if self.pos == self.len {
                match self.input.read(&mut self.buffer) {
                    Ok(0) => {
                        self.done = true;
                        return match self.decoder.finish() {
                            Ok(Some(row)) => Some(Ok(row)),
                            Ok(None) => None,
                            Err(e) => Some(Err(ReadError::Csv(e))),
                        };
                    }
                    Ok(n) => {
                        self.pos = 0;
                        self.len = n;
                    }
                    Err(e)
                        if e.kind() == io::ErrorKind::Interrupted =>
                    {
                        continue;
                    }
                    Err(e) => {
                        self.done = true;
                        return Some(Err(ReadError::Io(e)));
                    }
                }
            }
            let byte = self.buffer[self.pos];
            self.pos += 1;
            match self.decoder.push(byte) {
                Ok(Some(row)) => return Some(Ok(row)),
                Ok(None) => {}
                Err(e) => {
                    self.done = true;
                    return Some(Err(ReadError::Csv(e)));
                }
            }
        }
    }
}

impl<R: Read> std::iter::FusedIterator for Records<R> {}
