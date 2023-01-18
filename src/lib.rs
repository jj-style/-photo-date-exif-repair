use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[macro_use]
extern crate lazy_static;
use crossbeam::channel::{unbounded, Receiver};

use colored::Colorize;
use regex::Regex;

mod cli;
pub use cli::Args;
mod error;
pub use error::Error;

const DEFAULT_TIME: &str = "00:00:00";

pub fn run<'a>(args: Args) -> Result<(), Error<'a>> {
    let (s, r) = unbounded();

    let pool = threadpool::ThreadPool::new(5);

    for i in 0..5 {
        let rx = r.clone();
        pool.execute(move || work(rx, i, args.dryrun, args.overwrite));
    }

    for file in args.files {
        let path = Path::new(&file);

        let existing_date_in_exif = get_datetime_from_metadata(path);
        if existing_date_in_exif.is_some() && args.ignore_existing_date {
            continue;
        }

        let date = match get_date_from_file(path) {
            Ok(d) => d,
            Err(err) => {
                eprintln!("{}", format!("{}", err).red());
                continue;
            }
        };
        println!(
            "{}",
            format!(
                "[{:?}] extracted date from filename '{}'",
                path.file_name().unwrap(),
                date
            )
            .blue()
        );

        if let Some(exifdate) = existing_date_in_exif {
            println!(
                "{}",
                format!(
                    "[{:?}] date already found in exif {:?}",
                    path.file_name().unwrap(),
                    exifdate
                )
                .yellow()
            );
            //let _delta = (exifdate - date).num_days();
            // TODO - if delta > 1 set date?
            continue;
        }

        if let Err(err) = s.send((path.to_path_buf(), date)) {
            eprintln!(
                "{}",
                format!(
                    "error adding {:?} to queue: {}",
                    path.file_name().unwrap(),
                    err
                )
                .red()
            );
        }
    }
    drop(s);
    pool.join();
    Ok(())
}

