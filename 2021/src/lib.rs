use std::{
    io::{BufRead, BufReader, Read},
    marker::PhantomData,
    str::FromStr,
};

pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;

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
            }
        }
    }

    pub fn read_line(&mut self) -> Option<&str> {
        self.buffer.clear();
        match self.file.read_line(&mut self.buffer) {
            Ok(n) => {
                if n == 0 {
                    None
                } else {
                    Some(self.buffer.trim_end_matches('\n'))
                }
            }
            Err(err) => {
                self.error = Some(err);
                None
            }
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

impl<'a, R, T> Iterator for ParseIter<'a, R, T>
where
    R: Read,
    T: FromStr,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.parse_line()
    }
}

#[macro_export]
macro_rules! include_input {
    ($day:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/inputs/", $day, "/input.txt"))
    };
}

#[macro_export]
macro_rules! test_day {
    ($day:expr, $name:expr, $part1:expr, $part2:expr) => {
        #[cfg(test)]
        mod test {
            const INPUT: &[u8] = crate::include_input!($name);

            #[test]
            fn part1() {
                let output1 = ($day.part1)(INPUT).expect("part 1 should work");
                assert_eq!(output1, $part1, "part 1");
            }
            #[test]
            fn part2() {
                let output2 = ($day.part2)(&mut INPUT.as_ref()).expect("part 2 should work");
                assert_eq!(output2, $part2, "part 2");
            }
        }
    };
}
