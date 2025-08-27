pub mod fit;
pub mod gpx;
pub mod google_timeline;

use geo::Point;
use std::path::Path;

/// Trait for parsers that extract GPS points from various file formats
pub trait Parser {
    /// Extract GPS points from files in the given directory
    /// Returns a vector of all points found
    fn parse(&self, data_dir: &Path) -> Result<Vec<Point>, Box<dyn std::error::Error>>;
    
    /// Get the name of this parser for logging purposes
    fn name(&self) -> &'static str;
}
