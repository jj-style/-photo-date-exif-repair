use chrono::TimeZone;
use image::{ImageFormat, RgbImage};
use photo_date_exif_repair::{run, Args};
use std::{fs::File, path::PathBuf};
use tempfile::{tempdir, TempDir};

struct TestData {
    file: PathBuf,
    dir: TempDir,
}

struct TestCaseHappy {
    filename: String,
    expected_datetime: chrono::DateTime<chrono::Local>,
}

#[test]
fn test_set_date_from_filename_e2e() -> Result<(), Box<dyn std::error::Error>> {
    let test_cases = vec![
        TestCaseHappy {
            filename: "IMG_20210820_133000_image.jpg".to_string(),
            expected_datetime: chrono::Local
                .with_ymd_and_hms(2021, 8, 20, 13, 30, 0)
                .unwrap(),
        },
        TestCaseHappy {
            filename: "00100lrPORTRAIT_00100_BURST20210506122850023_COVER.jpg".to_string(),
            expected_datetime: chrono::Local
                .with_ymd_and_hms(2021, 5, 6, 12, 28, 50)
                .unwrap(),
        },
    ];

    for case in test_cases {
        // Arrange
        let data = create_jpeg_with_filename(&case.filename)?;

        // Expect error parsing date from exif as there is no metadata on the file
        assert!(get_date_from_exif_from_file(File::open(&data.file)?).is_err());

        // Act
        let result = run(Args {
            files: vec![data.file.to_str().unwrap().to_string()],
            dryrun: false,
            overwrite: true,
            ignore_existing_date: false,
        });

        // Assert
        assert!(result.is_ok());
        let got_datetime = get_date_from_exif_from_file(File::open(&data.file)?)?;
        assert_eq!(got_datetime, case.expected_datetime);

        data.dir.close()?;
    }
    Ok(())
}

fn create_jpeg_with_filename(filename: &str) -> Result<TestData, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let file_path = dir.path().join(filename);
    let _ = File::create(&file_path)?;

    let img = RgbImage::new(32, 32);
    img.save_with_format(&file_path, ImageFormat::Jpeg)?;
    Ok(TestData {
        file: file_path,
        dir,
    })
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
