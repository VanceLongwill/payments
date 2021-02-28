use clap::Clap;
use csv;
use std::error::Error;

pub mod transaction;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Vance Longwill <vancelongwill@gmail.com>")]
struct Opts {
    file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    println!("Hello, world {} !", opts.file);
    // @TODO: add extension validation
    let mut reader = csv::Reader::from_path(opts.file)?;
    for result in reader.deserialize() {
        let record: transaction::Transaction = result?;
        println!("found {:?}", record);
    }
    Ok(())
}
