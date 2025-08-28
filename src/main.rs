mod buffer;
mod hashable_point;
mod io;
mod parsers;

use buffer::build_buffered_geometries;
use hashable_point::{sanitize, sanitize_to_1m_no_dedup};
use io::{write_buffered_to_flatgeobuf, write_to_flatgeobuf};
use parsers::{Parser, fit::FitParser, google_timeline::GoogleTimelineParser, gpx::GpxParser};
use proj::Proj;
use std::path::Path;

pub const DATA_DIR: &str = "data";
pub const OUT_PATH: &str = "data/out.fgb";
pub const OUT_PATH_100: &str = "data/out_buffer_100.fgb";
pub const OUT_PATH_1000: &str = "data/out_buffer_1000.fgb";
pub const HEATMAP_PATH: &str = "data/heatmap.fgb";

pub const EPSG_WGS84: i32 = 4326;
pub const EPSG_METERS: i32 = 3857;

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

    // Process heatmap points (1m accuracy, no deduplication)
    println!("\nProcessing heatmap points...");

    // Sanitize to 1m accuracy without deduplication
    let heatmap_sanitized = sanitize_to_1m_no_dedup(all_points);

    println!("Writing heatmap points to {}...", HEATMAP_PATH);
    write_to_flatgeobuf(&heatmap_sanitized, HEATMAP_PATH).expect("writing heatmap to FGB to work");

    println!(
        "✓ Successfully wrote {} heatmap points to {}",
        heatmap_sanitized.len(),
        HEATMAP_PATH
    );

    let (sanitized_points, stats) = sanitize(heatmap_sanitized);
    stats.print();

    println!("\nWriting points to {}...", OUT_PATH);

    write_to_flatgeobuf(&sanitized_points, OUT_PATH).expect("writing to FGB to work");

    println!(
        "✓ Successfully wrote {} points to {}",
        sanitized_points.len(),
        OUT_PATH
    );

    println!("\nBuilding buffered 100m geometries...");
    let buffered_geometries = build_buffered_geometries(
        &sanitized_points,
        50.0,      // 50m radius
        8,         // quadrant segments
        1_000,     // chunk size
        Some(0.5), // simplify tolerance
    );

    println!("Writing buffered geometries to {}...", OUT_PATH_100);
    write_buffered_to_flatgeobuf(&buffered_geometries, OUT_PATH_100)
        .expect("writing buffered geometries to FGB to work");

    println!(
        "✓ Successfully wrote {} buffered geometries to {}",
        buffered_geometries.len(),
        OUT_PATH_100
    );

    println!("\nBuilding buffered 1km geometries...");
    let buffered_geometries = build_buffered_geometries(
        &sanitized_points,
        500.0,     // 500m radius
        8,         // quadrant segments
        1_000,     // chunk size
        Some(0.5), // simplify tolerance
    );

    println!("Writing buffered geometries to {}...", OUT_PATH_1000);
    write_buffered_to_flatgeobuf(&buffered_geometries, OUT_PATH_1000)
        .expect("writing buffered geometries to FGB to work");

    println!(
        "✓ Successfully wrote {} buffered geometries to {}",
        buffered_geometries.len(),
        OUT_PATH_1000
    );

    Ok(())
}
