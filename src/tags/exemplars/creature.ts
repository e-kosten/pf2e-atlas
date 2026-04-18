import type { DerivedTagExemplarCategory } from "../../types.js";

export const CREATURE_DERIVED_TAG_EXEMPLARS = {
  category: "creature",
  exemplars: [
    {
      tag: "urban_setting",
      positives: [
        {
          name: "Conspirator Dragon (Adult)",
          recordKey: "pathfinder-monster-core:TGYELuImcTcuX0aH",
        },
        {
          name: "Spiritbound Aluum",
          recordKey: "age-of-ashes-bestiary:n6FQeNsDgKaDIF7b",
        },
      ],
      negatives: [],
    },
  ],
} satisfies DerivedTagExemplarCategory;
