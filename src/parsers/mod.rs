pub mod fit;
pub mod google_timeline;
pub mod gpx;

use geo::Point;
use std::path::Path;

// extract Vec<Point> from different file types
pub trait Parser {
    fn parse(&self, data_dir: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>>;

    fn name(&self) -> &'static str;
}
