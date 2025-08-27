mod hashable_point;
mod parsers;

use fgbfile::FgbFile;
use geo::Point;
use hashable_point::sanitize;
use parsers::{Parser, fit::FitParser, google_timeline::GoogleTimelineParser, gpx::GpxParser};
use proj::Proj;
use rayon::prelude::*;
use serde::Serialize;
use std::path::Path;

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
    let data_dir = Path::new(DATA_DIR);

    let parsers: Vec<Box<dyn Parser>> = vec![
        Box::new(GpxParser),
        Box::new(GoogleTimelineParser),
        Box::new(FitParser),
    ];

    let mut all_points = Vec::new();

    for parser in &parsers {
        println!("\n--- Running {} ---", parser.name());
        match parser.parse(data_dir) {
            Ok(mut points) => {
                println!("✓ {} extracted {} points", parser.name(), points.len());
                all_points.append(&mut points);
            }
            Err(e) => {
                println!("✗ {} failed: {}", parser.name(), e);
            }
        }
    }

    println!("\n--- Summary ---");
    println!(
        "Collected {} total points from all parsers",
        all_points.len()
    );

    if all_points.is_empty() {
        println!("No points to process.");
        return Ok(());
    }

    println!("Transforming coordinates...");

    PROJ_METER.with(|proj| {
        proj.project_array(&mut all_points, false)
            .expect("transformation to proper EPSG should work")
    });

    println!("Successfully transformed {} points", all_points.len());

    let (sanitized_points, stats) = sanitize(all_points);
    stats.print();

    println!("\nWriting points to {}...", OUT_PATH);

    write_to_flatgeobuf(&sanitized_points).expect("writing to FGB to work");

    println!(
        "✓ Successfully wrote {} points to {}",
        sanitized_points.len(),
        OUT_PATH
    );

    Ok(())
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
