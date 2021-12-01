use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

#[derive(Debug)]
pub struct ParseError<E> {
    line: usize,
    error: E,
}

impl<E: Error> Error for ParseError<E> {}

impl<E: Display> Display for ParseError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error in line {}: {}", self.line, self.error)
    }
}

/// Read each line as one entry of the vector, parsing it according to the FromStr instance.
pub fn read_lines<P: AsRef<Path>, T: FromStr>(path: P) -> Result<Vec<T>, Box<dyn Error>>
where
    T::Err: Error + 'static,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut result: Vec<T> = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let entry = line?.parse::<T>().map_err(|error| ParseError {
            line: line_number + 1,
            error,
        })?;
        result.push(entry);
    }

    Ok(result)
}
