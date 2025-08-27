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

        let timeline_entries: Vec<Value> = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse Google Timeline JSON: {}", e))?;

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

fn extract_geo_strings_vec(value: &Value) -> Vec<String> {
    let mut geo_strings = Vec::new();
    extract_geo_strings_recursive(value, &mut geo_strings);
    geo_strings
}

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
        _ => {} // ignore other value types
    }
}

// geo:[lat],[lon]
fn parse_geo_string(geo_str: &str) -> Option<Point> {
    if !geo_str.starts_with("geo:") {
        return None;
    }

    let coords = &geo_str[4..];
    let parts: Vec<&str> = coords.split(',').collect();

    if parts.len() != 2 {
        return None;
    }

    let latitude: f64 = parts[0].parse().ok()?;
    let longitude: f64 = parts[1].parse().ok()?;

    Some(Point::new(longitude, latitude))
}
