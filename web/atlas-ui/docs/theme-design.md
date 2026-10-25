# Atlas product color system

Atlas uses one product identity with independent light, dark and system appearance.
The light composition draws from printed rulebooks: parchment canvas, warm paper
surfaces, oxblood actions and restrained sage links. The dark composition uses an
ordered warm-neutral ramp, oxblood actions and brass links without flattening the
interface into pure black. Its current default combines the Ink neutral ramp
with the Bright text palette.

The authored anchors live in `src/shared/theme/themeDefinitions.ts`. Ant Design
receives those roles through the root `ConfigProvider`; Atlas CSS consumes the
matching semantic custom properties. Layout, spacing and typography do not change
with appearance.

## Authored dark variants

The product default is **Ink + Bright**. These values are semantic anchors; they
are projected through `atlasTheme.ts` rather than used directly by features.

| Ink neutral role | Value     |
| ---------------- | --------- |
| Shell            | `#090908` |
| Background       | `#0E0D0C` |
| Surface          | `#151311` |
| Elevated         | `#1B1815` |
| Subtle           | `#221D19` |
| Separator        | `#342C25` |
| Border           | `#705F50` |

| Bright text role | Value     |
| ---------------- | --------- |
| Primary text     | `#F7F1E7` |
| Muted text       | `#D0C6B7` |

The Theme popover exposes the authored neutral ramps and text palettes as
session-scoped comparison controls. They do not persist as additional supported
product themes.

Retain the following earlier anchors as exact historical references. The Warm
text palette remains an authored comparison option. Prior Standard Umber records
the original neutral ramp; its border predates the current 3:1 control-boundary
requirement, and selecting the current Standard comparison does not recreate the
entire earlier dark appearance. Exploratory palettes do not need permanent
documentation unless they are deliberately accepted for possible reuse.

| Standard Umber neutral role | Value     |
| --------------------------- | --------- |
| Shell                       | `#111110` |
| Background                  | `#171615` |
| Surface                     | `#211F1C` |
| Elevated                    | `#28241F` |
| Subtle                      | `#302A23` |
| Separator                   | `#4A4136` |
| Border                      | `#5C5144` |

| Warm text role | Value     |
| -------------- | --------- |
| Primary text   | `#F3E9D7` |
| Muted text     | `#C4B6A1` |

The current Standard comparison keeps the same first six neutral anchors but
uses `#776858` for its control border. When changing the product default, update
the corresponding `DEFAULT_*` constant in `themeDefinitions.ts` and this section
together. Keep neutral ramps independent from text palettes so either retained
direction can be evaluated without changing semantic accents such as brass,
rarity, status or HP colors.

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
Context bars use the content surface in both appearances, with rules providing
separation from the workbench below. The shared surface keeps the context region
and panes visually continuous; the top-level shell carries the distinct page
chrome. Selected results use the selected surface, text and border roles as a
set; they must not reuse the primary action foreground on the selected
background.

Table headers use the neutral content surface and separators rather than a brown
fill. The light palette assigns oxblood to active navigation, while the dark
palette assigns brass, so active views remain distinct from inactive labels.

## Brand and typography

The header pairs the full `PF2e Atlas` wordmark in bundled Lora with the shared
sans-serif interface typography. `--wordmark-font` owns that narrow brand role;
do not apply it to navigation, headings, controls or reference content. The
wordmark remains visible at every supported width.

The shared header mark is an inline, theme-driven SVG. Its ring compass uses the
approved tucked-under secondary points around the restrained faceted d20. Every
fill and stroke comes from the palette's `brand.mark` roles. Browser-tab icons
use simplified light and dark SVGs without the smallest points and ticks, which
would become visual noise at favicon sizes. The resolved application appearance
selects the matching favicon.

Pathfinder action glyphs in reference content use the surrounding body-text
color. Glyphs inside solid encounter resource controls inherit the control
foreground, so their contrast follows the Ant `primary`/`onPrimary` token pair.

## Validation

Check authored text/background pairs, solid-control foregrounds, HP fills against
their missing-HP track, and rendered Ant tokens. Review the full header mark and
simplified favicon at their actual sizes. Visual acceptance in both appearances
remains separate from focused contrast tests and the full frontend gate.
