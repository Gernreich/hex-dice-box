//! A hexagonal dice box for 3mm ply: six finger-jointed walls on a
//! finger-jointed floor, a stack of glued insert layers that make seven
//! hexagonal pockets -- six round one in the middle, a polyhedral set's
//! worth -- and a lid held on by 2 × 1mm neodymium disc magnets glued into
//! rastered indents. Rust since 2026-11-08 (hex_dice_box.py until then; the
//! move changed no byte of the sheet or the pages).
//!
//! THE BOX. A regular hexagonal prism, open at the top. The walls meet each
//! other at 120 degrees and the floor at 90, jointed as the four-hole
//! ocarinas' polyhedron.rs joints any convex solid: along an edge the run is
//! cut into cells that alternate between the two plates; in its own cell a
//! plate reaches the edge line, in the other's it stops short by
//!
//!     clear(theta) = T sin(theta)
//!
//! which is T at the floor and 2.6mm between walls, where a V-notch shows
//! outside and the inside corner closes. The floor owns the bottom corners,
//! so the floor edges take an odd count of cells, the floor's at both ends;
//! the wall edges have no owner at either end and take an even count, so all
//! six walls are the same part. The walls' top edges are plain.
//!
//! THE INSERTS. LAYERS hexagons, the box's inside less FIT all round, each
//! cut with the seven pockets, glued in a stack on the floor. Pockets are
//! regular hexagons POCKET across the flats, their flats parallel to the walls,
//! WEB apart and RIM in from the insert's edge. The top layer carries the
//! box's magnet indents, rastered into its top face.
//!
//! THE LID. A plain hexagon the box's outside in size, which rests on the
//! wall tops, with a plug glued under it: the inside less PLUG_FIT all round,
//! pocketed like an insert so the dice get its 3mm of headroom too. The plug
//! drops into the 3mm left above the stack and lands on the top insert; its
//! indents, rastered into the face that ends up underneath, sit over the
//! box's, so each pair of magnets meets face to face.
//!
//! THE MAGNETS. Discs MAG_D across and MAG_T thick in indents INDENT_D across
//! and INDENT_DEPTH deep, in two rings: one at the six outer corners, between
//! two outer pockets and the wall corner, and one at the six junctions where
//! the middle pocket meets two outer ones. --magnets=6 keeps only the corners.
//!
//! Writes cut-files/hex-dice-box.svg (one sheet, every part), index.html and
//! turn.html, and with --page=OUT.html a copy of the page without its
//! document skeleton. --solidcraft=DIR writes the box as a parametric
//! SolidCraft design instead (solidcraft_design.rs). Takes --kerf=MM (default
//! 0.13, the measured figure), --pocket=MM, --layers=N, --magnets=6|12 and
//! --no-write.
#[path = "src/py.rs"]
mod py;
#[path = "src/pyrandom.rs"]
mod pyrandom;
#[path = "src/solidcraft.rs"]
mod solidcraft;
#[path = "src/solidcraft_design.rs"]
mod solidcraft_design;

use py::{g, hypot, mcos as cos, msin as sin, radians, repr, round_dp};

pub type P = (f64, f64);

const T: f64 = 3.0; // ply
const FINGER: f64 = 2.0 * T; // the least cell along an edge, the square flute's 6mm

const WEB: f64 = 4.0; // ply between two pockets
const RIM: f64 = 4.0; // ply between an outer pocket and the insert's edge
const FIT: f64 = 0.15; // insert to wall, each side: glued, a slide fit
const PLUG_FIT: f64 = 0.3; // plug to wall, each side: the lid drops on freely
const NIP: f64 = 0.5; // inserts' and plug's corners cut off, clear of glue

const MAG_D: f64 = 2.0;
const MAG_T: f64 = 1.0;
const INDENT_D: f64 = 2.2; // room for a smear of glue round the disc
const INDENT_DEPTH: f64 = 1.1; // the disc 0.1 below the face, glue under it

const BED: (f64, f64) = (600.0, 305.0); // the laser's work area
const MARGIN: f64 = 5.0;
const GAP: f64 = 4.0;
const MIN_WEB: f64 = 1.0; // thinnest ply left beside an indent

// house inks: blue engraves, orange cuts inside a part, black frees it
const BLUE: &str = "#0000ff";
const ORANGE: &str = "#ff8000";
const BLACK: &str = "#000000";

const HERE: &str = env!("CARGO_MANIFEST_DIR");

/// The settings the command line can change (the Python's globals).
#[derive(Clone)]
pub struct Cfg {
    pub kerf: f64,   // measured; --kerf= sets it
    pub pocket: f64, // pocket across the flats: a 22mm metal d20 lies in it
    pub layers: i64, // insert layers: 18mm, and the plug's 3mm, 21mm deep
    pub magnets: i64,
}

