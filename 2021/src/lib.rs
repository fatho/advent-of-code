use std::{io::BufRead, str::FromStr};

pub trait BufReadExt: BufRead {
    fn read_parse<T: FromStr>(&mut self, buffer: &mut String) -> std::io::Result<T> {
        match self.read_parse_or_eof(buffer) {
            Ok(None) => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no parse")),
            Ok(Some(v)) => Ok(v),
            Err(e) => Err(e),
        }
    }

    fn read_parse_or_eof<T: FromStr>(&mut self, buffer: &mut String) -> std::io::Result<Option<T>> {
        buffer.clear();
        if self.read_line(buffer)? == 0 {
            return Ok(None)
        }
        let val = buffer
            .trim_end_matches('\n')
            .parse::<T>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "no parse"))?;
        Ok(Some(val))
    }
}

impl<T: BufRead> BufReadExt for T {}
