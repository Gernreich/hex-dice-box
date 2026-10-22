#!/usr/bin/env python3
"""A hexagonal dice box for 3mm ply: six finger-jointed
walls on a finger-jointed floor, a stack of glued insert layers that make
seven hexagonal pockets -- six round one in the middle, a polyhedral set's
worth -- and a lid held on by 2 × 1mm neodymium disc magnets glued into
rastered indents.

THE BOX. A regular hexagonal prism, open at the top. The walls meet each
other at 120 degrees and the floor at 90, jointed as the four-hole
ocarinas' polyhedron.py joints any convex solid: along an edge the run is
cut into cells that alternate between the two plates; in its own cell a
plate reaches the edge line, in the other's it stops short by

    clear(theta) = T sin(theta)

which is T at the floor and 2.6mm between walls, where a V-notch shows
outside and the inside corner closes. The floor owns the bottom corners,
so the floor edges take an odd count of cells, the floor's at both ends;
the wall edges have no owner at either end and take an even count, so all
six walls are the same part. The walls' top edges are plain.

THE INSERTS. LAYERS hexagons, the box's inside less FIT all round, each
cut with the seven pockets, glued in a stack on the floor. Pockets are
regular hexagons POCKET across the flats, their flats parallel to the walls,
WEB apart and RIM in from the insert's edge. The top layer carries the
box's magnet indents, rastered into its top face.

THE LID. A plain hexagon the box's outside in size, which rests on the
wall tops, with a plug glued under it: the inside less PLUG_FIT all round,
pocketed like an insert so the dice get its 3mm of headroom too. The plug
drops into the 3mm left above the stack and lands on the top insert; its
indents, rastered into the face that ends up underneath, sit over the
box's, so each pair of magnets meets face to face.

THE MAGNETS. Discs MAG_D across and MAG_T thick in indents INDENT_D across
and INDENT_DEPTH deep, in two rings: one at the six outer corners, between
two outer pockets and the wall corner, and one at the six junctions where
the middle pocket meets two outer ones. --magnets=6 keeps only the corners.

Writes cut-files/hex-dice-box.svg (one sheet, every part) and, with
--page=OUT.html, a turnable 3D view with the sheet beside it. Takes
--kerf=MM (default 0.13, the measured figure), --pocket=MM, --layers=N,
--magnets=6|12 and --no-write.
"""
import json
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

T = 3.0              # ply
KERF = 0.13          # measured; --kerf= sets it
FINGER = 2 * T       # the least cell along an edge, the square flute's 6mm

POCKET = 25.0        # pocket across the flats: a 22mm metal d20 lies in it
WEB = 4.0            # ply between two pockets
RIM = 4.0            # ply between an outer pocket and the insert's edge
LAYERS = 6           # insert layers: 18mm, and the plug's 3mm, 21mm deep
FIT = 0.15           # insert to wall, each side: glued, a slide fit
PLUG_FIT = 0.3       # plug to wall, each side: the lid drops on freely
NIP = 0.5            # inserts' and plug's corners cut off, clear of glue

MAG_D, MAG_T = 2.0, 1.0
INDENT_D = 2.2       # room for a smear of glue round the disc
INDENT_DEPTH = 1.1   # the disc 0.1 below the face, glue under it
MAGNETS = 12

BED = (600.0, 305.0)  # the laser's work area
MARGIN, GAP = 5.0, 4.0
MIN_WEB = 1.0        # thinnest ply left beside an indent

# house inks: blue engraves, orange cuts inside a part, black frees it
BLUE, ORANGE, BLACK = '#0000ff', '#ff8000', '#000000'

S3 = math.sqrt(3)


def sizes():
    """Every size that follows from the settings."""
    s = POCKET + WEB                      # pocket centre to centre
    a_ins = s + POCKET / 2 + RIM          # insert apothem
    a_in = a_ins + FIT                    # box inside
    a_out = a_in + T                      # box outside
    E = 2 * a_out / S3                    # outside edge
    H = T + LAYERS * T + T                # floor, stack, plug
    return dict(s=s, a_ins=a_ins, a_in=a_in, a_out=a_out, E=E, H=H,
                a_plug=a_in - PLUG_FIT)


