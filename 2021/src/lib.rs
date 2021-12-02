use std::{
    io::{BufRead, BufReader, Read},
    marker::PhantomData,
    str::FromStr,
};

pub mod day1;
pub mod day2;

pub mod runner;
pub use runner::{aoc_main, Day};

pub struct FileParser<R> {
    file: BufReader<R>,
    buffer: String,
    error: Option<std::io::Error>,
}

impl<R: Read> FileParser<R> {
    pub fn new(file: R) -> Self {
        Self {
            file: BufReader::new(file),
            buffer: String::new(),
            error: None,
        }
    }

    pub fn iter_parse<T>(&mut self) -> ParseIter<R, T> {
        ParseIter {
            parser: self,
            output: PhantomData,
        }
    }

    pub fn parse_line<T: FromStr>(&mut self) -> Option<T> {
        self.buffer.clear();
        match self.file.read_line(&mut self.buffer) {
            Ok(n) => {
                if n == 0 {
                    None
                } else {
                    match self.buffer.trim_end_matches('\n').parse::<T>() {
                        Ok(val) => Some(val),
                        Err(_err) => {
                            self.error = Some(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "no parse",
                            ));
                            None
                        }
                    }
                }
            }
            Err(err) => {
                self.error = Some(err);
                None
            },
        }
    }

    pub fn finish(self) -> std::io::Result<()> {
        match self.error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

pub struct ParseIter<'a, R, T> {
    parser: &'a mut FileParser<R>,
    output: PhantomData<T>,
}

impl<'a, R, T> Iterator for ParseIter<'a, R, T> where R: Read, T: FromStr {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.parse_line()
    }
}

pub trait BufReadExt: BufRead {
    fn read_parse<T: FromStr>(&mut self, buffer: &mut String) -> std::io::Result<T> {
        match self.read_parse_or_eof(buffer) {
            Ok(None) => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "no parse",
            )),
            Ok(Some(v)) => Ok(v),
            Err(e) => Err(e),
        }
    }

    fn read_parse_or_eof<T: FromStr>(&mut self, buffer: &mut String) -> std::io::Result<Option<T>> {
        buffer.clear();
        if self.read_line(buffer)? == 0 {
            return Ok(None);
        }
        let val = buffer
            .trim_end_matches('\n')
            .parse::<T>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "no parse"))?;
        Ok(Some(val))
    }
}

impl<T: BufRead> BufReadExt for T {}
