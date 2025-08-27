use super::Parser;
use geo::Point;
use rayon::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct GoogleTimelineParser;

impl Parser for GoogleTimelineParser {
    fn parse(&self, data_dir: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
        let timeline_path = data_dir.join("location-history.json");

        if !timeline_path.exists() {
            println!("No location-history.json found in {}", data_dir.display());
            return Ok(Vec::new());
        }

        println!(
            "Parsing Google Timeline data from {}...",
            timeline_path.display()
        );

        let file = File::open(&timeline_path)?;
        let reader = BufReader::new(file);

        // Parse as raw JSON values first
        let timeline_entries: Vec<Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse Google Timeline JSON: {}", e))?;

        // Use parallel iterator to extract and parse geo strings
        let points: Result<Vec<Point>, String> = timeline_entries
            .into_par_iter()
            .flat_map(|entry| extract_geo_strings_vec(&entry))
            .map(|geo_str| {
                parse_geo_string(&geo_str)
                    .ok_or_else(|| format!("Failed to parse geo string '{}'", geo_str))
            })
            .collect();

        let points = points.map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        println!(
            "✓ Extracted {} location points from Google Timeline",
            points.len()
        );
        Ok(points)
    }

    fn name(&self) -> &'static str {
        "Google Timeline Parser"
    }
}

/// Recursively extract all geo strings from a JSON value and return them as a Vec
fn extract_geo_strings_vec(value: &Value) -> Vec<String> {
    let mut geo_strings = Vec::new();
    extract_geo_strings_recursive(value, &mut geo_strings);
    geo_strings
}

/// Helper function to recursively extract geo strings
fn extract_geo_strings_recursive(value: &Value, geo_strings: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            if s.starts_with("geo:") {
                geo_strings.push(s.clone());
            }
        }
        Value::Object(map) => {
            for (_, v) in map {
                extract_geo_strings_recursive(v, geo_strings);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                extract_geo_strings_recursive(v, geo_strings);
            }
        }
        _ => {} // Ignore other value types
    }
}

/// Parse a geo string in the format "geo:latitude,longitude" into a Point
/// Returns None if the string is malformed or coordinates are invalid
fn parse_geo_string(geo_str: &str) -> Option<Point> {
    // Check for proper geo: prefix
    if !geo_str.starts_with("geo:") {
        return None;
    }

    let coords = &geo_str[4..]; // Remove "geo:" prefix
    let parts: Vec<&str> = coords.split(',').collect();

    // Must have exactly two parts (lat,lon)
    if parts.len() != 2 {
        return None;
    }

    // Parse latitude and longitude
    let latitude: f64 = parts[0].parse().ok()?;
    let longitude: f64 = parts[1].parse().ok()?;

    Some(Point::new(longitude, latitude))
}