# --- 2D ------------------------------------------------------------------------

def hexagon(a, c=(0.0, 0.0), turn=0.0):
    """A regular hexagon of apothem a: flats facing 0, 60, ... degrees
    (plus turn), corners anticlockwise."""
    R = 2 * a / S3
    return [(c[0] + R * math.cos(math.radians(30 + 60 * k + turn)),
             c[1] + R * math.sin(math.radians(30 + 60 * k + turn))) for k in range(6)]


def area2(poly):
    return sum(x0 * y1 - x1 * y0 for (x0, y0), (x1, y1) in zip(poly, poly[1:] + poly[:1]))


def offset(poly, d):
    """Any simple polygon grown by d (shrunk if d < 0): every edge moved out
    parallel, each corner where its two moved edges cross."""
    s = 1 if area2(poly) > 0 else -1
    n = len(poly)
    lines = []
    for i in range(n):
        (x0, y0), (x1, y1) = poly[i], poly[(i + 1) % n]
        L = math.hypot(x1 - x0, y1 - y0)
        dx, dy = (x1 - x0) / L, (y1 - y0) / L
        nx, ny = s * dy, -s * dx                      # outward
        lines.append(((x0 + d * nx, y0 + d * ny), (dx, dy)))
    return [meet(lines[i - 1], lines[i]) for i in range(n)]


def meet(l1, l2):
    (p, d1), (q, d2) = l1, l2
    det = d1[0] * -d2[1] + d1[1] * d2[0]
    if abs(det) < 1e-12:                              # straight on: no corner
        return q
    r = ((q[0] - p[0]) * -d2[1] + (q[1] - p[1]) * d2[0]) / det
    return (p[0] + r * d1[0], p[1] + r * d1[1])


def simplify(pts):
    out = []
    for p in pts:
        p = (round(p[0], 6), round(p[1], 6))
        if not out or p != out[-1]:
            out.append(p)
    if out[0] == out[-1]:
        out.pop()
    changed = True
    while changed:
        changed = False
        for i in range(len(out)):
            a, b, c = out[i - 1], out[i], out[(i + 1) % len(out)]
            if abs((b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0])) < 1e-9:
                out.pop(i)
                changed = True
                break
    return out


def jointed(corners, runs):
    """A plate's outline: corners anticlockwise, and for each edge from
    corners[j] to corners[j+1] its cells as (s0, s1, depth). Each edge is
    walked at its depth inward; a corner is where the last set line of one
    edge crosses the first of the next."""
    k = len(corners)
    R = []
    for j in range(k):
        p, q = corners[j], corners[(j + 1) % k]
        L = math.hypot(q[0] - p[0], q[1] - p[1])
        dv = ((q[0] - p[0]) / L, (q[1] - p[1]) / L)
        nv = (-dv[1], dv[0])                          # inward, anticlockwise
        R.append((p, dv, nv, sorted(runs[j])))
    pts = []
    for j, (p, dv, nv, segs) in enumerate(R):
        at = lambda s, dp: (p[0] + s * dv[0] + dp * nv[0], p[1] + s * dv[1] + dp * nv[1])
        line = [at(s, dp) for s0, s1, dp in segs for s in (s0, s1)]
        pp, pd, pn, psegs = R[j - 1]
        a = (pp[0] + psegs[-1][2] * pn[0], pp[1] + psegs[-1][2] * pn[1])
        b = (p[0] + segs[0][2] * nv[0], p[1] + segs[0][2] * nv[1])
        pts += [meet((a, pd), (b, dv))] + line[1:-1]
    return simplify(pts)


