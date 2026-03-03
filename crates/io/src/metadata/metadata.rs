
use exif::{In, Reader, Tag, Value};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Unified metadata structure for all supported formats.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Metadata {
    pub capture_date: Option<String>,
    pub iso: Option<u32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<f32>,
    pub focal_length: Option<f32>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub orientation: Option<u16>,
}

impl Metadata {
    /// Read EXIF metadata from a file and return a populated Metadata struct.
    ///
    /// Works for JPEG, TIFF, and most TIFF-based RAW formats (CR2, NEF, RW2, ARW, etc.)
    pub fn read_exif<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut bufreader = BufReader::new(file);

        let exif = Reader::new().read_from_container(&mut bufreader)?;

        let mut metadata = Metadata::default();

        // Capture date
        if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
            metadata.capture_date = Some(field.display_value().with_unit(&exif).to_string());
        }

        // ISO
        if let Some(field) = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY) {
            if let Value::Short(ref vec) = field.value {
                metadata.iso = vec.first().map(|v| *v as u32);
            } else if let Value::Long(ref vec) = field.value {
                metadata.iso = vec.first().copied();
            }
        }

        // Shutter speed
        if let Some(field) = exif.get_field(Tag::ExposureTime, In::PRIMARY) {
            metadata.shutter_speed = Some(field.display_value().with_unit(&exif).to_string());
        }

        // Aperture
        if let Some(field) = exif.get_field(Tag::FNumber, In::PRIMARY) {
            if let Value::Rational(ref vec) = field.value {
                if let Some(r) = vec.first() {
                    metadata.aperture = Some(r.to_f64() as f32);
                }
            }
        }

        // Focal length
        if let Some(field) = exif.get_field(Tag::FocalLength, In::PRIMARY) {
            if let Value::Rational(ref vec) = field.value {
                if let Some(r) = vec.first() {
                    metadata.focal_length = Some(r.to_f64() as f32);
                }
            }
        }

        // Camera make
        if let Some(field) = exif.get_field(Tag::Make, In::PRIMARY) {
            metadata.camera_make = Some(field.display_value().with_unit(&exif).to_string());
        }

        // Camera model
        if let Some(field) = exif.get_field(Tag::Model, In::PRIMARY) {
            metadata.camera_model = Some(field.display_value().with_unit(&exif).to_string());
        }

        // Lens model
        if let Some(field) = exif.get_field(Tag::LensModel, In::PRIMARY) {
            metadata.lens_model = Some(field.display_value().with_unit(&exif).to_string());
        }

        // Orientation
        if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
            if let Value::Short(ref vec) = field.value {
                metadata.orientation = vec.first().copied();
            }
        }

        // GPS Latitude
        let lat = exif.get_field(Tag::GPSLatitude, In::PRIMARY);
        let lat_ref = exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY);

        if let (Some(lat), Some(lat_ref)) = (lat, lat_ref) {
            if let (Value::Rational(vec), Value::Ascii(ascii)) = (&lat.value, &lat_ref.value) {
                if vec.len() == 3 && !ascii.is_empty() {
                    let deg = vec[0].to_f64();
                    let min = vec[1].to_f64();
                    let sec = vec[2].to_f64();

                    let mut value = deg + (min / 60.0) + (sec / 3600.0);

                    let dir = std::str::from_utf8(&ascii[0])?;
                    if dir.trim() == "S" {
                        value = -value;
                    }

                    metadata.gps_latitude = Some(value);
                }
            }
        }

        // GPS Longitude
        let lon = exif.get_field(Tag::GPSLongitude, In::PRIMARY);
        let lon_ref = exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY);

        if let (Some(lon), Some(lon_ref)) = (lon, lon_ref) {
            if let (Value::Rational(vec), Value::Ascii(ascii)) = (&lon.value, &lon_ref.value) {
                if vec.len() == 3 && !ascii.is_empty() {
                    let deg = vec[0].to_f64();
                    let min = vec[1].to_f64();
                    let sec = vec[2].to_f64();

                    let mut value = deg + (min / 60.0) + (sec / 3600.0);

                    let dir = std::str::from_utf8(&ascii[0])?;
                    if dir.trim() == "W" {
                        value = -value;
                    }

                    metadata.gps_longitude = Some(value);
                }
            }
        }

        Ok(metadata)
    }
}
