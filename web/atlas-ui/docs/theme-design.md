# Atlas product color system

Atlas uses one product identity with independent light, dark and system appearance.
The light composition draws from printed rulebooks: parchment canvas, warm paper
surfaces, oxblood actions and restrained sage links. The dark composition uses an
ordered Warm Umber neutral ramp, oxblood actions and brass links without
flattening the interface into pure black.

The authored anchors live in `src/shared/theme/themeDefinitions.ts`. Ant Design
receives those roles through the root `ConfigProvider`; Atlas CSS consumes the
matching semantic custom properties. Layout, spacing and typography do not change
with appearance.

Components select a semantic role, not a hue. Each appearance may map that role
to a different foreground when contrast requires it: selected search results use
oxblood text on parchment in light appearance and the normal light text on an
oxblood surface in dark appearance. Always consume the complete selection
foreground/background/border set rather than combining parts from other roles.

## Semantic color

HP is independent of the brand palette. Healthy is green, wounded is amber,
critical is red, and temporary HP is sage in light appearance and parchment in
dark appearance. The missing-HP track keeps at least 3:1 contrast with every fill.

Rarity follows familiar game semantics and is visible for every known rarity:
neutral common, lighter copper uncommon, oxblood rare and purple unique. Unknown
rarities remain neutral rather than inheriting a misleading known-rarity color.

Keep supplied light and dark anchors intact. Add new generic component states via
Ant component or global tokens, and add product-specific roles to the theme
definition. Do not introduce raw color literals in feature components or CSS.

Each authored palette defines its brand text and navigation foreground/background
roles. The projection layer passes those roles directly to Ant `Menu` and `Tabs`
component tokens without choosing colors based on appearance; feature CSS must not
recreate those states. The full PF2e Atlas wordmark remains visible at every
supported width.

Filter polarity is semantic rather than a variation of the primary action color.
Included choices use the sage accent role and excluded choices use the error
oxblood role. Both the open picker rows and closed selection chips consume the
same theme-defined foreground, background and border roles.

The primary search filter region does not introduce a separate tinted surface.
Context bars use the content surface in light
appearance and the page background in dark appearance, with rules providing the
separation. The top-level shell carries the distinct page chrome. Selected results use the selected surface,
text and border roles as a set; they must not reuse the primary action foreground
on the selected background.

Table headers use the neutral content surface and separators rather than a brown
fill. The light palette assigns oxblood to active navigation, while the dark
palette assigns brass, so active views remain distinct from inactive labels.

## Validation

Check authored text/background pairs, solid-control foregrounds, HP fills against
their missing-HP track, and rendered Ant tokens. Visual acceptance in both
appearances remains separate from focused contrast tests and the full frontend
gate.