def cells(L, n, mine, d):
    """n equal cells along L; cell i at depth 0 where mine(i), else d."""
    w = L / n
    return [(i * w, (i + 1) * w, 0.0 if mine(i) else d) for i in range(n)]


def clear(theta):
    return T * math.sin(math.radians(theta))


def counts(z):
    """Cells along the floor edges (odd: the floor's at both ends) and up
    the wall edges (even: no owner at either end), none under FINGER."""
    nb = int(z['E'] / FINGER + 1e-9)
    nb -= 1 - nb % 2
    nv = int(z['H'] / FINGER + 1e-9)
    nv -= nv % 2
    return nb, nv


def nipped(poly, d):
    """Every corner of a convex polygon cut off square to its bisector, d in
    from the corner."""
    c = (sum(p[0] for p in poly) / len(poly), sum(p[1] for p in poly) / len(poly))
    out = []
    for i, p in enumerate(poly):
        a, b = poly[i - 1], poly[(i + 1) % len(poly)]
        L = math.hypot(c[0] - p[0], c[1] - p[1])
        u = ((c[0] - p[0]) / L, (c[1] - p[1]) / L)
        for q in (a, b):
            e = (q[0] - p[0], q[1] - p[1])
            t = d / (e[0] * u[0] + e[1] * u[1])
            out.append((p[0] + t * e[0], p[1] + t * e[1]))
    return out


# --- parts ---------------------------------------------------------------------

class Part:
    def __init__(self, name, outline, holes=(), dots=(), note=''):
        self.name, self.outline = name, outline
        self.holes = list(holes)          # polygons, cut orange
        self.dots = list(dots)            # (x, y), indents engraved blue
        self.note = note
        xs, ys = [p[0] for p in outline], [p[1] for p in outline]
        self.x0, self.y0, self.w, self.h = min(xs), min(ys), max(xs) - min(xs), max(ys) - min(ys)


def pockets(z):
    s = z['s']
    cs = [(0.0, 0.0)] + [(s * math.cos(math.radians(60 * k)), s * math.sin(math.radians(60 * k)))
                         for k in range(6)]
    return [hexagon(POCKET / 2, c) for c in cs]


def seg_dist(p, a, b):
    dx, dy = b[0] - a[0], b[1] - a[1]
    t = max(0.0, min(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy)))
    return math.hypot(p[0] - a[0] - t * dx, p[1] - a[1] - t * dy)


def poly_dist(p, poly):
    return min(seg_dist(p, poly[i - 1], poly[i]) for i in range(len(poly)))


def inside(p, poly):
    c = False
    for i in range(len(poly)):
        (x0, y0), (x1, y1) = poly[i - 1], poly[i]
        if (y0 > p[1]) != (y1 > p[1]) and p[0] < x0 + (p[1] - y0) * (x1 - x0) / (y1 - y0):
            c = not c
    return c


def magnet_spots(z):
    """The indents' centres, the same in the top insert and the plug. The
    corner ring: on each line from the middle to a box corner, the point
    farthest from every pocket and from the plug's edge (the smaller of the
    two plates). The junction ring: where the middle pocket's corner points
    between two outer pockets, s/sqrt3 out."""
    plug = nipped(hexagon(z['a_plug']), NIP)
    pk = pockets(z)
    room = lambda p: min([poly_dist(p, h) for h in pk] + [poly_dist(p, plug)])
    spots = []
    for k in range(6):
        a = math.radians(30 + 60 * k)
        rs = [z['s'] / S3 + i * 0.01 for i in range(int((2 * z['a_plug'] / S3 - z['s'] / S3) / 0.01))]
        best = max((r for r in rs if not any(inside((r * math.cos(a), r * math.sin(a)), h) for h in pk)),
                   key=lambda r: room((r * math.cos(a), r * math.sin(a))))
        spots.append((best * math.cos(a), best * math.sin(a)))
    if MAGNETS == 12:
        r = z['s'] / S3
        spots += [(r * math.cos(math.radians(30 + 60 * k)), r * math.sin(math.radians(30 + 60 * k)))
                  for k in range(6)]
    return spots, room