fn s3() -> f64 {
    3f64.sqrt()
}

/// Every size that follows from the settings.
#[derive(Clone, Copy)]
pub struct Z {
    pub s: f64,
    pub a_ins: f64,
    pub a_in: f64,
    pub a_out: f64,
    pub e: f64,
    pub h: f64,
    pub a_plug: f64,
}

fn sizes(c: &Cfg) -> Z {
    let s = c.pocket + WEB; // pocket centre to centre
    let a_ins = s + c.pocket / 2.0 + RIM; // insert apothem
    let a_in = a_ins + FIT; // box inside
    let a_out = a_in + T; // box outside
    let e = 2.0 * a_out / s3(); // outside edge
    let h = T + c.layers as f64 * T + T; // floor, stack, plug
    Z { s, a_ins, a_in, a_out, e, h, a_plug: a_in - PLUG_FIT }
}

// --- 2D ------------------------------------------------------------------------

/// A regular hexagon of apothem a: flats facing 0, 60, ... degrees (plus
/// turn), corners anticlockwise.
fn hexagon(a: f64, c: P, turn: f64) -> Vec<P> {
    let r = 2.0 * a / s3();
    (0..6).map(|k| (c.0 + r * cos(radians(30.0 + 60.0 * k as f64 + turn)), c.1 + r * sin(radians(30.0 + 60.0 * k as f64 + turn)))).collect()
}

fn area2(poly: &[P]) -> f64 {
    let n = poly.len();
    let mut s = 0.0;
    for i in 0..n {
        let ((x0, y0), (x1, y1)) = (poly[i], poly[(i + 1) % n]);
        s += x0 * y1 - x1 * y0;
    }
    s
}

/// Any simple polygon grown by d (shrunk if d < 0): every edge moved out
/// parallel, each corner where its two moved edges cross.
fn offset(poly: &[P], d: f64) -> Vec<P> {
    let s = if area2(poly) > 0.0 { 1.0 } else { -1.0 };
    let n = poly.len();
    let mut lines = vec![];
    for i in 0..n {
        let ((x0, y0), (x1, y1)) = (poly[i], poly[(i + 1) % n]);
        let l = hypot(x1 - x0, y1 - y0);
        let (dx, dy) = ((x1 - x0) / l, (y1 - y0) / l);
        let (nx, ny) = (s * dy, -s * dx); // outward
        lines.push(((x0 + d * nx, y0 + d * ny), (dx, dy)));
    }
    (0..n).map(|i| meet(lines[(i + n - 1) % n], lines[i])).collect()
}

fn meet(l1: (P, P), l2: (P, P)) -> P {
    let ((p, d1), (q, d2)) = (l1, l2);
    let det = d1.0 * -d2.1 + d1.1 * d2.0;
    if det.abs() < 1e-12 {
        // straight on: no corner
        return q;
    }
    let r = ((q.0 - p.0) * -d2.1 + (q.1 - p.1) * d2.0) / det;
    (p.0 + r * d1.0, p.1 + r * d1.1)
}

