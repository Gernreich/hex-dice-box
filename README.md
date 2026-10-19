# Hexagonal dice box

**[Read the writeup](https://gernreich.github.io/hex-dice-box/)**: spin the box
in 3D, with the cut sheet and the assembly steps on the same page.

A laser-cut dice box for 3mm ply on the xTool P2S (600 x 305mm bed). Six
finger-jointed walls sit on a hexagonal floor. Inside, a stack of glued
insert layers makes seven hexagonal pockets, six round one in the middle,
for a polyhedral dice set. The lid drops into the top and is held by 2 x 1mm
neodymium discs glued into rastered indents.

That page is `index.html`, published by GitHub Pages; opened locally it
works too, given a network connection for the fonts and the 3D library.

Nothing has been cut yet.

## The design

| | |
|---|---|
| Box | 97.3mm across the flats, 112.4 across the corners, 27 tall with the lid |
| Pockets | 7 x 25mm across the flats, 21mm deep (6 insert layers + the lid plug) |
| Joints | floor edges 9 fingers of 6.24mm, wall edges 4 of 6.00mm |
| Magnets | 12 pairs of 2 x 1mm discs, 24 discs in all |
| Indents | 2.2mm across, 1.1mm deep |

- **Box.** All six walls are the same part, plain edge up. Walls meet at
  120 degrees: in the other wall's cells a wall stops T sin(120) = 2.6mm short
  of the edge, the least that keeps the two slabs apart. A small V shows
  outside each corner, and inside the joint closes on a line with a
  0.87 x 1.5mm groove. A coat of shellac fills both.
- **Inserts.** Each layer is the box's inside less 0.15mm all round, with
  the seven pockets cut out. They're glued in a stack on the floor. The top
  layer carries the box's magnet indents, facing up, and ends 3mm below the
  wall tops.
- **Lid.** A plain hexagon the size of the box's outside, sitting on the
  wall tops, with a plug glued under it. The plug is pocketed like an insert
  and has 0.3mm clearance each side. Its indents face down and line up with
  the top insert's, so each pair of discs meets face to face.
- **Magnets.** Six at the outer corners and six where the middle pocket
  meets two outer ones. `--magnets=6` keeps only the corners.

## Cutting

One sheet, `cut-files/hex-dice-box.svg`, 1 user unit = 1mm:

1. **Blue**: fill engrave, the magnet indents. Set power and passes on
   scrap until a 1mm disc sits just below the face.
2. **Orange**: cut, the pockets.
3. **Black**: cut, frees the parts.

## Assembly

1. Dry-fit the six walls into the floor, then glue them and give the box a
   coat of shellac.
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

- `--kerf=` is the laser's real kerf (default 0.13mm, measured). Every
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