def build():
    z = sizes()
    E, H = z['E'], z['H']
    nb, nv = counts(z)
    c90, c120 = clear(90), clear(120)
    parts = []

    floor = jointed(hexagon(z['a_out']),
                    [cells(E, nb, lambda i: i % 2 == 0, c90) for _ in range(6)])
    parts.append(Part('floor', floor, note='finger-jointed on all six edges'))

    # the wall seen from outside, u along the bottom, v up; its left side
    # (u = 0) owns the even cells counted from the floor, its right the odd
    wall = jointed([(0, 0), (E, 0), (E, H), (0, H)], [
        cells(E, nb, lambda i: i % 2 == 1, c90),
        cells(H, nv, lambda i: i % 2 == 1, c120),
        [(0.0, E, 0.0)],
        [(H - s1, H - s0, d) for s0, s1, d in cells(H, nv, lambda i: i % 2 == 0, c120)],
    ])
    for k in range(6):
        parts.append(Part(f'wall {k + 1}', wall, note='plain edge up'))

    pk = pockets(z)
    spots, room = magnet_spots(z)
    ins = nipped(hexagon(z['a_ins']), NIP)
    for k in range(LAYERS):
        top = k == LAYERS - 1
        parts.append(Part(f'insert {k + 1}' + (' (top)' if top else ''), ins, pk,
                          spots if top else (),
                          'indents up, glued last' if top else 'glued in the stack'))
    parts.append(Part('lid', hexagon(z['a_out']), note='plain, rests on the walls'))
    parts.append(Part('lid plug', nipped(hexagon(z['a_plug']), NIP), pk, spots,
                      'glued under the lid, indents DOWN'))
    return z, parts, wall, floor, spots, room


# --- 3D check ------------------------------------------------------------------

def plates3d(z, wall, floor):
    """The box's seven plates as (polygon, to3d(u, v, w)), w inward 0..T."""
    out = [(floor, lambda u, v, w: (u, v, w))]
    for k in range(6):
        a = math.radians(60 * k)
        n = (math.cos(a), math.sin(a))
        t = (-n[1], n[0])
        P0 = (z['a_out'] * n[0] - z['E'] / 2 * t[0], z['a_out'] * n[1] - z['E'] / 2 * t[1])
        out.append((wall, lambda u, v, w, n=n, t=t, P0=P0:
                    (P0[0] + u * t[0] - w * n[0], P0[1] + u * t[1] - w * n[1], v)))
    return out


def check_shell(z, wall, floor, N=400000, seed=1):
    """Throw points at the shell and count the plates holding each: none
    may be in two, and none in the inner 0.3mm of a wall or the floor may
    be in none (the seal). Points are tested in each plate's own frame.
    Within 3mm of an inside wall corner the skin is left out: there the
    cut-back wall touches its neighbour on a line, and inside that line a
    groove 0.87 by 1.5mm runs up the corner in its cells, which the shellac
    fills. corner_contact() measures that line."""
    rnd = random.Random(seed)
    frames = []
    for k in range(6):
        a = math.radians(60 * k)
        frames.append(((math.cos(a), math.sin(a)), (-math.sin(a), math.cos(a))))

    def held(p):
        x, y, zz = p
        n = 0
        if 0 <= zz <= T and inside((x, y), floor):
            n += 1
        for nn, t in frames:
            w = z['a_out'] - (x * nn[0] + y * nn[1])
            if not 0 <= w <= T:
                continue
            u = (x * t[0] + y * t[1]) + z['E'] / 2
            if inside((u, zz), wall):
                n += 1
        return n

    over = leaks = 0
    R = 2 * z['a_out'] / S3
    corners = hexagon(z['a_in'])
    for _ in range(N):
        p = (rnd.uniform(-R, R), rnd.uniform(-R, R), rnd.uniform(0, z['H']))
        apo = max(p[0] * math.cos(math.radians(60 * k)) + p[1] * math.sin(math.radians(60 * k))
                  for k in range(6))
        if apo > z['a_out']:
            continue
        h = held(p)
        if h > 1:
            over += 1
        if min(math.hypot(p[0] - c[0], p[1] - c[1]) for c in corners) < 3.0:
            continue
        inner_skin = (z['a_in'] <= apo <= z['a_in'] + 0.3 and p[2] >= T - 0.3) or \
                     (T - 0.3 <= p[2] <= T and apo <= z['a_in'])
        if inner_skin and h == 0:
            leaks += 1
    return over, leaks


