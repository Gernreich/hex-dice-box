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

    python3 hex_dice_box.py [--kerf=MM] [--pocket=MM] [--layers=N] [--magnets=6|12] [--page=PATH] [--no-write]

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

## STEP file

`step/hex-dice-box.step` holds the box as a STEP file, to open or import in a CAD
program such as SolidCraft, Fusion or FreeCAD: one solid body per plate, in
millimetres, assembled as the 3D view shows it with the lid on. Every face is
flat; the magnet indents, which are engraved rather than cut, and the magnets are
left out. It is written from `index.html` by
[lasermade-tools](https://github.com/Gernreich/lasermade-tools)' `step-export`,
checked out beside this one:

    ~/LaserMadeMusic/GIT/lasermade-tools/target/release/step-export index.html --name=hex-dice-box

Run it after `hex_dice_box.py` whenever the box changes.

## Credits

Made by Michael Gernreich with [Claude](https://www.anthropic.com/claude) (Anthropic), working in Claude Code, which co-wrote the code and the documentation. Commits Claude made carry a `Co-Authored-By: Claude` line.
