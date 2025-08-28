use geo::{MultiPoint, Point};
use geos::{BufferParams, BufferParamsBuilder, Geom, Geometry};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

pub fn build_buffered_geometries(
    points: &[Point<f64>],
    radius_m: f64,             // e.g., 50.0
    quad_segs: i32,            // e.g., 8
    chunk_size: usize,         // e.g., 100_000
    simplify_tol: Option<f64>, // e.g., Some(0.5) to reduce vertices a bit
) -> Vec<Geometry> {
    let buf_params: BufferParams = BufferParamsBuilder::default()
        .quadrant_segments(quad_segs)
        .build()
        .expect("Buffer params to be built");

    // Buffer in chunks to keep memory predictable using parallel processing.
    let chunks: Vec<_> = points.chunks(chunk_size).collect();
    let total_chunks = chunks.len();
    println!(
        "Processing {} chunks of {} points each...",
        total_chunks, chunk_size
    );

    let buffered_parts: Vec<Geometry> = chunks
        .into_par_iter()
        .progress()
        .map(|chunk| {
            // MultiPoint -> GEOS
            let mp = MultiPoint::from(chunk.to_vec());
            let g = Geometry::try_from(&mp).expect("geo->geos conversion failed");

            // Buffer this chunk (returns MultiPolygon or Polygon)
            g.buffer_with_params(radius_m, &buf_params)
                .expect("buffer failed")
        })
        .collect();

    println!("Dissolving chunks");

    // Dissolve across chunks.
    let coll =
        Geometry::create_geometry_collection(buffered_parts).expect("geometry collection failed");
    let mut dissolved = coll.unary_union().expect("unary_union failed");

    // Optional light simplification (topology-preserving).
    if let Some(tol) = simplify_tol {
        dissolved = dissolved
            .topology_preserve_simplify(tol)
            .expect("simplify failed");
    }

    // Explode to individual Polygon geometries.
    let polygons = explode_polygons(dissolved);

    // Remove small holes from each polygon
    println!("Removing small holes...");
    let min_hole_area = std::f64::consts::PI * radius_m * radius_m; // Area of circle with given radius
    polygons
        .into_par_iter()
        .progress()
        .map(|poly| remove_small_holes(poly, min_hole_area))
        .collect()
}

/// Extracts all Polygon parts (flattens MultiPolygon/GeometryCollection).
fn explode_polygons(g: Geometry) -> Vec<Geometry> {
    match g.geometry_type() {
        geos::GeometryTypes::Polygon => vec![g],
        geos::GeometryTypes::MultiPolygon | geos::GeometryTypes::GeometryCollection => {
            let n = g.get_num_geometries().expect("get geometries");
            let mut out = Vec::with_capacity(n);
            for i in 1..n {
                let sub: Geometry = g.get_geometry_n(i).expect("geometry to exist").clone();
                match sub.geometry_type() {
                    geos::GeometryTypes::Polygon => out.push(sub),
                    geos::GeometryTypes::MultiPolygon | geos::GeometryTypes::GeometryCollection => {
                        out.extend(explode_polygons(sub));
                    }
                    _ => { /* ignore non-polygonal pieces */ }
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

/// Removes holes from a polygon that have an area smaller than the given threshold.
fn remove_small_holes(polygon: Geometry, min_area: f64) -> Geometry {
    match polygon.geometry_type() {
        geos::GeometryTypes::Polygon => {
            // Get the exterior ring
            let exterior = polygon
                .get_exterior_ring()
                .expect("polygon should have exterior ring")
                .clone();

            // Get all interior rings (holes)
            let num_holes = polygon
                .get_num_interior_rings()
                .expect("should get interior ring count");
            let mut large_holes = Vec::new();

            for i in 0..num_holes {
                if let Ok(hole_ring) = polygon.get_interior_ring_n(i as u32) {
                    // Create a polygon from the hole to calculate its area
                    if let Ok(hole_poly) = Geometry::create_polygon(hole_ring.clone(), vec![]) {
                        let hole_area = hole_poly.area().unwrap_or(0.0);

                        // Keep the hole if it's large enough
                        if hole_area >= min_area {
                            large_holes.push(hole_ring.clone());
                        }
                    }
                }
            }

            // Create new polygon with only large holes
            Geometry::create_polygon(exterior, large_holes).unwrap_or(polygon) // Fall back to original if creation fails
        }
        _ => polygon, // Return unchanged if not a polygon
    }
}
