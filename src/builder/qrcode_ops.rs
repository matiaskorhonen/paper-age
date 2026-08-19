//! Polygon-based QR code renderer
//!
//! The algorithm is inspired by the PolyQR Python library by Kurt Böhm:
//! <https://github.com/KurtBoehm/polyqr>

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use log::{debug, info};
use printpdf::{
    Color, LinePoint, Op, PaintMode, Point, Polygon, PolygonRing, Pt, Rgb, WindingOrder,
};
use qrcode::{types::Color as ModuleColor, types::QrError, EcLevel, QrCode};

use crate::page::PageSize;

// ---------------------------------------------------------------------------
// Grid-corner point and edge types
// ---------------------------------------------------------------------------

/// A corner point on the module grid, in (row, col) coordinates.
///
/// For a module at grid position (r, c), its four corners are:
/// `(r, c)`, `(r, c+1)`, `(r+1, c)`, `(r+1, c+1)`.
type GridPoint = (i32, i32);

/// An undirected edge between two grid-corner points, stored in sorted order.
type Edge = (GridPoint, GridPoint);

/// Return the canonical (sorted) representation of an undirected edge.
fn normalized_edge(p: GridPoint, q: GridPoint) -> Edge {
    if p <= q {
        (p, q)
    } else {
        (q, p)
    }
}

/// Return `true` if three grid points are collinear (share a row or column).
fn collinear(a: GridPoint, b: GridPoint, c: GridPoint) -> bool {
    (a.0 == b.0 && b.0 == c.0) || (a.1 == b.1 && b.1 == c.1)
}

// ---------------------------------------------------------------------------
// Connected-component helpers
// ---------------------------------------------------------------------------

/// Find connected components of an undirected graph given as an adjacency map.
fn boundary_components(adj: &BTreeMap<GridPoint, BTreeSet<GridPoint>>) -> Vec<BTreeSet<GridPoint>> {
    let mut unvisited: BTreeSet<GridPoint> = adj.keys().copied().collect();
    let mut components = Vec::new();

    while let Some(&start) = unvisited.iter().next() {
        unvisited.remove(&start);
        let mut component = BTreeSet::new();
        component.insert(start);
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while let Some(u) = queue.pop_front() {
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    if unvisited.remove(&v) {
                        component.insert(v);
                        queue.push_back(v);
                    }
                }
            }
        }
        components.push(component);
    }

    // Sort by descending size so the outer boundary comes first.
    components.sort_by_key(|c| Reverse(c.len()));
    components
}

// ---------------------------------------------------------------------------
// Polygon extraction
// ---------------------------------------------------------------------------

