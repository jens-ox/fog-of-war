use fgbfile::FgbFile;
use geo::{Geometry as GeoGeometry, Point, Polygon};
use geos::Geometry;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::EPSG_METERS;

#[derive(Serialize)]
pub struct PointGeometry {
    pub geo: Point,
}

#[derive(Serialize)]
pub struct BufferedGeometry {
    pub geo: Polygon,
}

pub fn write_to_flatgeobuf(
    points: &Vec<Point>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let point_geometries: Vec<PointGeometry> = points
        .into_par_iter()
        .map(|p| PointGeometry { geo: p.to_owned() })
        .collect();
    FgbFile::create(output_path)
        .unwrap()
        .epsg(EPSG_METERS)
        .write_features(&point_geometries)
        .expect("file to be written");

    // Generate PMTiles file
    generate_pmtiles_for_points(output_path)?;

    Ok(())
}

pub fn write_buffered_to_flatgeobuf(
    geometries: &Vec<Geometry>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let buffered_geometries: Vec<BufferedGeometry> = geometries
        .into_par_iter()
        .filter_map(|g| {
            // Convert GEOS geometry to geo polygon
            GeoGeometry::try_from(g).ok().and_then(|geo_geom| {
                match geo_geom {
                    GeoGeometry::Polygon(poly) => Some(BufferedGeometry { geo: poly }),
                    _ => None, // Skip non-polygon geometries
                }
            })
        })
        .collect();
    FgbFile::create(output_path)
        .unwrap()
        .epsg(EPSG_METERS)
        .write_features(&buffered_geometries)
        .expect("file to be written");

    // Generate PMTiles file
    generate_pmtiles_for_buffered(output_path)?;

    Ok(())
}

/// Generate PMTiles for point data using tippecanoe
fn generate_pmtiles_for_points(fgb_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Extract filename and change extension to .pmtiles
    let fgb_filename = Path::new(fgb_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid FGB path")?;
    let pmtiles_filename = fgb_filename.replace(".fgb", ".pmtiles");

    // Create ui/public directory if it doesn't exist
    let ui_public_dir = "ui/public";
    fs::create_dir_all(ui_public_dir)?;

    let pmtiles_path = format!("{}/{}", ui_public_dir, pmtiles_filename);

    println!("Generating PMTiles: {}...", pmtiles_path);

    let output = Command::new("tippecanoe")
        .args([
            "-o",
            &pmtiles_path,
            "--projection=EPSG:3857",
            "--force",
            "--cluster-distance=1",
            "-r1",
            fgb_path,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tippecanoe failed: {}", stderr).into());
    }

    println!("✓ Generated PMTiles: {}", pmtiles_path);
    Ok(())
}

/// Generate PMTiles for buffered geometry data using tippecanoe
fn generate_pmtiles_for_buffered(fgb_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Extract filename and change extension to .pmtiles
    let fgb_filename = Path::new(fgb_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid FGB path")?;
    let pmtiles_filename = fgb_filename.replace(".fgb", ".pmtiles");

    // Create ui/public directory if it doesn't exist
    let ui_public_dir = "ui/public";
    fs::create_dir_all(ui_public_dir)?;

    let pmtiles_path = format!("{}/{}", ui_public_dir, pmtiles_filename);

    println!("Generating PMTiles: {}...", pmtiles_path);

    let output = Command::new("tippecanoe")
        .args([
            "-o",
            &pmtiles_path,
            "--projection=EPSG:3857",
            "--force",
            fgb_path,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tippecanoe failed: {}", stderr).into());
    }

    println!("✓ Generated PMTiles: {}", pmtiles_path);
    Ok(())
}
