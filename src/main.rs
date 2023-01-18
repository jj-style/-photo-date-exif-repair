use clap::Parser;
use photo_date_exif_repair::{run, Args};

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    run(args)?;
    Ok(())
}
