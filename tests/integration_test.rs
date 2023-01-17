use chrono::TimeZone;
use image::{ImageFormat, RgbImage};
use photo_date_exif_repair::{run, Args};
use std::{fs::File, path::PathBuf};
use tempfile::{tempdir, TempDir};

struct TestData(PathBuf, TempDir);

#[test]
fn test_date_rename_e2e() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let data = create_jpeg_with_filename("IMG_20210820_133000_image.jpg")?;
    
    // Act
    let result = run(Args {
        files: vec![data.0.to_str().unwrap().to_string()],
        dryrun: false,
        overwrite: true,
    });

    // Assert
    assert!(result.is_ok());
    let got_datetime = get_date_from_exif_from_file(File::open(&data.0)?)?;
    let expected_datetime = chrono::Local
        .with_ymd_and_hms(2021, 8, 20, 13, 30, 0)
        .unwrap();
    assert_eq!(got_datetime, expected_datetime);

    data.1.close()?;
    Ok(())
}

fn create_jpeg_with_filename(filename: &str) -> Result<TestData, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join(filename);
    let _ = File::create(&file_path)?;

    let img = RgbImage::new(32, 32);
    img.save_with_format(&file_path, ImageFormat::Jpeg)?;
    Ok(TestData(file_path, dir))
}

fn get_date_from_exif_from_file(
    file: File,
) -> Result<chrono::DateTime<chrono::Local>, Box<dyn std::error::Error>> {
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;
    let datetime_field = exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY);
    match datetime_field {
        Some(date) => {
            let datetime_value = date.display_value().to_string();
            Ok(dateparser::parse(&datetime_value)?.with_timezone(&chrono::Local))
        }
        None => Err(Box::from("no date in exif".to_string())),
    }
}
