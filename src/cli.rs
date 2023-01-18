use clap::Parser;

/// Set exif for media based on the date in their filename.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// just show what changes would be made to files, do not actually make them
    #[arg(short, long, default_value = "false")]
    pub dryrun: bool,

    /// overwrite files when setting their date, or if false, saves a copy of the original.
    #[arg(short, long)]
    pub overwrite: bool,

    /// completely ignore files that already have a date set in their EXIF data
    #[arg(short = 'I', long)]
    pub ignore_existing_date: bool,

    /// files to set date for
    pub files: Vec<String>,
}