fn simplify(pts: &[P]) -> Vec<P> {
    let mut out: Vec<P> = vec![];
    for p in pts {
        let p = (round_dp(p.0, 6), round_dp(p.1, 6));
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    if out[0] == out[out.len() - 1] {
        out.pop();
    }
    let mut changed = true;
    while changed {
        changed = false;
        let n = out.len();
        for i in 0..n {
            let (a, b, c) = (out[(i + n - 1) % n], out[i], out[(i + 1) % n]);
            if ((b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)).abs() < 1e-9 {
                out.remove(i);
                changed = true;
                break;
            }
        }
    }
    out
}

/// A cell along an edge: from s0 to s1, at depth inward.
type Cell = (f64, f64, f64);

/// A plate's outline: corners anticlockwise, and for each edge from
/// corners[j] to corners[j+1] its cells. Each edge is walked at its depth
/// inward; a corner is where the last set line of one edge crosses the first
/// of the next.
fn jointed(corners: &[P], runs: &[Vec<Cell>]) -> Vec<P> {
    let k = corners.len();
    let mut rr: Vec<(P, P, P, Vec<Cell>)> = vec![];
    for j in 0..k {
        let (p, q) = (corners[j], corners[(j + 1) % k]);
        let l = hypot(q.0 - p.0, q.1 - p.1);
        let dv = ((q.0 - p.0) / l, (q.1 - p.1) / l);
        let nv = (-dv.1, dv.0); // inward, anticlockwise
        let mut segs = runs[j].clone();
        segs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        rr.push((p, dv, nv, segs));
    }
    let mut pts = vec![];
    for j in 0..k {
        let (p, dv, nv, segs) = &rr[j];
        let at = |s: f64, dp: f64| (p.0 + s * dv.0 + dp * nv.0, p.1 + s * dv.1 + dp * nv.1);
        let line: Vec<P> = segs.iter().flat_map(|&(s0, s1, dp)| [at(s0, dp), at(s1, dp)]).collect();
        let (pp, pd, pn, psegs) = &rr[(j + k - 1) % k];
        let last = psegs[psegs.len() - 1].2;
        let a = (pp.0 + last * pn.0, pp.1 + last * pn.1);
        let b = (p.0 + segs[0].2 * nv.0, p.1 + segs[0].2 * nv.1);
        pts.push(meet((a, *pd), (b, *dv)));
        pts.extend_from_slice(&line[1..line.len() - 1]);
    }
    simplify(&pts)
}

/// n equal cells along L; cell i at depth 0 where mine(i), else d.
fn cells(l: f64, n: i64, mine: impl Fn(i64) -> bool, d: f64) -> Vec<Cell> {
    let w = l / n as f64;
    (0..n).map(|i| (i as f64 * w, (i + 1) as f64 * w, if mine(i) { 0.0 } else { d })).collect()
}

fn clear(theta: f64) -> f64 {
    T * sin(radians(theta))
}

/// Cells along the floor edges (odd: the floor's at both ends) and up the
/// wall edges (even: no owner at either end), none under FINGER.
pub fn counts(z: &Z) -> (i64, i64) {
    let mut nb = (z.e / FINGER + 1e-9) as i64;
    nb -= 1 - nb.rem_euclid(2);
    let mut nv = (z.h / FINGER + 1e-9) as i64;
    nv -= nv.rem_euclid(2);
    (nb, nv)
}

/// Every corner of a convex polygon cut off square to its bisector, d in
/// from the corner.
fn nipped(poly: &[P], d: f64) -> Vec<P> {
    let n = poly.len();
    let (mut sx, mut sy) = (0.0, 0.0);
    for p in poly {
        sx += p.0;
    }
    for p in poly {
        sy += p.1;
    }
    let c = (sx / n as f64, sy / n as f64);
    let mut out = vec![];
    for i in 0..n {
        let p = poly[i];
        let (a, b) = (poly[(i + n - 1) % n], poly[(i + 1) % n]);
        let l = hypot(c.0 - p.0, c.1 - p.1);
        let u = ((c.0 - p.0) / l, (c.1 - p.1) / l);
        for q in [a, b] {
            let e = (q.0 - p.0, q.1 - p.1);
            let t = d / (e.0 * u.0 + e.1 * u.1);
            out.push((p.0 + t * e.0, p.1 + t * e.1));
        }
    }
    out
}

// --- parts ---------------------------------------------------------------------

#[derive(Clone)]
pub struct Part {
    pub name: String,
    pub outline: Vec<P>,
    pub holes: Vec<Vec<P>>, // polygons, cut orange
    pub dots: Vec<P>,       // indents engraved blue
    pub x0: f64,
    pub y0: f64,
    pub w: f64,
    pub h: f64,
}

impl Part {
    fn new(name: &str, outline: Vec<P>, holes: Vec<Vec<P>>, dots: Vec<P>) -> Part {
        let xs: Vec<f64> = outline.iter().map(|p| p.0).collect();
        let ys: Vec<f64> = outline.iter().map(|p| p.1).collect();
        let (x0, x1) = (fmin(&xs), fmax(&xs));
        let (y0, y1) = (fmin(&ys), fmax(&ys));
        Part { name: name.into(), outline, holes, dots, x0, y0, w: x1 - x0, h: y1 - y0 }
    }
}

/// Python's min() and max() of a list: the first of equals.
fn fmin(v: &[f64]) -> f64 {
    v[1..].iter().fold(v[0], |a, &b| if b < a { b } else { a })
}
fn fmax(v: &[f64]) -> f64 {
    v[1..].iter().fold(v[0], |a, &b| if b > a { b } else { a })
}

pub fn pocket_centres(z: &Z) -> Vec<P> {
    let s = z.s;
    let mut cs = vec![(0.0, 0.0)];
    cs.extend((0..6).map(|k| (s * cos(radians(60.0 * k as f64)), s * sin(radians(60.0 * k as f64)))));
    cs
}

fn pockets(z: &Z, c: &Cfg) -> Vec<Vec<P>> {
    pocket_centres(z).into_iter().map(|cc| hexagon(c.pocket / 2.0, cc, 0.0)).collect()
}

fn seg_dist(p: P, a: P, b: P) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let raw = ((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (dx * dx + dy * dy);
    let t = py::max(0.0, py::min(1.0, raw));
    hypot(p.0 - a.0 - t * dx, p.1 - a.1 - t * dy)
}

fn poly_dist(p: P, poly: &[P]) -> f64 {
    let n = poly.len();
    let d: Vec<f64> = (0..n).map(|i| seg_dist(p, poly[(i + n - 1) % n], poly[i])).collect();
    fmin(&d)
}

fn inside(p: P, poly: &[P]) -> bool {
    let n = poly.len();
    let mut c = false;
    for i in 0..n {
        let ((x0, y0), (x1, y1)) = (poly[(i + n - 1) % n], poly[i]);
        if (y0 > p.1) != (y1 > p.1) && p.0 < x0 + (p.1 - y0) * (x1 - x0) / (y1 - y0) {
            c = !c;
        }
    }
    c
}

/// The room round a point: its distance to the nearest pocket or the plug's
/// edge.
fn room(p: P, pk: &[Vec<P>], plug: &[P]) -> f64 {
    let mut d: Vec<f64> = pk.iter().map(|h| poly_dist(p, h)).collect();
    d.push(poly_dist(p, plug));
    fmin(&d)
}

/// The indents' centres, the same in the top insert and the plug. The
/// corner ring: on each line from the middle to a box corner, the point
/// farthest from every pocket and from the plug's edge (the smaller of the
/// two plates). The junction ring: where the middle pocket's corner points
/// between two outer pockets, s/sqrt3 out. Also returns the corner ring's
/// radius, which the SolidCraft design takes as found here.
pub fn magnet_spots(z: &Z, c: &Cfg) -> (Vec<P>, f64) {
    let plug = nipped(&hexagon(z.a_plug, (0.0, 0.0), 0.0), NIP);
    let pk = pockets(z, c);
    let mut spots = vec![];
    let mut found = 0.0;
    for k in 0..6 {
        let a = radians((30 + 60 * k) as f64);
        let m = ((2.0 * z.a_plug / s3() - z.s / s3()) / 0.01) as i64;
        let mut best: Option<(f64, f64)> = None;
        for i in 0..m {
            let r = z.s / s3() + i as f64 * 0.01;
            let pt = (r * cos(a), r * sin(a));
            if pk.iter().any(|h| inside(pt, h)) {
                continue;
            }
            let rm = room(pt, &pk, &plug);
            if best.map_or(true, |(_, b)| rm > b) {
                best = Some((r, rm));
            }
        }
        let r = best.unwrap().0;
        if k == 0 {
            found = r;
        }
        spots.push((r * cos(a), r * sin(a)));
    }
    if c.magnets == 12 {
        let r = z.s / s3();
        spots.extend((0..6).map(|k| (r * cos(radians((30 + 60 * k) as f64)), r * sin(radians((30 + 60 * k) as f64)))));
    }
    (spots, found)
}

pub struct Built {
    pub z: Z,
    pub parts: Vec<Part>,
    pub wall: Vec<P>,
    pub floor: Vec<P>,
    pub spots: Vec<P>,
    pub corner_r: f64,
}

pub fn build(c: &Cfg) -> Built {
    let z = sizes(c);
    let (e, h) = (z.e, z.h);
    let (nb, nv) = counts(&z);
    let (c90, c120) = (clear(90.0), clear(120.0));
    let mut parts = vec![];

    let floor = jointed(&hexagon(z.a_out, (0.0, 0.0), 0.0), &(0..6).map(|_| cells(e, nb, |i| i % 2 == 0, c90)).collect::<Vec<_>>());
    parts.push(Part::new("floor", floor.clone(), vec![], vec![]));

    // the wall seen from outside, u along the bottom, v up; its left side
    // (u = 0) owns the even cells counted from the floor, its right the odd
    let left: Vec<Cell> = cells(h, nv, |i| i % 2 == 0, c120).into_iter().map(|(s0, s1, d)| (h - s1, h - s0, d)).collect();
    let wall = jointed(&[(0.0, 0.0), (e, 0.0), (e, h), (0.0, h)], &[cells(e, nb, |i| i % 2 == 1, c90), cells(h, nv, |i| i % 2 == 1, c120), vec![(0.0, e, 0.0)], left]);
    for k in 0..6 {
        parts.push(Part::new(&format!("wall {}", k + 1), wall.clone(), vec![], vec![]));
    }

    let pk = pockets(&z, c);
    let (spots, corner_r) = magnet_spots(&z, c);
    let ins = nipped(&hexagon(z.a_ins, (0.0, 0.0), 0.0), NIP);
    for k in 0..c.layers {
        let top = k == c.layers - 1;
        let name = format!("insert {}{}", k + 1, if top { " (top)" } else { "" });
        parts.push(Part::new(&name, ins.clone(), pk.clone(), if top { spots.clone() } else { vec![] }));
    }
    parts.push(Part::new("lid", hexagon(z.a_out, (0.0, 0.0), 0.0), vec![], vec![]));
    parts.push(Part::new("lid plug", nipped(&hexagon(z.a_plug, (0.0, 0.0), 0.0), NIP), pk, spots.clone()));
    Built { z, parts, wall, floor, spots, corner_r }
}

// --- 3D check ------------------------------------------------------------------

/// Throw points at the shell and count the plates holding each: none may be
/// in two, and none in the inner 0.3mm of a wall or the floor may be in none
/// (the seal). Points are tested in each plate's own frame. Within 3mm of an
/// inside wall corner the skin is left out: there the cut-back wall touches
/// its neighbour on a line, and inside that line a groove 0.87 by 1.5mm runs
/// up the corner in its cells, which the shellac fills. corner_contact()
/// measures that line.
fn check_shell(z: &Z, wall: &[P], floor: &[P], n: usize, seed: i64) -> (usize, usize) {
    let mut rnd = pyrandom::Mt::new(seed);
    let frames: Vec<(P, P)> = (0..6)
        .map(|k| {
            let a = radians(60.0 * k as f64);
            ((cos(a), sin(a)), (-sin(a), cos(a)))
        })
        .collect();
    let held = |p: (f64, f64, f64)| -> usize {
        let (x, y, zz) = p;
        let mut n = 0;
        if 0.0 <= zz && zz <= T && inside((x, y), floor) {
            n += 1;
        }
        for (nn, t) in &frames {
            let w = z.a_out - (x * nn.0 + y * nn.1);
            if !(0.0 <= w && w <= T) {
                continue;
            }
            let u = (x * t.0 + y * t.1) + z.e / 2.0;
            if inside((u, zz), wall) {
                n += 1;
            }
        }
        n
    };
    let (mut over, mut leaks) = (0, 0);
    let r = 2.0 * z.a_out / s3();
    let corners = hexagon(z.a_in, (0.0, 0.0), 0.0);
    for _ in 0..n {
        let x = rnd.uniform(-r, r);
        let y = rnd.uniform(-r, r);
        let zz = rnd.uniform(0.0, z.h);
        let p = (x, y, zz);
        let ap: Vec<f64> = (0..6).map(|k| p.0 * cos(radians(60.0 * k as f64)) + p.1 * sin(radians(60.0 * k as f64))).collect();
        let apo = fmax(&ap);
        if apo > z.a_out {
            continue;
        }
        let h = held(p);
        if h > 1 {
            over += 1;
        }
        let dc: Vec<f64> = corners.iter().map(|c| hypot(p.0 - c.0, p.1 - c.1)).collect();
        if fmin(&dc) < 3.0 {
            continue;
        }
        let inner_skin = (z.a_in <= apo && apo <= z.a_in + 0.3 && p.2 >= T - 0.3) || (T - 0.3 <= p.2 && p.2 <= T && apo <= z.a_in);
        if inner_skin && h == 0 {
            leaks += 1;
        }
    }
    (over, leaks)
}

/// Where the cut-back wall's end meets its neighbour. In the cut-back wall's
/// frame, along its plane from the edge line and depth in from its outside,
/// the neighbour's inner end corner lies at (T sin phi, T cos phi), phi = 180
/// - theta the turn between the walls; the wall stops clear(theta) along.
/// Returns the gap between the two (0 is a line contact) and the contact's
/// depth.
fn corner_contact(theta: f64) -> (f64, f64) {
    let phi = radians(180.0 - theta);
    (clear(theta) - T * sin(phi), T * cos(phi))
}

// --- sheet ---------------------------------------------------------------------

fn path(pts: &[P]) -> String {
    let p: Vec<String> = pts.iter().map(|(x, y)| format!("{x:.3},{y:.3}")).collect();
    format!("M {} Z", p.join(" L "))
}

/// Shelves across the bed, tallest first.
fn pack(parts: &[Part], kerf: f64) -> Result<(Vec<(Part, f64, f64)>, f64, f64), String> {
    let mut order: Vec<&Part> = parts.iter().collect();
    order.sort_by(|a, b| (-a.h, -a.w).partial_cmp(&(-b.h, -b.w)).unwrap());
    let mut placed = vec![];
    let (mut x, mut y, mut rowh) = (MARGIN, MARGIN, 0.0f64);
    for p in order {
        let (pw, ph) = (p.w + kerf, p.h + kerf);
        if x + pw > BED.0 - MARGIN {
            x = MARGIN;
            y = y + rowh + GAP;
            rowh = 0.0;
        }
        if y + ph > BED.1 - MARGIN {
            return Err(format!("error: {} does not fit on the {}x{} bed.", p.name, g(BED.0), g(BED.1)));
        }
        placed.push((p.clone(), x, y));
        x += pw + GAP;
        rowh = py::max(rowh, ph);
    }
    let w = fmax(&placed.iter().map(|(p, x, _)| x + p.w + kerf).collect::<Vec<_>>()) + MARGIN;
    let h = fmax(&placed.iter().map(|(p, _, y)| y + p.h + kerf).collect::<Vec<_>>()) + MARGIN;
    Ok((placed, w, h))
}

fn svg(placed: &[(Part, f64, f64)], w: f64, h: f64, c: &Cfg) -> String {
    let (mut blue, mut orange, mut black) = (vec![], vec![], vec![]);
    let kerf = c.kerf;
    for (p, sx, sy) in placed {
        // y flipped for the sheet's y-down; the parts are all symmetric
        let (dx, dy) = (sx - p.x0 + kerf / 2.0, sy + p.h + p.y0 + kerf / 2.0);
        let mv = |pts: Vec<P>| -> Vec<P> { pts.into_iter().map(|(x, y)| (x + dx, dy - y)).collect() };
        black.push(path(&mv(offset(&p.outline, kerf / 2.0))));
        for hole in &p.holes {
            orange.push(path(&mv(offset(hole, -kerf / 2.0))));
        }
        for (cx, cy) in &p.dots {
            blue.push(format!("<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{:.3}\"/>", cx + dx, dy - cy, INDENT_D / 2.0));
        }
    }
    let desc = format!(
        "1 user unit = 1mm. Hexagonal dice box, every part on one sheet: floor, six walls, {} insert layers, lid and lid plug. 3mm ply, drawn for a {}mm kerf (--kerf= recuts at another). Engrave first: blue {BLUE} is a fill engrave, the magnet indents, {}mm across and {}mm deep for {} × {}mm discs; set the depth on scrap. Then orange {ORANGE} cuts the pockets and black {BLACK} frees the parts.",
        c.layers,
        g(kerf),
        g(INDENT_D),
        g(INDENT_DEPTH),
        g(MAG_D),
        g(MAG_T)
    );
    let grp = |items: &[String], fill: &str, stroke: Option<&str>| -> String {
        let mut o = format!("  <g fill=\"{fill}\" stroke=\"{}\"", stroke.unwrap_or("none"));
        if stroke.is_some() {
            o.push_str(" stroke-width=\"0.2\"");
        }
        o.push_str(">\n");
        for i in items {
            if i.starts_with('<') {
                o.push_str(&format!("    {i}\n"));
            } else {
                o.push_str(&format!("    <path d=\"{i}\"/>\n"));
            }
        }
        o.push_str("  </g>\n");
        o
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}mm\" height=\"{h:.2}mm\" viewBox=\"0 0 {w:.2} {h:.2}\">\n<title>Hexagonal dice box</title>\n<desc>{desc}</desc>\n{}{}{}</svg>\n",
        grp(&blue, BLUE, None),
        grp(&orange, "none", Some(ORANGE)),
        grp(&black, "none", Some(BLACK))
    )
}

// --- the page ------------------------------------------------------------------

/// A JSON number as json.dumps writes it: an int bare, a float as its repr.
#[derive(Clone, Copy)]
enum Num {
    I(i64),
    F(f64),
}

fn num(n: Num) -> String {
    match n {
        Num::I(i) => i.to_string(),
        Num::F(x) => repr(x),
    }
}

fn pts_json(pts: &[P]) -> String {
    let v: Vec<String> = pts.iter().map(|(x, y)| format!("[{}, {}]", repr(*x), repr(*y))).collect();
    format!("[{}]", v.join(", "))
}

/// Every part placed in the assembly, for the 3D view: outline, holes, dots
/// and a 3x4 matrix taking (u, v, w) to the box's frame, w up from the
/// part's lower face or inward. As json.dumps writes it.
fn model_json(z: &Z, c: &Cfg, parts: &[Part]) -> String {
    use Num::{F, I};
    let mut m: Vec<String> = vec![];
    let by = |name: &str| parts.iter().find(|p| p.name == name).unwrap();
    let mut put = |name: &str, p: &Part, mm: [[Num; 4]; 3], kind: &str| {
        let holes: Vec<String> = p.holes.iter().map(|h| pts_json(h)).collect();
        let rows: Vec<String> = mm.iter().map(|r| format!("[{}]", r.iter().map(|&x| num(x)).collect::<Vec<_>>().join(", "))).collect();
        m.push(format!(
            "{{\"name\": \"{name}\", \"kind\": \"{kind}\", \"outline\": {}, \"holes\": [{}], \"dots\": {}, \"m\": [{}]}}",
            pts_json(&p.outline),
            holes.join(", "),
            pts_json(&p.dots),
            rows.join(", ")
        ));
    };
    put("floor", by("floor"), [[I(1), I(0), I(0), I(0)], [I(0), I(1), I(0), I(0)], [I(0), I(0), I(1), I(0)]], "box");
    for k in 0..6 {
        let a = radians(60.0 * k as f64);
        let (n, t) = ((cos(a), sin(a)), (-sin(a), cos(a)));
        let p0 = (z.a_out * n.0 - z.e / 2.0 * t.0, z.a_out * n.1 - z.e / 2.0 * t.1);
        let name = format!("wall {}", k + 1);
        put(&name, by(&name), [[F(t.0), I(0), F(-n.0), F(p0.0)], [F(t.1), I(0), F(-n.1), F(p0.1)], [I(0), I(1), I(0), I(0)]], "box");
    }
    for k in 0..c.layers {
        let pre = format!("insert {}", k + 1);
        let name = parts.iter().find(|p| p.name.starts_with(&pre)).unwrap().name.clone();
        put(&name, by(&name), [[I(1), I(0), I(0), I(0)], [I(0), I(1), I(0), I(0)], [I(0), I(0), I(1), F(T * (k + 1) as f64)]], "insert");
    }
    let top = z.h;
    put("lid plug", by("lid plug"), [[I(1), I(0), I(0), I(0)], [I(0), I(1), I(0), I(0)], [I(0), I(0), I(1), F(top - T)]], "lid");
    put("lid", by("lid"), [[I(1), I(0), I(0), I(0)], [I(0), I(1), I(0), I(0)], [I(0), I(0), I(1), F(top)]], "lid");
    format!("[{}]", m.join(", "))
}

fn data_json(b: &Built, c: &Cfg) -> String {
    let z = &b.z;
    format!(
        "{{\"model\": {}, \"T\": {}, \"sizes\": {{\"s\": {}, \"a_ins\": {}, \"a_in\": {}, \"a_out\": {}, \"E\": {}, \"H\": {}, \"a_plug\": {}}}, \"pocket\": {}, \"layers\": {}, \"magnets\": {}, \"kerf\": {}, \"indent\": {}, \"depth\": {}}}",
        model_json(z, c, &b.parts),
        repr(T),
        repr(z.s),
        repr(z.a_ins),
        repr(z.a_in),
        repr(z.a_out),
        repr(z.e),
        repr(z.h),
        repr(z.a_plug),
        repr(c.pocket),
        c.layers,
        b.spots.len(),
        repr(c.kerf),
        repr(INDENT_D),
        repr(INDENT_DEPTH)
    )
}

// --- main ----------------------------------------------------------------------

fn write(path: &str, s: &str) -> Result<(), String> {
    std::fs::write(path, s).map_err(|e| format!("{path}: {e}"))
}

fn run() -> Result<i32, String> {
    let mut c = Cfg { kerf: 0.13, pocket: 25.0, layers: 6, magnets: 12 };
    let (mut write_out, mut page, mut sc) = (true, None::<String>, None::<String>);
    for x in std::env::args().skip(1) {
        let bad = || format!("error: {x} is not a number.");
        if let Some(v) = x.strip_prefix("--kerf=") {
            c.kerf = py::float(v).ok_or_else(bad)?;
            if !(0.0 <= c.kerf && c.kerf <= 0.5) {
                return Err(format!("error: a {}mm kerf is outside 0 to 0.5mm.", g(c.kerf)));
            }
        } else if let Some(v) = x.strip_prefix("--pocket=") {
            c.pocket = py::float(v).ok_or_else(bad)?;
        } else if let Some(v) = x.strip_prefix("--layers=") {
            c.layers = v.trim().parse().map_err(|_| bad())?;
        } else if let Some(v) = x.strip_prefix("--magnets=") {
            c.magnets = v.trim().parse().map_err(|_| bad())?;
            if c.magnets != 6 && c.magnets != 12 {
                return Err("error: --magnets= takes 6 or 12.".into());
            }
        } else if let Some(v) = x.strip_prefix("--page=") {
            page = Some(v.to_string());
        } else if let Some(v) = x.strip_prefix("--solidcraft=") {
            sc = Some(v.to_string());
        } else if x == "--no-write" {
            write_out = false;
        } else {
            return Err(format!("error: {x}? It takes --kerf=, --pocket=, --layers=, --magnets=, --page=, --solidcraft= and --no-write."));
        }
    }

    let b = build(&c);
    let z = b.z;
    if let Some(dir) = sc {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{dir}: {e}"))?;
        let f = format!("{dir}/hex-dice-box.solidcraft.json");
        write(&f, &solidcraft_design::script(&c, &b)?)?;
        println!("wrote {f}");
        return Ok(0);
    }
    let (nb, nv) = counts(&z);
    let (placed, w, h) = pack(&b.parts, c.kerf)?;

    println!("box      {:.1}mm across the flats, {:.1} across the corners, {:.1} tall with the lid", 2.0 * z.a_out, 4.0 * z.a_out / s3(), z.h + T);
    println!(
        "pockets  7 × {}mm across the flats ({:.1} across the corners), {}mm deep under the lid",
        g(c.pocket),
        2.0 * c.pocket / s3(),
        g(c.layers as f64 * T + T)
    );
    println!("joints   floor edges {nb} cells of {:.2}mm, wall edges {nv} of {:.2}mm; wall-to-wall clear {:.2}mm", z.e / nb as f64, z.h / nv as f64, clear(120.0));
    println!("sheet    {w:.1} × {h:.1}mm of the {} × {} bed, {} parts", g(BED.0), g(BED.1), b.parts.len());
    let ns = b.spots.len();
    println!("magnets  {ns} in the top insert + {ns} in the plug = {} discs {} × {}mm", 2 * ns, g(MAG_D), g(MAG_T));

    let mut ok = true;
    let mut note = |good: bool, what: &str, detail: String| {
        ok &= good;
        println!("  {} {what}: {detail}", if good { "ok  " } else { "FAIL" });
    };
    println!("validation");
    let (over, leaks) = check_shell(&z, &b.wall, &b.floor, 400000, 1);
    note(over == 0, "no two box plates overlap", format!("{over} of 400000 points in two"));
    note(leaks == 0, "box seals inside", format!("{leaks} points of the inner skin in no plate"));
    let (gap, depth) = corner_contact(120.0);
    note(gap.abs() < 1e-9, "wall corners close on a line", format!("gap {gap:.3}mm, {depth:.1}mm in from the outside"));
    let plug = nipped(&hexagon(z.a_plug, (0.0, 0.0), 0.0), NIP);
    let pk = pockets(&z, &c);
    let rooms: Vec<f64> = b.spots.iter().map(|&p| room(p, &pk, &plug)).collect();
    let minroom = fmin(&rooms) - INDENT_D / 2.0;
    note(minroom >= MIN_WEB, "indents clear pockets and edge", format!("least ply beside an indent {minroom:.2}mm (>= {})", g(MIN_WEB)));
    note(INDENT_DEPTH < T - 1.0, "indents blind", format!("{:.1}mm of ply under each", T - INDENT_DEPTH));
    note(WEB >= 3.0 && RIM >= 3.0, "pocket webs", format!("{}mm between pockets, {}mm rim", g(WEB), g(RIM)));
    note(z.a_ins < z.a_in && z.a_plug < z.a_in, "inserts and plug fit inside", format!("{}mm and {}mm each side", g(FIT), g(PLUG_FIT)));
    note(h <= BED.1 && w <= BED.0, "one sheet", format!("{w:.1} × {h:.1}mm"));
    println!("validation {}", if ok { "passed" } else { "FAILED" });

    if write_out {
        std::fs::create_dir_all(format!("{HERE}/cut-files")).map_err(|e| e.to_string())?;
        let out = format!("{HERE}/cut-files/hex-dice-box.svg");
        let sheet = svg(&placed, w, h, &c);
        write(&out, &sheet)?;
        println!("wrote {out}");
        let tpl = std::fs::read_to_string(format!("{HERE}/page.html")).map_err(|e| format!("page.html: {e}"))?;
        let body = tpl.replace("/*DATA*/null", &data_json(&b, &c)).replace("<!--SHEET-->", sheet.split_once("?>").unwrap().1);
        // index.html stands alone, so it carries the document skeleton that
        // an artifact host supplies round --page='s copy
        let (head, main_) = body.split_once("<main>").ok_or("page.html has no <main>")?;
        let out = format!("{HERE}/index.html");
        write(
            &out,
            &format!(
                "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\">\n<style>body{{margin:0}}[hidden]{{display:none!important}}img{{max-width:100%}}</style>\n{head}</head>\n<body>\n<main>{main_}</body>\n</html>\n"
            ),
        )?;
        println!("wrote {out}");
        // turn.html: the 3D view alone, for framing on the hub
        let out = format!("{HERE}/turn.html");
        write(
            &out,
            &format!(
                "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<style>body{{margin:0}}[hidden]{{display:none!important}}</style>\n{head}<style>main>header,main>section,.hero>div:last-child{{display:none}}main{{padding:0!important;max-width:none}}.hero{{grid-template-columns:1fr}}.stage{{border:0;border-radius:0}}#view{{aspect-ratio:auto;height:calc(100vh - 52px)}}</style>\n</head>\n<body>\n<main>{main_}</body>\n</html>\n"
            ),
        )?;
        println!("wrote {out}");
        if let Some(pg) = page {
            write(&pg, &body)?;
            println!("wrote {pg}");
        }
    }
    Ok(if ok { 0 } else { 1 })
}

fn main() {
    std::process::exit(match run() {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    });
}
