# Hexagonal dice box

**[← All projects](https://gernreich.github.io/)**

**[Read the writeup](https://gernreich.github.io/hex-dice-box/)**: spin the box
in 3D, with the cut sheet and the assembly steps on the same page.

A laser-cut dice box for 3 mm ply that fits a 600 × 305 mm laser bed. Six
finger-jointed walls sit on a hexagonal floor. Inside, a stack of glued
insert layers makes seven hexagonal pockets, six round one in the middle,
for a polyhedral dice set. The lid drops into the top and is held by 2 × 1 mm
neodymium discs glued into rastered indents.

That page is `index.html`, published by GitHub Pages; opened locally it
works too, given a network connection for the fonts and the 3D library.

Nothing has been cut yet.

## The design

| | |
|---|---|
| Box | 97.3 mm across the flats, 112.4 across the corners, 27 tall with the lid |
| Pockets | 7 × 25 mm across the flats, 21 mm deep (6 insert layers + the lid plug) |
| Joints | floor edges 9 fingers of 6.24 mm, wall edges 4 of 6.00 mm |
| Magnets | 12 pairs of 2 × 1 mm discs, 24 discs in all |
| Indents | 2.2 mm across, 1.1 mm deep |

- **Box.** All six walls are the same part, plain edge up. Walls meet at
  120 degrees: in the other wall's cells a wall stops T sin(120) = 2.6 mm short
  of the edge, the least that keeps the two slabs apart. A small V shows
  outside each corner, and inside the joint closes on a line with a
  0.87 × 1.5 mm groove. A coat of shellac fills both.
- **Inserts.** Each layer is the box's inside less 0.15 mm all round, with
  the seven pockets cut out. They're glued in a stack on the floor. The top
  layer carries the box's magnet indents, facing up, and ends 3 mm below the
  wall tops.
- **Lid.** A plain hexagon the size of the box's outside, sitting on the
  wall tops, with a plug glued under it. The plug is pocketed like an insert
  and has 0.3 mm clearance each side. Its indents face down and line up with
  the top insert's, so each pair of discs meets face to face.
- **Magnets.** Six at the outer corners and six where the middle pocket
  meets two outer ones. `--magnets=6` keeps only the corners.

## Cutting

One sheet, `cut-files/hex-dice-box.svg`, 1 user unit = 1 mm:

1. **Blue**: fill engrave, the magnet indents. Set power and passes on
   scrap until a 1 mm disc sits just below the face.
2. **Orange**: cut, the pockets.
3. **Black**: cut, frees the parts.

## Assembly

1. Press the six walls into the floor, plain edge up. The finger joints are
   press-fit, not glued. Give the box a coat of shellac.
2. Glue the plain insert layers in one at a time, with the pockets lined up
   (a die pushed through each one keeps them in register). Glue the layer
   with indents last, indents up.
3. Glue the plug under the lid, plain face to the lid, indents down.
4. CA-glue a disc into each indent of the top insert, every one with the
   same pole up.
5. Stand a second disc on each box disc, put a drop of CA on top, and lower
   the lid until the glue sets. Every lid disc lands pole-correct, and the
   lid closes in any of its six turns.

## Regenerating

The generator is Rust (it was Python until 2026-11-08; the port changed no
byte of the sheet or the pages). It needs nothing beyond Rust itself:

    cargo build --release
    ./target/release/hex_dice_box [--kerf=MM] [--pocket=MM] [--layers=N] [--magnets=6|12] [--page=PATH] [--no-write]

- `hex_dice_box.rs` is the generator (`Cargo.toml` and `Cargo.lock` build it). `src/solidcraft_design.rs` writes the
  SolidCraft design (below); `src/py.rs`, `src/pyrandom.rs` and
  `src/solidcraft.rs` are copied from the LaserMadeMusic tools, so the
  repository builds alone: Python's number formatting and random numbers,
  which keep the output what the Python wrote, and the script writer.

- `--kerf=` is the laser's real kerf (default 0.13 mm, measured). Every
  outline is offset by half of it, and the finger joints are press-fit from
  the kerf alone, so recut at a wider kerf if they come out loose, or a
  narrower one if they're tight.
- `--pocket=` and `--layers=` set the pocket size and depth for bigger
  dice.
- Every run rewrites `index.html` from the `page.html` template, and
  `turn.html`, the 3D view alone, which the hub frames. `--page=PATH`
  also writes a copy without the document skeleton, for hosts that supply
  their own.

Every run prints a validation block: no two box plates overlap (400,000
sample points), the inside seals, the wall corners close on a line, the
indents are blind and clear of the pockets, the inserts and plug fit, and
everything fits on one sheet.

Released under [CC0 1.0](LICENSE).

## SolidCraft design and STEP file

`solidcraft/hex-dice-box.solidcraft.json` is the box as a parametric SolidCraft
design, written by

    ./target/release/hex_dice_box --solidcraft=solidcraft

and SolidCraft builds the design file and the STEP file from it:

    cd solidcraft
    solidcraft-cli run hex-dice-box.solidcraft.json --save hex-dice-box.sc --out ../step/hex-dice-box.step --quiet
    ~/LaserMadeMusic/GIT/lasermade-tools/target/release/sc-slim hex-dice-box.sc

`sc-slim` strips SolidCraft's build cache so the file comes out the same every
time. The design is one body per plate, 15 in all: the floor, six walls, six
inserts, the lid plug and the lid, assembled with the lid on, in millimetres.
It is the box as cut, so the magnet indents are in it: blind holes in the top
insert's top face and the plug's underside. The magnets are not.

The parameters:

| parameter | default | what it is |
|---|---|---|
| `ply` | 3 mm | every plate's thickness; the walls' height follows it |
| `pocket` | 25 mm | each pocket, across the flats |
| `web` | 4 mm | ply between two pockets |
| `rim` | 4 mm | ply between an outer pocket and an insert's edge |
| `fit` | 0.15 mm | insert to wall, each side |
| `plug_fit` | 0.3 mm | lid plug to wall, each side |
| `nip` | 0.5 mm | the inserts' and plug's corners cut off |
| `finger` | 6 mm | the least cell along a jointed edge; the counts follow from it |
| `indent_d`, `indent_depth` | 2.2, 1.1 mm | a magnet indent |
| `mag_r` | 41.77 mm | the corner magnets from the middle, where the generator's search put them |

The read-only ones (`s`, `a_ins`, `a_in`, `a_out`, `a_plug`, `edge`,
`box_h`, `n_b`, `n_v`, `cb`, `cv`) show the sizes that follow. Every point of
every plate is an expression in them, worked out as the generator works it
out: the nipped corners, the pockets, and the finger joints. The joints'
counts follow the sizes as the generator's do: `n_b` cells along a floor edge
(odd, none under `finger`, 6 mm) and `n_v` up a wall edge (even), each a
floor() expression, 9 and 4 at the defaults. The floor and walls are drawn
plain, and each row of cells cut from them is one cut, patterned along its
edge two cells apart. The insert layers (6) and magnets (12) stay the
default build's.

Before writing the design the generator checks every outline, pocket and
indent in it against its own, to within 2e-6 mm. Built, each body's volume
matches the generator's to within 1e-8 (2e-6 on the two with indents, which
SolidCraft meshes as polygons), and no two plates overlap. With `pocket` at
34 mm the floor gets 11 cells an edge, and the floor, walls and lid match
`--pocket=34`'s to within 1e-4 mm³; with `ply` at 4.5 mm the walls get 6 cells
a side and match a generator built with 4.5 mm ply the same way, and with
both changed too.

The STEP file used to be written from the 3D view by `step-export`; since
2026-11-08 SolidCraft builds it from the design.

The files:

- `solidcraft/hex-dice-box.solidcraft.json`
- `solidcraft/hex-dice-box.sc`
- `step/hex-dice-box.step`

## Credits

Made by Michael Gernreich with [Claude](https://www.anthropic.com/claude) (Anthropic), working in Claude Code, which co-wrote the code and the documentation. Commits Claude made carry a `Co-Authored-By: Claude` line.
