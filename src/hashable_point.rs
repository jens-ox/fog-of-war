use geo::Point;
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
