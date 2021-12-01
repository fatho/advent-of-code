use std::error::Error;
use std::path::{PathBuf, Path};
use std::io::{self, BufRead};
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(short, long)]
    day: i32,

    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let before = Instant::now();
    match opt.day {
        1 => day1(&opt.input)?,
        _ => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "This day is not implemented yet").into());
        }
    }
    let duration = before.elapsed();
    eprintln!("Took {:.3} ms", duration.as_secs_f64() * 1000.0);
    Ok(())
}

fn day1(input: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = io::BufReader::new(std::fs::File::open(input)?);
    let mut line = String::new();
    let mut previous_reading = None;
    let mut increases = 0;

    while reader.read_line(&mut line)? > 0 {
        let reading = line.trim().parse::<i32>()?;
        if let Some(previous_reading) = previous_reading {
            if reading > previous_reading {
                increases += 1;
            }
        }

        previous_reading = Some(reading);
        line.clear();
    }
    println!("{}", increases);
    Ok(())
}