def corner_contact(theta=120):
    """Where the cut-back wall's end meets its neighbour. In the cut-back
    wall's frame, along its plane from the edge line and depth in from its
    outside, the neighbour's inner end corner lies at (T sin phi, T cos phi),
    phi = 180 - theta the turn between the walls; the wall stops clear(theta)
    along. Returns the gap between the two (0 is a line contact) and the
    contact's depth."""
    phi = math.radians(180 - theta)
    return clear(theta) - T * math.sin(phi), T * math.cos(phi)


# --- sheet ---------------------------------------------------------------------

def path(pts):
    return 'M ' + ' L '.join(f'{x:.3f},{y:.3f}' for x, y in pts) + ' Z'


def pack(parts):
    """Shelves across the bed, tallest first; every part drawn turned a
    quarter so a hexagon stands on a corner, 2a wide."""
    order = sorted(parts, key=lambda p: (-p.h, -p.w))
    placed, x, y, rowh = [], MARGIN, MARGIN, 0.0
    for p in order:
        pw, ph = p.w + KERF, p.h + KERF
        if x + pw > BED[0] - MARGIN:
            x, y, rowh = MARGIN, y + rowh + GAP, 0.0
        if y + ph > BED[1] - MARGIN:
            raise SystemExit(f'error: {p.name} does not fit on the {BED[0]:g}x{BED[1]:g} bed.')
        placed.append((p, x, y))
        x += pw + GAP
        rowh = max(rowh, ph)
    w = max(x + p.w + KERF for p, x, y in placed) + MARGIN
    h = max(y + p.h + KERF for p, x, y in placed) + MARGIN
    return placed, w, h


def svg(placed, w, h, z):
    blue, orange, black = [], [], []
    for p, sx, sy in placed:
        # y flipped for the sheet's y-down; the parts are all symmetric
        dx, dy = sx - p.x0 + KERF / 2, sy + p.h + p.y0 + KERF / 2
        mv = lambda pts: [(x + dx, dy - y) for x, y in pts]
        black.append(path(mv(offset(p.outline, KERF / 2))))
        for hole in p.holes:
            orange.append(path(mv(offset(hole, -KERF / 2))))
        for cx, cy in p.dots:
            blue.append(f'<circle cx="{cx + dx:.3f}" cy="{dy - cy:.3f}" r="{INDENT_D / 2:.3f}"/>')
    desc = (f'1 user unit = 1mm. Hexagonal dice box, every part on one sheet: floor, six walls, '
            f'{LAYERS} insert layers, lid and lid plug. 3mm ply, drawn for a {KERF:g}mm kerf '
            f'(--kerf= recuts at another). Engrave first: blue {BLUE} is a fill engrave, the '
            f'magnet indents, {INDENT_D:g}mm across and {INDENT_DEPTH:g}mm deep for {MAG_D:g} × '
            f'{MAG_T:g}mm discs; set the depth on scrap. Then orange {ORANGE} cuts the pockets '
            f'and black {BLACK} frees the parts.')
    g = lambda col, items, fill='none', stroke=None: (
        f'  <g fill="{fill}" stroke="{stroke or "none"}"' + (' stroke-width="0.2"' if stroke else '')
        + '>\n' + ''.join(f'    {i}\n' if i.startswith('<') else f'    <path d="{i}"/>\n'
                          for i in items) + '  </g>\n')
    return (f'<?xml version="1.0" encoding="utf-8"?>\n'
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{w:.2f}mm" height="{h:.2f}mm" '
            f'viewBox="0 0 {w:.2f} {h:.2f}">\n'
            f'<title>Hexagonal dice box</title>\n<desc>{desc}</desc>\n'
            + g(BLUE, blue, fill=BLUE) + g(ORANGE, orange, stroke=ORANGE)
            + g(BLACK, black, stroke=BLACK) + '</svg>\n')


