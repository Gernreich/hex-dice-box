//! The box as a parametric SolidCraft design (2026-11-08):
//! `hex_dice_box --solidcraft=DIR` writes it, and SolidCraft builds the STEP
//! file from it.
//!
//! The ply, the pocket, the webs, the fits, the nip and the indents are
//! parameters, and every point of every plate is worked out from them as the
//! generator works it out: the hexagons, each finger joint's cells and their
//! cut-backs, the nipped corners, the pockets. Every one of those points is a
//! sum of the box's sizes times fixed numbers (the hexagon's directions never
//! change), so each is written as that sum and follows its parameters. The
//! counts stay the default build's: fingers on each edge, insert layers,
//! magnets. The corner magnets sit where the generator's search put them,
//! `mag_r` from the middle; the junction ring is s/sqrt3 out and follows.
//!
//! Unlike the old STEP file, which was the 3D view's plates, this is the box
//! as it is cut: the indents are in it, blind holes in the top insert's top
//! face and the plug's underside. The magnets are not.
use crate::solidcraft::{n as nn, Script};
use crate::{counts, pocket_centres, Built, Cfg, P};

/// A length as a sum of named parameters times numbers; "" is millimetres.
#[derive(Clone, Debug, Default)]
struct Lin(Vec<(&'static str, f64)>);

impl Lin {
    fn of(name: &'static str, k: f64) -> Lin {
        Lin(vec![(name, k)])
    }
    fn add(&self, o: &Lin) -> Lin {
        let mut v = self.0.clone();
        for &(n, k) in &o.0 {
            match v.iter_mut().find(|(m, _)| *m == n) {
                Some(t) => t.1 += k,
                None => v.push((n, k)),
            }
        }
        Lin(v)
    }
    fn sub(&self, o: &Lin) -> Lin {
        self.add(&o.mul(-1.0))
    }
    fn mul(&self, k: f64) -> Lin {
        Lin(self.0.iter().map(|&(n, c)| (n, c * k)).collect())
    }
    fn val(&self, env: &[(&str, f64)]) -> f64 {
        self.0.iter().map(|&(n, k)| k * if n.is_empty() { 1.0 } else { env.iter().find(|(m, _)| *m == n).unwrap().1 }).sum()
    }
    fn e(&self) -> String {
        let t: Vec<String> = self
            .0
            .iter()
            .filter(|(_, k)| k.abs() > 1e-12)
            .map(|&(n, k)| if n.is_empty() { format!("{} mm", nn(k)) } else { format!("{}*{n}", nn(k)) })
            .collect();
        if t.is_empty() {
            "0 mm".into()
        } else {
            format!("({})", t.join(" + "))
        }
    }
}

/// A point: its coordinates as Lins, and their value at the defaults.
#[derive(Clone, Debug)]
struct Pt {
    x: Lin,
    y: Lin,
    v: P,
}

struct G<'a> {
    env: Vec<(&'a str, f64)>,
}

impl<'a> G<'a> {
    fn pt(&self, x: Lin, y: Lin) -> Pt {
        let v = (x.val(&self.env), y.val(&self.env));
        Pt { x, y, v }
    }
    fn at(&self, p: &Pt, s: &Lin, d: P, dp: &Lin, nv: P) -> Pt {
        self.pt(p.x.add(&s.mul(d.0)).add(&dp.mul(nv.0)), p.y.add(&s.mul(d.1)).add(&dp.mul(nv.1)))
    }
    /// hexagon(a, c): corners anticlockwise, flats facing 0, 60, ... degrees
    fn hexagon(&self, a: &Lin, c: (&Lin, &Lin)) -> Vec<Pt> {
        let r = a.mul(2.0 / 3f64.sqrt());
        (0..6)
            .map(|k| {
                let t = (30.0 + 60.0 * k as f64).to_radians();
                self.pt(c.0.add(&r.mul(t.cos())), c.1.add(&r.mul(t.sin())))
            })
            .collect()
    }
    /// meet() of two lines whose directions are fixed numbers
    fn meet(&self, p: &Pt, d1: P, q: &Pt, d2: P) -> Pt {
        let det = d1.0 * -d2.1 + d1.1 * d2.0;
        if det.abs() < 1e-12 {
            return q.clone();
        }
        let r = q.x.sub(&p.x).mul(-d2.1 / det).add(&q.y.sub(&p.y).mul(d2.0 / det));
        self.pt(p.x.add(&r.mul(d1.0)), p.y.add(&r.mul(d1.1)))
    }
    /// jointed(): the plate outline from its corners and each edge's cells,
    /// (s0, s1, depth) as Lins
    fn jointed(&self, corners: &[Pt], runs: &[Vec<(Lin, Lin, Lin)>]) -> Vec<Pt> {
        let k = corners.len();
        let mut rr = vec![];
        for j in 0..k {
            let (p, q) = (&corners[j], &corners[(j + 1) % k]);
            let l = (q.v.0 - p.v.0).hypot(q.v.1 - p.v.1);
            let dv = ((q.v.0 - p.v.0) / l, (q.v.1 - p.v.1) / l);
            let nv = (-dv.1, dv.0);
            let mut segs = runs[j].clone();
            segs.sort_by(|a, b| a.0.val(&self.env).partial_cmp(&b.0.val(&self.env)).unwrap());
            rr.push((p.clone(), dv, nv, segs));
        }
        let mut pts = vec![];
        for j in 0..k {
            let (p, dv, nv, segs) = &rr[j];
            let line: Vec<Pt> = segs.iter().flat_map(|(s0, s1, dp)| [self.at(p, s0, *dv, dp, *nv), self.at(p, s1, *dv, dp, *nv)]).collect();
            let (pp, pd, pn, psegs) = &rr[(j + k - 1) % k];
            let last = &psegs[psegs.len() - 1].2;
            let a = self.pt(pp.x.add(&last.mul(pn.0)), pp.y.add(&last.mul(pn.1)));
            let first = &segs[0].2;
            let b = self.pt(p.x.add(&first.mul(nv.0)), p.y.add(&first.mul(nv.1)));
            pts.push(self.meet(&a, *pd, &b, *dv));
            pts.extend_from_slice(&line[1..line.len() - 1]);
        }
        simplify(pts)
    }
    /// nipped(): each corner of a regular polygon cut square to its
    /// bisector, d in. On a regular polygon e/(e.u) is a fixed vector, so
    /// the cut points are the corner plus d times it.
    fn nipped(&self, poly: &[Pt], d: &Lin) -> Vec<Pt> {
        let n = poly.len();
        let c = (poly.iter().map(|p| p.v.0).sum::<f64>() / n as f64, poly.iter().map(|p| p.v.1).sum::<f64>() / n as f64);
        let mut out = vec![];
        for i in 0..n {
            let p = &poly[i];
            let l = (c.0 - p.v.0).hypot(c.1 - p.v.1);
            let u = ((c.0 - p.v.0) / l, (c.1 - p.v.1) / l);
            for q in [&poly[(i + n - 1) % n], &poly[(i + 1) % n]] {
                let e = (q.v.0 - p.v.0, q.v.1 - p.v.1);
                let k = e.0 * u.0 + e.1 * u.1;
                out.push(self.pt(p.x.add(&d.mul(e.0 / k)), p.y.add(&d.mul(e.1 / k))));
            }
        }
        out
    }
}

/// simplify(): drop repeats and points in line, judged on the values
fn simplify(pts: Vec<Pt>) -> Vec<Pt> {
    let r6 = |x: f64| (x * 1e6).round() / 1e6;
    let mut out: Vec<Pt> = vec![];
    for p in pts {
        if out.last().map_or(true, |q| (r6(q.v.0), r6(q.v.1)) != (r6(p.v.0), r6(p.v.1))) {
            out.push(p);
        }
    }
    if (r6(out[0].v.0), r6(out[0].v.1)) == (r6(out[out.len() - 1].v.0), r6(out[out.len() - 1].v.1)) {
        out.pop();
    }
    let mut changed = true;
    while changed {
        changed = false;
        let n = out.len();
        for i in 0..n {
            let (a, b, c) = (out[(i + n - 1) % n].v, out[i].v, out[(i + 1) % n].v);
            let (a, b, c) = ((r6(a.0), r6(a.1)), (r6(b.0), r6(b.1)), (r6(c.0), r6(c.1)));
            if ((b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)).abs() < 1e-9 {
                out.remove(i);
                changed = true;
                break;
            }
        }
    }
    out
}

fn off(got: &[Pt], want: &[P]) -> f64 {
    if got.len() != want.len() {
        return f64::INFINITY;
    }
    got.iter().zip(want).map(|(g, w)| (g.v.0 - w.0).hypot(g.v.1 - w.1)).fold(0.0, f64::max)
}

fn area(p: &[P]) -> f64 {
    let n = p.len();
    (0..n).map(|i| p[i].0 * p[(i + 1) % n].1 - p[(i + 1) % n].0 * p[i].1).sum::<f64>().abs() / 2.0
}

pub fn script(c: &Cfg, b: &Built) -> Result<String, String> {
    let z = &b.z;
    let mut s = Script::new();
    let t = crate::T;
    s.len("ply", format!("{} mm", nn(t)), "the ply: every plate");
    s.len("pocket", format!("{} mm", nn(c.pocket)), "each pocket, across the flats");
    s.len("web", format!("{} mm", nn(crate::WEB)), "ply between two pockets");
    s.len("rim", format!("{} mm", nn(crate::RIM)), "ply between an outer pocket and the insert's edge");
    s.len("fit", format!("{} mm", nn(crate::FIT)), "insert to wall, each side");
    s.len("plug_fit", format!("{} mm", nn(crate::PLUG_FIT)), "lid plug to wall, each side");
    s.len("nip", format!("{} mm", nn(crate::NIP)), "the inserts' and plug's corners cut off, in from the corner");
    s.len("indent_d", format!("{} mm", nn(crate::INDENT_D)), "a magnet indent, across");
    s.len("indent_depth", format!("{} mm", nn(crate::INDENT_DEPTH)), "a magnet indent, deep");
    s.len("mag_r", format!("{} mm", nn(b.corner_r)), "the corner magnets from the middle: where the generator's search put them");
    s.num("layers", format!("{}", c.layers), "read only: insert layers, the default build's");
    s.len("s", "pocket + web", "read only: pocket centre to centre");
    s.len("a_ins", "s + pocket/2 + rim", "read only: an insert's apothem");
    s.len("a_in", "a_ins + fit", "read only: the box's inside apothem");
    s.len("a_out", "a_in + ply", "read only: the box's outside apothem");
    s.len("a_plug", "a_in - plug_fit", "read only: the plug's apothem");
    s.len("edge", "2*a_out/sqrt(3)", "read only: the outside edge, a wall's width");
    s.len("box_h", "(layers + 2)*ply", "read only: floor, stack and plug: a wall's height");
    let g = G {
        env: vec![("ply", t), ("pocket", c.pocket), ("s", z.s), ("a_ins", z.a_ins), ("a_out", z.a_out), ("a_plug", z.a_plug), ("nip", crate::NIP), ("mag_r", b.corner_r)],
    };
    let zero = Lin::default();
    let (nb, nv) = counts(z);
    let r3 = 2.0 / 3f64.sqrt();
    let e = Lin::of("a_out", r3);
    let h = Lin::of("ply", (c.layers + 2) as f64);
    let c90 = Lin::of("ply", 1.0);
    let c120 = Lin::of("ply", (120f64).to_radians().sin());
    let cells = |l: &Lin, n: i64, mine: &dyn Fn(i64) -> bool, d: &Lin| -> Vec<(Lin, Lin, Lin)> {
        (0..n).map(|i| (l.mul(i as f64 / n as f64), l.mul((i + 1) as f64 / n as f64), if mine(i) { Lin::default() } else { d.clone() })).collect()
    };
    let floor = g.jointed(&g.hexagon(&Lin::of("a_out", 1.0), (&zero, &zero)), &(0..6).map(|_| cells(&e, nb, &|i| i % 2 == 0, &c90)).collect::<Vec<_>>());
    let left: Vec<(Lin, Lin, Lin)> = cells(&h, nv, &|i| i % 2 == 0, &c120).into_iter().map(|(s0, s1, d)| (h.sub(&s1), h.sub(&s0), d)).collect();
    let corners = [g.pt(zero.clone(), zero.clone()), g.pt(e.clone(), zero.clone()), g.pt(e.clone(), h.clone()), g.pt(zero.clone(), h.clone())];
    let wall = g.jointed(&corners, &[cells(&e, nb, &|i| i % 2 == 1, &c90), cells(&h, nv, &|i| i % 2 == 1, &c120), vec![(zero.clone(), e.clone(), zero.clone())], left]);
    let nip = Lin::of("nip", 1.0);
    let ins = g.nipped(&g.hexagon(&Lin::of("a_ins", 1.0), (&zero, &zero)), &nip);
    let plug = g.nipped(&g.hexagon(&Lin::of("a_plug", 1.0), (&zero, &zero)), &nip);
    let lid = g.hexagon(&Lin::of("a_out", 1.0), (&zero, &zero));
    let pks: Vec<Vec<Pt>> = pocket_centres(z)
        .iter()
        .enumerate()
        .map(|(k, _)| {
            let (cx, cy) = if k == 0 {
                (zero.clone(), zero.clone())
            } else {
                let a = (60.0 * (k - 1) as f64).to_radians();
                (Lin::of("s", a.cos()), Lin::of("s", a.sin()))
            };
            g.hexagon(&Lin::of("pocket", 0.5), (&cx, &cy))
        })
        .collect();
    let mut spots: Vec<Pt> = (0..6)
        .map(|k| {
            let a = (30.0 + 60.0 * k as f64).to_radians();
            g.pt(Lin::of("mag_r", a.cos()), Lin::of("mag_r", a.sin()))
        })
        .collect();
    if c.magnets == 12 {
        spots.extend((0..6).map(|k| {
            let a = (30.0 + 60.0 * k as f64).to_radians();
            g.pt(Lin::of("s", a.cos() / 3f64.sqrt()), Lin::of("s", a.sin() / 3f64.sqrt()))
        }));
    }
    // each against the generator's own
    let part = |n: &str| b.parts.iter().find(|p| p.name.starts_with(n)).unwrap();
    let top = part(&format!("insert {}", c.layers));
    let checks: Vec<(&str, f64)> = vec![
        ("floor", off(&floor, &b.floor)),
        ("wall", off(&wall, &b.wall)),
        ("insert", off(&ins, &top.outline)),
        ("lid plug", off(&plug, &part("lid plug").outline)),
        ("lid", off(&lid, &part("lid").outline)),
        ("pockets", (0..7).map(|k| off(&pks[k], &top.holes[k])).fold(0.0, f64::max)),
        ("indents", off(&spots, &b.spots)),
    ];
    for (what, d) in &checks {
        // the generator rounds its jointed outlines to 6 decimals
        if *d > 2e-6 {
            return Err(format!("the design's {what} is {d:e}mm off the generator's"));
        }
    }
    // the parts
    let xy = |p: &Pt| [p.x.e(), p.y.e()];
    let ring = |p: &[Pt]| p.iter().map(xy).collect::<Vec<_>>();
    let mut vols: Vec<f64> = vec![];
    let pv = |p: &[Pt]| p.iter().map(|q| q.v).collect::<Vec<P>>();
    let hole_area: f64 = pks.iter().map(|h| area(&pv(h))).sum();
    let dot = std::f64::consts::PI * (crate::INDENT_D / 2.0).powi(2) * crate::INDENT_DEPTH;
    // floor
    let sk = s.sketch("XY", None, "floor sketch");
    s.poly(&ring(&floor));
    s.finish();
    s.extrude(&sk, "ply", false, "new", &[], "floor");
    vols.push(area(&pv(&floor)) * t);
    // the walls, each in the plane of its outside face, a_out out along its
    // normal, extruded ply inward
    for k in 0..6 {
        let a = (60.0 * k as f64).to_radians();
        let tv = [-a.sin(), a.cos(), 0.0];
        let name = format!("wall {}", k + 1);
        let sk = s.sketch_on([0.0, 0.0, 0.0], tv, [0.0, 0.0, 1.0], Some("a_out"), &format!("{name} sketch"));
        let pts: Vec<[String; 2]> = wall.iter().map(|p| [p.x.sub(&e.mul(0.5)).e(), p.y.e()]).collect();
        s.poly(&pts);
        s.finish();
        s.extrude(&sk, "ply", true, "new", &[], &name);
        vols.push(area(&pv(&wall)) * t);
    }
    // the inserts, the top one with its indents in its top face
    for k in 0..c.layers {
        let topk = k == c.layers - 1;
        let name = format!("insert {}{}", k + 1, if topk { " (top)" } else { "" });
        let sk = s.sketch("XY", Some(&format!("{}*ply", k + 1)), &format!("{name} sketch"));
        s.poly(&ring(&ins));
        for h in &pks {
            s.poly(&ring(h));
        }
        s.finish();
        s.extrude(&sk, "ply", false, "new", &[], &name);
        let mut v = (area(&pv(&ins)) - hole_area) * t;
        if topk {
            for (j, p) in spots.iter().enumerate() {
                let base = [p.x.e(), p.y.e(), format!("{}*ply - indent_depth", k + 2)];
                s.cylinder(&format!("{name} indent {}", j + 1), base, Some([0.0, 0.0, 1.0]), "indent_d", "indent_depth + 1 mm", "cut", &[&name]);
                v -= dot;
            }
        }
        vols.push(v);
    }
    // the lid plug, its indents in its underside, and the lid
    let sk = s.sketch("XY", Some("box_h - ply"), "lid plug sketch");
    s.poly(&ring(&plug));
    for h in &pks {
        s.poly(&ring(h));
    }
    s.finish();
    s.extrude(&sk, "ply", false, "new", &[], "lid plug");
    let mut v = (area(&pv(&plug)) - hole_area) * t;
    for (j, p) in spots.iter().enumerate() {
        let base = [p.x.e(), p.y.e(), "box_h - ply - 1 mm".to_string()];
        s.cylinder(&format!("lid plug indent {}", j + 1), base, Some([0.0, 0.0, 1.0]), "indent_d", "indent_depth + 1 mm", "cut", &["lid plug"]);
        v -= dot;
    }
    vols.push(v);
    let sk = s.sketch("XY", Some("box_h"), "lid sketch");
    s.poly(&ring(&lid));
    s.finish();
    s.extrude(&sk, "ply", false, "new", &[], "lid");
    vols.push(area(&pv(&lid)) * t);
    s.measure();
    if std::env::var("SC_VOLUMES").is_ok() {
        for v in &vols {
            eprintln!("{v:.4}");
        }
    }
    Ok(s.text())
}
