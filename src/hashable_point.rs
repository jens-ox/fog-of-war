use geo::Point;
use rayon::prelude::*;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub fn round_to_10_meters(point: Point) -> Point {
    let x = (point.x() / 10.0).round() * 10.0;
    let y = (point.y() / 10.0).round() * 10.0;
    Point::new(x, y)
}

pub fn round_to_1_meter(point: Point) -> Point {
    let x = point.x().round();
    let y = point.y().round();
    Point::new(x, y)
}

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

    let unique_points: HashSet<HashablePoint> =
        points.into_par_iter().map(HashablePoint::from).collect();

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

pub fn sanitize_to_1m_no_dedup(points: Vec<Point>) -> Vec<Point> {
    let original_count = points.len();
    
    if original_count == 0 {
        return points;
    }

    println!(
        "Sanitizing {} points to 1m accuracy (no deduplication)...",
        original_count
    );

    let sanitized_points: Vec<Point> = points
        .into_par_iter()
        .map(round_to_1_meter)
        .collect();

    println!("Final point count: {}", sanitized_points.len());
    
    sanitized_points
}

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
