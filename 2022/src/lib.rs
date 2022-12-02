use std::{
    io::{BufRead, BufReader, Read},
    marker::PhantomData,
    str::FromStr,
};

pub mod day1;
pub mod day2;
pub mod day3;
pub mod day4;
pub mod day5;
pub mod day6;
pub mod day7;
pub mod day8;
pub mod day9;

pub mod day10;
pub mod day11;
pub mod day12;
pub mod day13;
pub mod day14;
pub mod day15;
pub mod day16;
pub mod day17;
pub mod day18;
pub mod day19;

pub mod day20;
pub mod day21;
pub mod day22;
pub mod day23;
pub mod day24;
pub mod day25;

pub mod parsers;
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
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/inputs/",
            $day,
            "/input.txt"
        ))
    };
}

#[macro_export]
macro_rules! include_input_env {
    ($day:expr) => {{
        match std::env::var_os("AOC_INPUT_DIR") {
            None => $crate::include_input!($day).as_slice().to_owned(),
            Some(path) => {
                let mut source = std::path::PathBuf::from(path);
                source.push($day);
                source.push("input.txt");
                std::fs::read(&source).unwrap()
            }
        }
    }};
}

// fn foo() {
//     match std::env::var_os("AOC_INPUT_DIR") {
//         None => include_input!("...").to_owned(),
//         Some(path) => {
//             let source = std::path::PathBuf::from(path);
//             source.push("...");
//             source.push("input.txt");
//             std::fs::read(source.as_ref()).unwrap()
//         }
//     }
// }

#[macro_export]
macro_rules! test_day {
    ($day:expr, $name:expr, $part1:expr, $part2:expr) => {
        #[cfg(test)]
        const __TEST_INPUT: &[u8] = $crate::include_input!($name);

        #[test]
        fn test_part1() {
            let output1 = ($day.part1)(__TEST_INPUT).expect("part 1 should work");
            assert_eq!(output1, $part1, "part 1");
        }

        #[test]
        fn test_part2() {
            let output2 = ($day.part2)(&mut __TEST_INPUT.as_ref()).expect("part 2 should work");
            assert_eq!(output2, $part2, "part 2");
        }
    };
}
