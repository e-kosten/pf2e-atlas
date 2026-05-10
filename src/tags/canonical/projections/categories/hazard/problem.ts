import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";

export const hazardProblemProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.PROBLEM_PROBLEM_SHAPE, {
    endurance_pressure: {
      description:
        "Hazard whose main prep problem is surviving repeated exposure long enough to finish the scene rather than landing one clean solve immediately.",
    },
    layered_resolution: {
      concept: "multi_stage_resolution",
      description:
        "Hazard that asks the party to solve multiple linked pieces rather than one single disable check or obvious answer.",
    },
    observation_first: {
      concept: "observation_driven",
      description:
        "Hazard that rewards careful watching, clue gathering, or reading the environment before a safe approach becomes obvious.",
    },
    source_tracing: {
      concept: "source_discovery",
      description:
        "Hazard whose real puzzle is locating the hidden anchor, leak point, cursed source, contamination engine, or origin node before a clean solution becomes possible.",
      appliesWhen: [
        "The hazard is naturally retrieved because identifying the source object, origin point, or contamination engine is a major part of solving it.",
        "Finding what is powering the danger matters more than merely surviving exposure or executing a known disable procedure.",
      ],
      doesNotApplyWhen: [
        "The hazard is fully understandable up front and the real challenge is timing, endurance, or multi-step execution rather than finding an origin.",
        "The stronger fit is source_cleanup_countermeasure because the source is already obvious and the remaining task is neutralizing it.",
      ],
      adjacentTags: ["observation_first", "source_cleanup_countermeasure"],
    },
    timing_window: {
      description:
        "Hazard that is best handled by acting during the right cycle, opening, lull, or repeating timing pattern.",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"hazard">[];
