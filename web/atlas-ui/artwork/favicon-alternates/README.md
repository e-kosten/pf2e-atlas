# Favicon artwork

The active favicon has separate
[light](../../public/favicon-light.svg) and
[dark](../../public/favicon-dark.svg) variants. Both use the simplified small-size
form of the ring compass and d20 mark; the application header renders the full
eight-point tucked-under treatment from the shared `AtlasMark` component.

These matching parchment + oxblood alternatives are retained for future selection:

- [Ornate spellbook · compass rose](ornate-spellbook-compass-parchment-oxblood.svg):
  a vector reconstruction of spellbook concept 12, with a compass cover emblem,
  metal corners and clasp, without a bookmark.
- [Detailed reference](detailed-compass-rose-parchment-oxblood.svg): the detailed
  faceted compass rose, locally traced from the supplied PNG and recolored as SVG.

The alternatives are not imported by the app and are outside `public` and `src`,
so they are not included in the web build. The retained designs use SVG geometry;
no bitmap is embedded.

To choose an alternative later, create matching light and dark production assets
and update the shared header mark rather than replacing only one appearance.
