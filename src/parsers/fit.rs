use super::Parser;
use fitparser::{FitDataRecord, Value};
use flate2::read::GzDecoder;
use geo::Point;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

/**
 * Parse .fit.gz files, which I get from Strava for newer activities.
 */
pub struct FitParser;

impl Parser for FitParser {
    fn parse(&self, data_dir: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
        println!(
            "Searching for .fit.gz files in {} directory...",
            data_dir.display()
        );

        let fit_files: Vec<_> = WalkDir::new(data_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                if !entry.file_type().is_file() {
                    return false;
                }

                let path = entry.path();
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");

                file_name.ends_with(".fit.gz")
            })
            .collect();

        println!("Found {} .fit.gz files", fit_files.len());

        if fit_files.is_empty() {
            return Ok(Vec::new());
        }

        println!(
            "Processing {} .fit.gz files in parallel...",
            fit_files.len()
        );

        let all_points: Vec<Point> = fit_files
            .into_par_iter()
            .progress()
            .filter_map(|entry| {
                let file_path = entry.path();

                match extract_points_from_fit_gz(file_path) {
                    Ok(points) => Some(points),
                    Err(e) => {
                        println!("✗ Error processing {}: {}", file_path.display(), e);
                        None
                    }
                }
            })
            .flatten()
            .collect();

        println!(
            "✓ Extracted {} total points from .fit.gz files",
            all_points.len()
        );
        Ok(all_points)
    }

    fn name(&self) -> &'static str {
        "FIT Parser"
    }
}

fn extract_points_from_fit_gz(file_path: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let mut decoder = GzDecoder::new(file);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;

    let fit_file = fitparser::from_bytes(&decompressed_data)?;

    let mut points = Vec::new();

    for record in fit_file.iter() {
        if let Some(point) = extract_coordinates_from_record(record) {
            points.push(point);
        }
    }

    Ok(points)
}

fn extract_coordinates_from_record(record: &FitDataRecord) -> Option<Point> {
    let mut latitude: Option<f64> = None;
    let mut longitude: Option<f64> = None;

    for field in record.fields() {
        match field.name() {
            "position_lat" => {
                if let Some(lat_value) = extract_coordinate_value(field.value()) {
                    latitude = Some(lat_value);
                }
            }
            "position_long" => {
                if let Some(lon_value) = extract_coordinate_value(field.value()) {
                    longitude = Some(lon_value);
                }
            }
            _ => {} // ignore other fields
        }
    }

    if let (Some(lat), Some(lon)) = (latitude, longitude) {
        // convert from semicircles to degrees
        let lat_degrees = lat * (180.0 / 2_147_483_648.0);
        let lon_degrees = lon * (180.0 / 2_147_483_648.0);

        Some(Point::new(lon_degrees, lat_degrees))
    } else {
        None
    }
}

fn extract_coordinate_value(value: &Value) -> Option<f64> {
    match value {
        Value::SInt32(v) => Some(*v as f64),
        Value::UInt32(v) => Some(*v as f64),
        Value::SInt16(v) => Some(*v as f64),
        Value::UInt16(v) => Some(*v as f64),
        Value::SInt8(v) => Some(*v as f64),
        Value::UInt8(v) => Some(*v as f64),
        Value::Float32(v) => Some(*v as f64),
        Value::Float64(v) => Some(*v),
        _ => None,
    }
}
