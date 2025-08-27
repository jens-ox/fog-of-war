use fgbfile::FgbFile;
use geo::Point;
use gpx::Gpx;
use indicatif::ParallelProgressIterator;
use proj::Proj;
use rayon::prelude::*;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use walkdir::WalkDir;

pub const DATA_DIR: &str = "data";
pub const OUT_PATH: &str = "data/out.fgb";

pub const EPSG_WGS84: i32 = 4326;
pub const EPSG_METERS: i32 = 3857;

#[derive(Serialize)]
pub struct PointGeometry {
    geo: Point,
}

thread_local! {
    // project WGS84 to proper EPSG
    pub static PROJ_METER: Proj = Proj::new_known_crs(format!("EPSG:{}", EPSG_WGS84).as_str(), format!("EPSG:{}", EPSG_METERS).as_str(), None).unwrap();
}

fn main() -> Result<(), ()> {
    println!("Searching for .gpx files in {} directory...", DATA_DIR);

    // Find all .gpx files recursively
    let gpx_files: Vec<_> = WalkDir::new(DATA_DIR)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().map_or(false, |ext| ext == "gpx")
        })
        .collect();

    println!("Found {} .gpx files", gpx_files.len());
    println!("Processing files in parallel...\n");

    // Process files in parallel using rayon and collect all points
    let mut all_points: Vec<Point> = gpx_files
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

    println!(
        "\nCollected {} total points from all files",
        all_points.len()
    );

    if all_points.is_empty() {
        println!("No points to process.");
        return Ok(());
    }

    // Transform coordinates from WGS84 to EPSG:3857
    println!("Transforming coordinates from WGS84 to EPSG:3857...");

    PROJ_METER.with(|proj| {
        proj.project_array(&mut all_points, false)
            .expect("transformation to proper EPSG should work")
    });

    println!("Successfully transformed {} points", all_points.len());

    // Write to FlatGeobuf file
    println!("Writing points to {}...", OUT_PATH);

    write_to_flatgeobuf(&all_points).expect("writing to FGB to work");

    println!(
        "✓ Successfully wrote {} points to {}",
        all_points.len(),
        OUT_PATH
    );

    Ok(())
}

fn extract_points_from_gpx(file_path: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let gpx: Gpx = gpx::read(reader)?;

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

fn write_to_flatgeobuf(points: &Vec<Point>) -> Result<(), Box<dyn std::error::Error>> {
    let point_geometries: Vec<PointGeometry> = points
        .into_par_iter()
        .map(|p| PointGeometry { geo: p.to_owned() })
        .collect();
    FgbFile::create(OUT_PATH)
        .unwrap()
        .epsg(EPSG_METERS)
        .write_features(&point_geometries)
        .expect("file to be written");

    Ok(())
}
