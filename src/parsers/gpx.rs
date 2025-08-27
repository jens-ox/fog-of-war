use super::Parser;
use flate2::read::GzDecoder;
use geo::Point;
use gpx::Gpx;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use walkdir::WalkDir;

pub struct GpxParser;

impl Parser for GpxParser {
    fn parse(&self, data_dir: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
        println!("Searching for .gpx and .gpx.gz files in {} directory...", data_dir.display());

        // Find all .gpx and .gpx.gz files recursively
        let gpx_files: Vec<_> = WalkDir::new(data_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| {
                if !entry.file_type().is_file() {
                    return false;
                }
                
                let path = entry.path();
                let file_name = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                
                file_name.ends_with(".gpx") || file_name.ends_with(".gpx.gz")
            })
            .collect();

        println!("Found {} GPX files (.gpx and .gpx.gz)", gpx_files.len());

        if gpx_files.is_empty() {
            return Ok(Vec::new());
        }

        println!("Processing {} GPX files in parallel...", gpx_files.len());

        // Process files in parallel using rayon and collect all points
        let all_points: Vec<Point> = gpx_files
            .into_par_iter()
            .progress()
            .filter_map(|entry| {
                let file_path = entry.path();

                match extract_points_from_gpx(file_path) {
                    Ok(points) => Some(points),
                    Err(e) => {
                        println!("✗ Error processing {}: {}", file_path.display(), e);
                        None
                    }
                }
            })
            .flatten()
            .collect();

        println!("✓ Extracted {} total points from GPX files", all_points.len());
        Ok(all_points)
    }

    fn name(&self) -> &'static str {
        "GPX Parser"
    }
}

fn extract_points_from_gpx(file_path: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    
    // Check if the file is gzip compressed
    let is_gzipped = file_path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".gz"))
        .unwrap_or(false);
    
    let gpx: Gpx = if is_gzipped {
        // Decompress gzip file
        let decoder = GzDecoder::new(file);
        let reader = BufReader::new(decoder);
        gpx::read(reader)?
    } else {
        // Read plain GPX file
        let reader = BufReader::new(file);
        gpx::read(reader)?
    };

    let mut points = Vec::new();

    // Extract waypoints
    for waypoint in &gpx.waypoints {
        points.push(waypoint.point());
    }

    // Extract track points
    for track in &gpx.tracks {
        for segment in &track.segments {
            for track_point in &segment.points {
                points.push(track_point.point());
            }
        }
    }

    // Extract route points
    for route in &gpx.routes {
        for route_point in &route.points {
            points.push(route_point.point());
        }
    }

    Ok(points)
}