# --- the page ------------------------------------------------------------------

def model(z, parts, wall, floor):
    """Every part placed in the assembly, for the 3D view: outline, holes,
    dots and a 4x4 matrix taking (u, v, w) to the box's frame, w up from the
    part's lower face or inward."""
    m = []
    put = lambda name, p, M, kind: m.append(dict(
        name=name, kind=kind, outline=p.outline, holes=p.holes, dots=p.dots, m=M))
    by = {p.name: p for p in parts}
    put('floor', by['floor'], [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0]], 'box')
    for k in range(6):
        a = math.radians(60 * k)
        n, t = (math.cos(a), math.sin(a)), (-math.sin(a), math.cos(a))
        P0 = (z['a_out'] * n[0] - z['E'] / 2 * t[0], z['a_out'] * n[1] - z['E'] / 2 * t[1])
        put(f'wall {k + 1}', by[f'wall {k + 1}'],
            [[t[0], 0, -n[0], P0[0]], [t[1], 0, -n[1], P0[1]], [0, 1, 0, 0]], 'box')
    for k in range(LAYERS):
        name = next(p.name for p in parts if p.name.startswith(f'insert {k + 1}'))
        put(name, by[name], [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, T * (k + 1)]], 'insert')
    top = z['H']
    put('lid plug', by['lid plug'], [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, top - T]], 'lid')
    put('lid', by['lid'], [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, top]], 'lid')
    return m


# --- main ----------------------------------------------------------------------

