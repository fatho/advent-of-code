use std::{error::Error, fs::File, collections::HashSet, io::{BufRead, BufReader}, path::Path};

fn main() {
    for arg in std::env::args_os().skip(1) {
        let path = Path::new(&arg);
        match process(path) {
            Ok(result) => {
                println!("{}: {}", path.display(), result);
            }
            Err(err) => {
                println!("{}: {}", path.display(), err);
            }
        }
    }
}

fn process(path: &Path) -> Result<i32, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut seen = HashSet::new();

    for line in reader.lines() {
        let num = line?.parse::<i32>()?;
        let complement = 2020 - num;
        if seen.contains(&complement) {
            return Ok(complement * num)
        } else {
            seen.insert(num);
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "No two numbers add up to 2020").into())
}