/// Extract simplified polygon chains for all connected components of dark
/// modules in the QR code.
///
/// Returns a list of connected-component polygon groups. Each group is a list
/// of closed point chains (the first is the outer boundary; subsequent ones
/// are holes). Every chain is a sequence of grid-corner points.
fn extract_polygons(code: &QrCode) -> Vec<Vec<Vec<GridPoint>>> {
    let n = code.width() as i32;
    let mut visited = vec![vec![false; n as usize]; n as usize];
    let mut all_chains: Vec<Vec<Vec<GridPoint>>> = Vec::new();

    for r in 0..n {
        for c in 0..n {
            if code[(c as usize, r as usize)] != ModuleColor::Dark
                || visited[r as usize][c as usize]
            {
                continue;
            }

            // --- Flood-fill this connected component ---
            let mut queue = VecDeque::new();
            queue.push_back((r, c));
            visited[r as usize][c as usize] = true;
            let mut edge_counts: HashMap<Edge, u32> = HashMap::new();

            while let Some((cr, cc)) = queue.pop_front() {
                // The four corners of the module at (cr, cc).
                let p00 = (cr, cc);
                let p01 = (cr, cc + 1);
                let p10 = (cr + 1, cc);
                let p11 = (cr + 1, cc + 1);

                // Count every edge of this module.
                for &(p, q) in &[(p00, p01), (p00, p10), (p01, p11), (p10, p11)] {
                    *edge_counts.entry(normalized_edge(p, q)).or_insert(0) += 1;
                }

                // Enqueue unvisited dark neighbors.
                for &(dr, dc) in &[(-1, 0), (0, -1), (0, 1), (1, 0)] {
                    let nr = cr + dr;
                    let nc = cc + dc;
                    if nr >= 0
                        && nr < n
                        && nc >= 0
                        && nc < n
                        && code[(nc as usize, nr as usize)] == ModuleColor::Dark
                        && !visited[nr as usize][nc as usize]
                    {
                        visited[nr as usize][nc as usize] = true;
                        queue.push_back((nr, nc));
                    }
                }
            }

            // --- Boundary edges (used exactly once) ---
            let boundary_edges: HashSet<Edge> = edge_counts
                .into_iter()
                .filter(|&(_, cnt)| cnt == 1)
                .map(|(e, _)| e)
                .collect();

            // --- Build adjacency list of the boundary graph ---
            let mut adj: BTreeMap<GridPoint, BTreeSet<GridPoint>> = BTreeMap::new();
            for &(p, q) in &boundary_edges {
                adj.entry(p).or_default().insert(q);
                adj.entry(q).or_default().insert(p);
            }

            let components = boundary_components(&adj);

            let mut chains: Vec<Vec<GridPoint>> = Vec::new();
            for component in &components {
                let init = *component.iter().next().unwrap(); // lexicographic min (BTreeSet)

                let mut edges_left: HashSet<Edge> = boundary_edges
                    .iter()
                    .filter(|(p, q)| component.contains(p) || component.contains(q))
                    .copied()
                    .collect();

                // --- Construct the initial cycle ---
                let mut chain = vec![init];
                let mut prec: Option<GridPoint> = None;

                loop {
                    let curr = *chain.last().unwrap();
                    let closing = normalized_edge(curr, init);
                    if edges_left.contains(&closing) && chain.len() > 1 {
                        edges_left.remove(&closing);
                        break;
                    }

                    // Available successors via unused edges.
                    let mut succs: Vec<GridPoint> = adj[&curr]
                        .iter()
                        .filter(|&&v| edges_left.contains(&normalized_edge(curr, v)))
                        .copied()
                        .collect();

                    if let Some(pv) = prec {
                        // Prefer a turn (non-collinear) over going straight.
                        succs.sort_by_key(|&sv| collinear(pv, curr, sv) as u8);
                    }

                    let succ = succs[0];
                    chain.push(succ);
                    edges_left.remove(&normalized_edge(curr, succ));
                    prec = Some(curr);
                }

                // --- Extend cycle if it doesn't cover all vertices ---
                while chain.iter().collect::<HashSet<_>>().len() < component.len() {
                    let mut new_chain = vec![init];
                    prec = None;
                    let mut idx = 1;

                    loop {
                        let curr = *new_chain.last().unwrap();
                        let mut succs: Vec<GridPoint> = adj[&curr]
                            .iter()
                            .filter(|&&v| edges_left.contains(&normalized_edge(curr, v)))
                            .copied()
                            .collect();

                        if succs.is_empty() {
                            if idx == chain.len() {
                                break;
                            }
                            // Follow the previous chain when no unused edge is available.
                            let succ = chain[idx];
                            idx += 1;
                            new_chain.push(succ);
                            prec = Some(curr);
                        } else {
                            if let Some(pv) = prec {
                                succs.sort_by_key(|&sv| collinear(pv, curr, sv) as u8);
                            }
                            let succ = succs[0];
                            edges_left.remove(&normalized_edge(curr, succ));
                            new_chain.push(succ);
                            prec = Some(curr);
                        }
                    }

                    chain = new_chain;
                }

                // --- Remove collinear vertices to simplify the polygon ---
                let mut i = 0;
                while i < chain.len() && chain.len() > 2 {
                    let len = chain.len();
                    let p0 = chain[(i + len - 1) % len];
                    let p1 = chain[i];
                    let p2 = chain[(i + 1) % len];
                    if collinear(p0, p1, p2) {
                        chain.remove(i);
                        // After removal, re-check the same index (the previous
                        // vertex may now be collinear with its new neighbors).
                        i = i.saturating_sub(1);
                    } else {
                        i += 1;
                    }
                }

                chains.push(chain);
            }

            all_chains.push(chains);
        }
    }

    all_chains
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build a QR code and return the `printpdf` drawing operations for it.
///
/// The QR code is positioned and sized for the given `page_size`:
/// - Centered horizontally.
/// - Filling `page_size.qrcode_size()` in the upper half of the page,
///   inset by the standard margin.
///
/// The error correction level is chosen automatically (H → Q → M → L),
/// returning the highest level that fits the data.
pub fn render(text: String, page_size: &PageSize) -> Result<Vec<Op>, QrError> {
    // Error Correction Capability (approx.): H 30% / Q 25% / M 15% / L 7%
    let levels = [EcLevel::H, EcLevel::Q, EcLevel::M, EcLevel::L];

    let mut result: Result<QrCode, QrError> = Err(QrError::DataTooLong);
    for &ec_level in &levels {
        debug!("Trying EC level {:?}", ec_level);
        result = QrCode::with_error_correction_level(text.clone(), ec_level);
        if result.is_ok() {
            break;
        }
    }
    let code = result?;

    info!("QR code EC level: {:?}", code.error_correction_level());
    info!("QR code version: {:?}", code.version());

    // Extract polygon groups
    let polygon_groups = extract_polygons(&code);

    let modules_count = code.width() as u32;

    // --- Coordinate calculations ---
    let desired_pt = page_size.qrcode_size().into_pt().0;
    let module_pt = desired_pt / modules_count as f32;
    let qr_size_pt = module_pt * modules_count as f32;

    let page_width_pt = page_size.dimensions().width.into_pt().0;
    let page_height_pt = page_size.dimensions().height.into_pt().0;
    let margin_pt = page_size.dimensions().margin.into_pt().0;

    // Bottom-left origin of the QR code in PDF coordinates (y-axis up).
    let origin_x = (page_width_pt - qr_size_pt) / 2.0;
    let origin_y = page_height_pt - qr_size_pt - margin_pt * 2.0;

    /// Convert a grid-corner point (row, col) to PDF coordinates.
    fn grid_to_pdf(
        pt: GridPoint,
        origin_x: f32,
        origin_y: f32,
        module_pt: f32,
        modules_count: u32,
    ) -> Point {
        let x = origin_x + pt.1 as f32 * module_pt;
        // In PDF, y-axis points up. Row 0 is at the top of the QR code.
        let y = origin_y + (modules_count as f32 - pt.0 as f32) * module_pt;
        Point { x: Pt(x), y: Pt(y) }
    }

    // --- Build ops ---
    let mut ops = vec![Op::SetFillColor {
        col: Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)),
    }];

    for chains in &polygon_groups {
        // Each connected component becomes a single polygon with multiple
        // rings (outer boundary + holes), using even-odd fill rule.
        let rings: Vec<PolygonRing> = chains
            .iter()
            .map(|chain| {
                let points: Vec<LinePoint> = chain
                    .iter()
                    .map(|&gp| LinePoint {
                        p: grid_to_pdf(gp, origin_x, origin_y, module_pt, modules_count),
                        bezier: false,
                    })
                    .collect();
                PolygonRing { points }
            })
            .collect();

        ops.push(Op::DrawPolygon {
            polygon: Polygon {
                rings,
                mode: PaintMode::Fill,
                winding_order: WindingOrder::EvenOdd,
            },
        });
    }

    Ok(ops)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{PageSize, A4_PAGE};

    #[test]
    fn test_pdf_qrcode_ok() {
        let ops = render(String::from("Some value"), &PageSize::A4).unwrap();
        // First op is SetFillColor, followed by at least one DrawPolygon.
        assert!(ops.len() > 1);
        assert!(matches!(ops[0], Op::SetFillColor { .. }));
        assert!(ops[1..]
            .iter()
            .all(|op| matches!(op, Op::DrawPolygon { .. })));
    }

    #[test]
    fn test_pdf_qrcode_polygon_count() {
        let ops = render(String::from("test"), &PageSize::A4).unwrap();
        // Subtract the leading SetFillColor op to count polygon groups.
        let polygon_count = ops.len() - 1;
        // With polygon merging, there should be far fewer ops than individual
        // module rectangles, but still at least one polygon.
        assert!(
            polygon_count >= 1,
            "Expected ≥1 polygon group, got {polygon_count}"
        );
        // There shouldn't be more polygon groups than there are dark modules
        // (441 for version 1 = 21×21).
        assert!(
            polygon_count <= 441,
            "Expected ≤441 polygon groups for version 1, got {polygon_count}"
        );
    }

    #[test]
    fn test_pdf_qrcode_fewer_ops_than_modules() {
        // The polygon approach should produce significantly fewer ops than
        // one rectangle per dark module.
        let ops = render(String::from("test"), &PageSize::A4).unwrap();
        let polygon_count = ops.len() - 1;
        // A version 1 QR code has ~100+ dark modules but should merge into
        // far fewer polygon groups (typically single digits).
        assert!(
            polygon_count < 50,
            "Expected polygon merging to reduce ops, got {polygon_count}"
        );
    }

    #[test]
    fn test_pdf_qrcode_even_odd_fill() {
        let ops = render(String::from("test"), &PageSize::A4).unwrap();
        // Every DrawPolygon should use EvenOdd winding order.
        for op in &ops[1..] {
            if let Op::DrawPolygon { polygon } = op {
                assert_eq!(polygon.winding_order, WindingOrder::EvenOdd);
                assert_eq!(polygon.mode, PaintMode::Fill);
            }
        }
    }

    #[test]
    fn test_pdf_qrcode_multi_ring_for_finder_pattern() {
        // The three finder patterns are large connected components that
        // have inner holes — they should produce polygons with multiple rings.
        let ops = render(String::from("test"), &PageSize::A4).unwrap();
        let multi_ring = ops[1..].iter().any(|op| {
            if let Op::DrawPolygon { polygon } = op {
                polygon.rings.len() > 1
            } else {
                false
            }
        });
        assert!(
            multi_ring,
            "Expected at least one polygon with multiple rings (finder pattern with hole)"
        );
    }

    #[test]
    fn test_pdf_qrcode_letter() {
        let ops = render(String::from("Some value"), &PageSize::Letter).unwrap();
        assert!(ops.len() > 1);
    }

    #[test]
    fn test_pdf_qrcode_origin_x_centered() {
        // For A4 the QR code should be horizontally centered.
        let ops = render(String::from("hi"), &PageSize::A4).unwrap();
        let page_width_pt = A4_PAGE.width.into_pt().0;
        let desired_pt = PageSize::A4.qrcode_size().into_pt().0;

        let expected_origin_x = (page_width_pt - desired_pt) / 2.0;

        // The first polygon should have a point at the leftmost edge (column 0).
        if let Op::DrawPolygon { polygon } = &ops[1] {
            let x_min: f32 = polygon
                .rings
                .iter()
                .flat_map(|ring| ring.points.iter())
                .map(|lp| lp.p.x.0)
                .fold(f32::INFINITY, f32::min);
            let diff = (x_min - expected_origin_x).abs();
            assert!(diff < 0.5, "Expected x ≈ {expected_origin_x}, got {x_min}");
        } else {
            panic!("Expected a DrawPolygon op");
        }
    }

    #[test]
    fn test_pdf_qrcode_too_large() {
        let result = render(
            String::from(include_str!("../../tests/data/too_large.txt")),
            &PageSize::A4,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err() == QrError::DataTooLong);
    }

    #[test]
    fn test_collinear() {
        assert!(collinear((0, 0), (0, 1), (0, 2)));
        assert!(collinear((0, 0), (1, 0), (2, 0)));
        assert!(!collinear((0, 0), (0, 1), (1, 1)));
        assert!(!collinear((0, 0), (1, 0), (1, 1)));
    }

    #[test]
    fn test_normalized_edge() {
        let e1 = normalized_edge((0, 1), (0, 0));
        let e2 = normalized_edge((0, 0), (0, 1));
        assert_eq!(e1, e2);
        assert_eq!(e1, ((0, 0), (0, 1)));
    }

    #[test]
    fn test_extract_polygons_basic() {
        // Verify that extract_polygons produces non-empty results.
        let code = QrCode::with_error_correction_level("test", EcLevel::L).unwrap();
        let polygons = extract_polygons(&code);
        assert!(
            !polygons.is_empty(),
            "Expected at least one connected component"
        );
        // Every chain should have at least 4 points (a rectangle).
        for group in &polygons {
            for chain in group {
                assert!(
                    chain.len() >= 4,
                    "Expected chain with ≥4 points, got {}",
                    chain.len()
                );
            }
        }
    }
}
