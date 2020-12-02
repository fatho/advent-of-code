use std::{error::Error, fs::File, io::{BufRead, BufReader}, path::Path, str::FromStr};

/// Read each line as one entry of the vector, parsing it according to the FromStr instance.
pub fn read_lines<P: AsRef<Path>, T: FromStr>(path: P) -> Result<Vec<T>, Box<dyn Error>>
where
    T::Err: Error + 'static,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut result: Vec<T> = Vec::new();

    for line in reader.lines() {
        let entry = line?.parse::<T>()?;
        result.push(entry);
    }

    Ok(result)
}
