use chrono::TimeZone;
use image::{ImageFormat, RgbImage};
use photo_date_exif_repair::{run, Args};
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_date_rename_e2e() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let dir = tempdir()?;
    let file_path = dir.path().join("IMG_20210820_133000_image.jpg");
    let file = File::create(&file_path)?;

    let img = RgbImage::new(32, 32);
    img.save_with_format(&file_path, ImageFormat::Jpeg)?;

    // Assert
    //get_date_from_exif_from_file(&file)?;

    // Act
    let result = run(Args {
        files: vec![file_path.to_str().unwrap().to_string()],
        dryrun: false,
        overwrite: true,
    });

    // Assert
    assert!(result.is_ok());
    let got_datetime = get_date_from_exif_from_file(File::open(&file_path)?)?;
    let expected_datetime = chrono::Utc
        .with_ymd_and_hms(2021, 8, 20, 13, 30, 0)
        .unwrap();
    assert_eq!(got_datetime, expected_datetime);

    drop(file);
    dir.close()?;
    Ok(())
}

fn get_date_from_exif_from_file(
    file: File,
) -> Result<chrono::DateTime<chrono::Utc>, Box<dyn std::error::Error>> {
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;
    let datetime_field = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY);
    match datetime_field {
        Some(date) => {
            let datetime_value = date.display_value().to_string();
            Ok(dateparser::parse(&datetime_value)?.with_timezone(&chrono::Utc))
        }
        None => Err(Box::from("no date in exif".to_string())),
    }
}