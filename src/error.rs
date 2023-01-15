use std::ffi::OsStr;

#[derive(thiserror::Error, Debug)]
pub enum Error<'a> {
    #[error("unable to open file")]
    OpenFile(#[from] std::io::Error),
    #[error("failed to parse date '{parsing}' from {filename:?}: {reason}")]
    DateParse {
        parsing: String,
        filename: &'a OsStr,
        reason: String,
    },
    #[error("no date identified in '{0:?}'")]
    NoDate(&'a OsStr),

    #[error("extracting date from exif: '{0}'")]
    ExtractExifDate(String),
}
