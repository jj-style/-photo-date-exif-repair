use std::{path::Path, ffi::OsStr};

#[macro_use]
extern crate lazy_static;

mod cli;
pub use cli::Args;
use colored::Colorize;
use regex::Regex;

#[derive(thiserror::Error, Debug)]
pub enum Error<'a> {
    #[error("unable to open file")]
    OpenFile(#[from] std::io::Error),
    #[error("failed to parse date '{parsing}' from {filename:?}: {reason}")]
    DateParse { parsing: String, filename: &'a OsStr, reason: String },
    #[error("no date identified in '{0}'")]
    NoDate(String),
}

pub fn run<'a>(args: Args) -> Result<(), Error<'a>> {
    for file in args.files {
        let path = Path::new(&file);
        match get_date_from_file(&path) {
            Ok(date) => {
                println!("{}", format!("extracted date='{}' from file={:?}", date, path.file_name().unwrap()).blue())
            },
            Err(err) => {
                println!("{}", format!("{}", err.to_string()).yellow())
            },
        }
    }
    Ok(())
}

fn get_date_from_file(path: &Path) -> Result<chrono::DateTime<chrono::Utc>, Error> {
    match extract_date_with_regex(path.to_str().unwrap()) {
        Some(date_string) => {
            let datetime= date_string.replace("_", "-");

            let good_datetimes = get_date_time_parts(&datetime);
            
            let date_to_parse = match good_datetimes {
                (Some(date), Some(time)) => format!("{} {}", date, time),
                (Some(date), None) => format!("{}", date),
                (None, Some(_)) | (None, None) => datetime
            };

            dateparser::parse(&date_to_parse).map_err(|err| Error::DateParse { parsing: date_to_parse, filename: path.file_name().unwrap(), reason: err.to_string() })
        },
        None => Err(Error::NoDate(path.file_name().unwrap().to_str().unwrap().to_string()))
    }
}

fn get_date_time_parts(input: &str) -> (Option<String>, Option<String>) {
    lazy_static! {
        static ref DATE_NOT_SPLIT: Regex = Regex::new(r#"^.*\b(\d{8})\b.*$"#).unwrap();
        static ref TIME_NOT_SPLIT: Regex = Regex::new(r#"^.*\b(\d{6})\b.*$"#).unwrap();
        static ref DATETIME_ALL_IN_ONE: Regex = Regex::new(r#"^.*\b(\d{8})(\d{6}).*\b$"#).unwrap();
        static ref DATETIME_NOT_SEPARATED: Regex = Regex::new(r#"^.*(\d{4}-\d{2}-\d{2})-?(\d{2}-?\d{2}-?\d{2}).*$"#).unwrap();
    }

    let split_date = |date: &str| format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8]);
    let split_time = |time: &str| format!("{}:{}:{}", &time[0..2], &time[2..4], &time[4..6]);

    if let Some(cap) = DATETIME_ALL_IN_ONE.captures(input) {
        return (Some(split_date(&cap[1])), Some(split_time(&cap[2])));
    }

    if let Some(cap) = DATETIME_NOT_SEPARATED.captures(input) {
        return (Some(cap[1].to_string()), Some(split_time(&cap[2].to_string().replace("-", ""))));
    }

    let new_date = match DATE_NOT_SPLIT.captures(input) {
        Some(cap) => {
            let date = &cap[1];
            Some(split_date(date))
        },
        None => None,
    };
    
    let new_time = match TIME_NOT_SPLIT.captures(input) {
        Some(cap) => {
            let time = &cap[1];
            Some(split_time(time))
        },
        None => None,
    };

    (new_date, new_time)
}

fn extract_date_with_regex(text: &str) -> Option<String> {
    lazy_static! {
        static ref NORMAL_DATE_RE: Regex = Regex::new(
            r#"^.*(20\d{2}[-_]?\d{2}[-_]?\d{2}[-_]?(\d{6}|\d{2}[-_]\d{2}[-_]\d{2})).*$"#
        )
        .unwrap();
        static ref WHATSAPP_DATE_RE: Regex = Regex::new(r#"^.*(20\d{6})-WA.*$"#).unwrap();
    }

    match (
        NORMAL_DATE_RE.captures(text),
        WHATSAPP_DATE_RE.captures(text),
    ) {
        (Some(normal), None) | (Some(normal), Some(_)) => {
            Some(normal[1].to_string())
        }
        (None, Some(whatsapp)) => {
            Some(whatsapp[1].to_string())
        }
        (None, None) => None,
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