fn work(
    r: Receiver<(PathBuf, chrono::DateTime<chrono::Local>)>,
    i: u8,
    dryrun: bool,
    overwrite: bool,
) {
    let msg = match r.recv() {
        Ok(d) => d,
        Err(_err) => {
            return;
        }
    };
    println!(
        "[thread {}] setting date={} for file={:?}",
        i,
        msg.1.to_rfc3339(),
        msg.0.file_name().unwrap()
    );
    let mut cmd = Command::new("exiftool");

    if overwrite {
        cmd.arg("-overwrite_original");
    }

    cmd.arg(format!("-AllDates=\"{}\"", msg.1.to_rfc3339(),))
        .arg(format!("{}", msg.0.canonicalize().unwrap().display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null());

    if dryrun {
        println!("[dryrun] {:?}", cmd);
        return;
    }

    match cmd.status() {
        Ok(_) => println!(
            "{}",
            format!(
                "[thread {}] successfully set date for {}",
                i,
                msg.0.display()
            )
            .green()
        ),
        Err(err) => eprintln!(
            "{}",
            format!(
                "[thread {}] error setting date for {}: {}",
                i,
                msg.0.display(),
                err
            )
            .red()
        ),
    };
}

fn get_date_from_file(path: &Path) -> Result<chrono::DateTime<chrono::Local>, Error> {
    match extract_date_with_regex(path.to_str().unwrap()) {
        Some(date_string) => {
            let good_datetimes = get_date_time_parts(&date_string);

            let date_to_parse = match good_datetimes {
                (Some(date), Some(time)) => format!("{} {}", date, time),
                (Some(date), None) => format!("{} {}", date, DEFAULT_TIME),
                (None, Some(_)) | (None, None) => date_string,
            };

            Ok(dateparser::parse(&date_to_parse)
                .map_err(|err| Error::DateParse {
                    parsing: date_to_parse,
                    filename: path.file_name().unwrap(),
                    reason: err.to_string(),
                })?
                .with_timezone(&chrono::Local))
        }
        None => Err(Error::NoDate(path.file_name().unwrap())),
    }
}

/// Given a datetime string, returns the date and time as constituent parts formatted to normal
/// standards.
fn get_date_time_parts(input: &str) -> (Option<String>, Option<String>) {
    lazy_static! {
        static ref DATE_NOT_SPLIT: Regex = Regex::new(r#"^.*\b(\d{8})\b.*$"#).unwrap();
        static ref TIME_NOT_SPLIT: Regex = Regex::new(r#"^.*\b(\d{6})\b.*$"#).unwrap();
        static ref DATETIME_ALL_IN_ONE: Regex = Regex::new(r#"^.*\b(\d{8})(\d{6}).*\b$"#).unwrap();
        static ref DATETIME_NOT_SEPARATED: Regex =
            Regex::new(r#"^.*(\d{4}-\d{2}-\d{2})-?(\d{2}-?\d{2}-?\d{2}).*$"#).unwrap();
    }
    let input = input.replace('_', "-");

    let split_date = |date: &str| format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8]);
    let split_time = |time: &str| format!("{}:{}:{}", &time[0..2], &time[2..4], &time[4..6]);

    if let Some(cap) = DATETIME_ALL_IN_ONE.captures(&input) {
        return (Some(split_date(&cap[1])), Some(split_time(&cap[2])));
    }

    if let Some(cap) = DATETIME_NOT_SEPARATED.captures(&input) {
        return (
            Some(cap[1].to_string()),
            Some(split_time(&cap[2].to_string().replace('-', ""))),
        );
    }

    let new_date = match DATE_NOT_SPLIT.captures(&input) {
        Some(cap) => {
            let date = &cap[1];
            Some(split_date(date))
        }
        None => None,
    };

    let new_time = match TIME_NOT_SPLIT.captures(&input) {
        Some(cap) => {
            let time = &cap[1];
            Some(split_time(time))
        }
        None => None,
    };

    (new_date, new_time)
}

/// Extracts a possible datetime string from some text
fn extract_date_with_regex(text: &str) -> Option<String> {
    lazy_static! {
        static ref NORMAL_DATE_RE: Regex =
            Regex::new(r#"^.*?(\d{4}[-_]?\d{2}[-_]?\d{2}[-_]?(\d{6}|\d{2}[-_]\d{2}[-_]\d{2})).*$"#)
                .unwrap();
        static ref WHATSAPP_DATE_RE: Regex = Regex::new(r#"^.*(20\d{6})-WA.*$"#).unwrap();
    }

    match (
        NORMAL_DATE_RE.captures(text),
        WHATSAPP_DATE_RE.captures(text),
    ) {
        (Some(normal), None) | (Some(normal), Some(_)) => Some(normal[1].to_string()),
        (None, Some(whatsapp)) => Some(whatsapp[1].to_string()),
        (None, None) => None,
    }
}

fn get_datetime_from_metadata(path: &Path) -> Option<chrono::DateTime<chrono::Local>> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);

    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;

    let datetime_field = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY);
    match datetime_field {
        Some(date) => {
            let datetime_value = date.display_value().to_string();
            Some(
                dateparser::parse(&datetime_value)
                    .ok()?
                    .with_timezone(&chrono::Local),
            )
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_normal_date_regex_extraction_from_filename_happy() {
        // Arrange
        let mut cases: HashMap<&str, Option<String>> = HashMap::new();

        cases.insert(
            "i-am-file-taken-at-2022-05-04-123000-find-me.jpg",
            Some("2022-05-04-123000".to_string()),
        );
        cases.insert(
            "i_am_file_taken_at_2022_05_04_123000_find_me.jpg",
            Some("2022_05_04_123000".to_string()),
        );
        cases.insert(
            "i_am_file_taken_at_20220504123000_find_me.jpg",
            Some("20220504123000".to_string()),
        );
        cases.insert(
            "00100lrPORTRAIT_00100_BURST20210506122850023_COVER.jpg",
            Some("20210506122850".to_string()),
        );

        // Act/Assert
        for (filename, expected) in cases {
            assert_eq!(expected, extract_date_with_regex(filename));
        }
    }

    #[test]
    fn test_normal_date_regex_extraction_from_filename_unhappy() {
        // Arrange
        let mut cases: HashMap<&str, Option<String>> = HashMap::new();

        cases.insert("i-am-file-taken-at-202-05-04-123000-find-me.jpg", None);
        cases.insert("i-am-file-taken-at-2022-5-04-123000-find-me.jpg", None);
        cases.insert("i-am-file-taken-at-2022-05-4-123000-find-me.jpg", None);
        cases.insert("i-am-file-taken-at-2022:05:04:123000-find-me.jpg", None);

        // Act/Assert
        for (filename, expected) in cases {
            assert_eq!(expected, extract_date_with_regex(filename));
        }
    }

    #[test]
    fn test_whatsapp_date_regex_extraction_from_filename_happy() {
        // Arrange
        let mut cases: HashMap<&str, Option<String>> = HashMap::new();

        cases.insert("IMG-20220504-WA0049", Some("20220504".to_string()));

        // Act/Assert
        for (filename, expected) in cases {
            assert_eq!(expected, extract_date_with_regex(filename));
        }
    }

    #[test]
    fn test_whatsapp_date_regex_extraction_from_filename_unhappy() {
        // Arrange
        let mut cases: HashMap<&str, Option<String>> = HashMap::new();

        cases.insert("IMG-20220504-W0049", None);
        cases.insert("IMG123-WA0049", None);
        cases.insert("WA0049-20220504", None);

        // Act/Assert
        for (filename, expected) in cases {
            assert_eq!(expected, extract_date_with_regex(filename));
        }
    }
}
