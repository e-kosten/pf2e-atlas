# Historical snapshot: setting.yaml

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/data/tags/catalog/setting.yaml`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````yaml
tags:
  - id: setting.habitat.aquatic
    label: Aquatic Habitat
    description: Strongly associated with open water, underwater spaces, reefs, rivers, lakes, or flooded environments.
    display:
      group: Setting
      subgroup: Habitat
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for underwater, river, lake, reef, shipwreck, or flooded-scene planning.
        - Aquatic terrain or water access is central to the record's ecology, encounter frame, or site identity.
      does_not_apply_when:
        - The record merely has a swim Speed or water trait but no meaningful aquatic scene association.

  - id: setting.habitat.arctic
    label: Arctic Habitat
    description: Strongly associated with snow, ice, tundra, glaciers, frozen seas, or extreme cold regions.
    display:
      group: Setting
      subgroup: Habitat
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for frozen wilderness, glacier, tundra, or cold-weather encounter prep.
        - The cold-region habitat is a recurring or defining frame, not a one-off environmental option.
      does_not_apply_when:
        - The record only deals cold damage or has cold resistance without a strong arctic setting identity.

  - id: setting.habitat.desert
    label: Desert Habitat
    description: Strongly associated with dunes, arid wastes, badlands, sun-baked ruins, or scarcity-driven desert scenes.
    display:
      group: Setting
      subgroup: Habitat
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record fits desert travel, survival, ruin exploration, or arid wilderness encounter prep.
        - Heat, sand, water scarcity, or desert ecology materially shapes why the record would be retrieved.
      does_not_apply_when:
        - The record is only fire-themed or dry-flavored without a strong desert setting association.

  - id: setting.habitat.forest
    label: Forest Habitat
    description: Strongly associated with forests, jungles, groves, briar-choked wilds, or dense canopy scenes.
    display:
      group: Setting
      subgroup: Habitat
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for forest, jungle, grove, or woodland expedition scenes.
        - Vegetation, canopy, growth, or woodland ecology is central to the record's identity.
      does_not_apply_when:
        - A plant trait or nature theme is present but the record is not meaningfully tied to a forest-like setting.

  - id: setting.habitat.mountain
    label: Mountain Habitat
    description: Strongly associated with cliffs, peaks, passes, caves above the tree line, or rocky heights.
    display:
      group: Setting
      subgroup: Habitat
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for peaks, passes, cliffs, alpine strongholds, or high-altitude scenes.
        - Mountain terrain is a recurring habitat or encounter frame.
      does_not_apply_when:
        - Rocky terrain is incidental or the stronger setting identity is underground, desert, or sky.

  - id: setting.habitat.underground
    label: Underground Habitat
    description: Strongly associated with caves, tunnels, mines, crypts, burrows, or subterranean spaces.
    display:
      group: Setting
      subgroup: Habitat
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for caves, mines, tunnels, burrows, or deep subterranean encounter prep.
        - Subterranean space shapes the record more strongly than generic darkness or enclosed architecture.
      does_not_apply_when:
        - The record can appear indoors or in darkness but has no strong underground association.

  - id: setting.darklands
    label: Darklands
    description: Closely associated with the Darklands, subterranean societies, or deep underworld regions.
    display:
      group: Setting
      subgroup: Regional
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record text, source context, ecology, or culture closely ties it to the Darklands.
        - The retrieval value is the civilization-bearing underworld macro-region, not generic cave terrain.
      does_not_apply_when:
        - The record merely can appear underground without specific Darklands association.

  - id: setting.planar.first_world
    label: First World
    description: Strongly associated with the First World, fey realms, or thin-boundary crossings into that plane.
    display:
      group: Setting
      subgroup: Planar
    applicability:
      any_of:
        - record_kinds: [creature, hazard, lore]
    guidance:
      applies_when:
        - The record is naturally retrieved for First World scenes, fey realms, or planar bleed from fey wilderness.
        - Planar fey context matters more than a generic fey trait.
      does_not_apply_when:
        - The record is merely whimsical, primal, or fey-adjacent without a strong First World association.
````