def main():
    global KERF, POCKET, LAYERS, MAGNETS
    write, page = True, None
    for x in sys.argv[1:]:
        try:
            if x.startswith('--kerf='):
                KERF = float(x[7:])
                if not 0 <= KERF <= 0.5:
                    raise SystemExit(f'error: a {KERF:g}mm kerf is outside 0 to 0.5mm.')
            elif x.startswith('--pocket='):
                POCKET = float(x[9:])
            elif x.startswith('--layers='):
                LAYERS = int(x[9:])
            elif x.startswith('--magnets='):
                MAGNETS = int(x[10:])
                if MAGNETS not in (6, 12):
                    raise SystemExit('error: --magnets= takes 6 or 12.')
            elif x.startswith('--page='):
                page = x[7:]
            elif x == '--no-write':
                write = False
            else:
                raise SystemExit(f'error: {x}? It takes --kerf=, --pocket=, --layers=, '
                                 f'--magnets=, --page= and --no-write.')
        except ValueError:
            raise SystemExit(f'error: {x} is not a number.')

    z, parts, wall, floor, spots, room = build()
    nb, nv = counts(z)
    placed, w, h = pack(parts)

    print(f'box      {2 * z["a_out"]:.1f}mm across the flats, {4 * z["a_out"] / S3:.1f} across '
          f'the corners, {z["H"] + T:.1f} tall with the lid')
    print(f'pockets  7 × {POCKET:g}mm across the flats ({2 * POCKET / S3:.1f} across the corners), '
          f'{LAYERS * T + T:g}mm deep under the lid')
    print(f'joints   floor edges {nb} cells of {z["E"] / nb:.2f}mm, wall edges {nv} of '
          f'{z["H"] / nv:.2f}mm; wall-to-wall clear {clear(120):.2f}mm')
    print(f'sheet    {w:.1f} × {h:.1f}mm of the {BED[0]:g} × {BED[1]:g} bed, {len(parts)} parts')
    print(f'magnets  {len(spots)} in the top insert + {len(spots)} in the plug = {2 * len(spots)} '
          f'discs {MAG_D:g} × {MAG_T:g}mm')

    ok = True

    def note(good, what, detail):
        nonlocal ok
        ok &= good
        print(f'  {"ok  " if good else "FAIL"} {what}: {detail}')

    print('validation')
    over, leaks = check_shell(z, wall, floor)
    note(over == 0, 'no two box plates overlap', f'{over} of 400000 points in two')
    note(leaks == 0, 'box seals inside', f'{leaks} points of the inner skin in no plate')
    gap, depth = corner_contact()
    note(abs(gap) < 1e-9, 'wall corners close on a line',
         f'gap {gap:.3f}mm, {depth:.1f}mm in from the outside')
    minroom = min(room(p) for p in spots) - INDENT_D / 2
    note(minroom >= MIN_WEB, 'indents clear pockets and edge',
         f'least ply beside an indent {minroom:.2f}mm (>= {MIN_WEB:g})')
    note(INDENT_DEPTH < T - 1.0, 'indents blind', f'{T - INDENT_DEPTH:.1f}mm of ply under each')
    note(WEB >= 3 and RIM >= 3, 'pocket webs', f'{WEB:g}mm between pockets, {RIM:g}mm rim')
    note(z['a_ins'] < z['a_in'] and z['a_plug'] < z['a_in'], 'inserts and plug fit inside',
         f'{FIT:g}mm and {PLUG_FIT:g}mm each side')
    note(h <= BED[1] and w <= BED[0], 'one sheet', f'{w:.1f} × {h:.1f}mm')
    print('validation ' + ('passed' if ok else 'FAILED'))

    if write:
        os.makedirs(os.path.join(HERE, 'cut-files'), exist_ok=True)
        out = os.path.join(HERE, 'cut-files', 'hex-dice-box.svg')
        sheet = svg(placed, w, h, z)
        open(out, 'w').write(sheet)
        print(f'wrote {out}')
        tpl = open(os.path.join(HERE, 'page.html')).read()
        data = dict(model=model(z, parts, wall, floor), T=T, sizes=z, pocket=POCKET,
                    layers=LAYERS, magnets=len(spots), kerf=KERF, indent=INDENT_D,
                    depth=INDENT_DEPTH)
        body = (tpl.replace('/*DATA*/null', json.dumps(data))
                .replace('<!--SHEET-->', sheet.split('?>', 1)[1]))
        # index.html stands alone, so it carries the document skeleton that
        # an artifact host supplies round --page='s copy
        head, main_ = body.split('<main>', 1)
        out = os.path.join(HERE, 'index.html')
        open(out, 'w').write(
            '<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n'
            '<meta name="viewport" content="width=device-width, initial-scale=1, '
            'viewport-fit=cover">\n<style>body{margin:0}[hidden]{display:none!important}'
            'img{max-width:100%}</style>\n' + head + '</head>\n<body>\n<main>'
            + main_ + '</body>\n</html>\n')
        print(f'wrote {out}')
        # turn.html: the 3D view alone, for framing on the hub
        out = os.path.join(HERE, 'turn.html')
        open(out, 'w').write(
            '<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n'
            '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
            '<style>body{margin:0}[hidden]{display:none!important}</style>\n' + head
            + '<style>main>header,main>section,.hero>div:last-child{display:none}'
            'main{padding:0!important;max-width:none}.hero{grid-template-columns:1fr}'
            '.stage{border:0;border-radius:0}#view{aspect-ratio:auto;height:calc(100vh - 52px)}</style>\n</head>\n<body>\n<main>'
            + main_ + '</body>\n</html>\n')
        print(f'wrote {out}')
        if page:
            open(page, 'w').write(body)
            print(f'wrote {page}')
    return 0 if ok else 1


if __name__ == '__main__':
    sys.exit(main())
