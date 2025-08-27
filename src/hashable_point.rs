use geo::Point;
use rayon::prelude::*;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Round coordinates to nearest 10 meters
pub fn round_to_10_meters(point: Point) -> Point {
    let x = (point.x() / 10.0).round() * 10.0;
    let y = (point.y() / 10.0).round() * 10.0;
    Point::new(x, y)
}

/// Wrapper for Point that implements Hash and Eq based on rounded coordinates
#[derive(Clone)]
pub struct HashablePoint {
    x_rounded: i64,
    y_rounded: i64,
    original: Point,
}

impl PartialEq for HashablePoint {
    fn eq(&self, other: &Self) -> bool {
        self.x_rounded == other.x_rounded && self.y_rounded == other.y_rounded
    }
}

impl Eq for HashablePoint {}

impl Hash for HashablePoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x_rounded.hash(state);
        self.y_rounded.hash(state);
    }
}

impl From<Point> for HashablePoint {
    fn from(point: Point) -> Self {
        let rounded_point = round_to_10_meters(point);
        HashablePoint {
            x_rounded: rounded_point.x() as i64,
            y_rounded: rounded_point.y() as i64,
            original: rounded_point,
        }
    }
}

impl From<HashablePoint> for Point {
    fn from(hashable: HashablePoint) -> Self {
        hashable.original
    }
}

/// Sanitize a collection of points by rounding to 10 meters and removing duplicates
/// Returns the sanitized points and statistics about the deduplication
pub fn sanitize(points: Vec<Point>) -> (Vec<Point>, SanitizeStats) {
    let original_count = points.len();

    if original_count == 0 {
        return (
            points,
            SanitizeStats {
                final_count: 0,
                removed_count: 0,
                removal_percentage: 0.0,
            },
        );
    }

    println!(
        "Sanitizing {} points (rounding to 10m and deduplicating)...",
        original_count
    );

    // Convert to hashable points (this automatically rounds and enables deduplication)
    let unique_points: HashSet<HashablePoint> =
        points.into_par_iter().map(HashablePoint::from).collect();

    // Convert back to regular points
    let sanitized_points: Vec<Point> = unique_points.into_iter().map(Point::from).collect();

    let final_count = sanitized_points.len();
    let removed_count = original_count - final_count;
    let removal_percentage = (removed_count as f64 / original_count as f64) * 100.0;

    let stats = SanitizeStats {
        final_count,
        removed_count,
        removal_percentage,
    };

    (sanitized_points, stats)
}

/// Statistics from the sanitization process
#[derive(Debug)]
pub struct SanitizeStats {
    pub final_count: usize,
    pub removed_count: usize,
    pub removal_percentage: f64,
}

impl SanitizeStats {
    pub fn print(&self) {
        println!(
            "Removed {} duplicate points ({:.2}% reduction)",
            self.removed_count, self.removal_percentage
        );
        println!("Final point count: {}", self.final_count);
    }
}
