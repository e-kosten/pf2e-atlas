import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const afflictionMetaphysicalProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.METAPHYSICAL_METAPHYSICAL_PROFILE, {
    curse_marking: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Brands the victim with a curse mark, doom sign, inherited hex, or similarly explicit supernatural stigma.",
      appliesWhen: [
        "The affliction leaves an explicit mark, sign, sigil, stain, or named curse-brand that matters to its identity.",
        "A user would retrieve it for visible or narratively explicit cursed marking, not just soul damage.",
      ],
      doesNotApplyWhen: [
        "The affliction is metaphysical but has no distinct branded or marked stigma.",
        "The stronger fit is void_soul_corruption or soul_binding without a visible curse sign.",
      ],
      adjacentTags: ["void_soul_corruption", "soul_binding"],
    },
    nightmare_torment: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Centers on nightmares, dream-torment, or similarly sleep-haunting affliction framing.",
    },
    possession_seed: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Plants an invading spirit, hostile presence, or takeover-ready metaphysical foothold inside the victim.",
      appliesWhen: [
        "The affliction prepares a victim to be ridden, entered, replaced, or overtaken by another presence.",
        "A latent invading spirit or takeover-ready foothold is central to the condition's danger.",
      ],
      doesNotApplyWhen: [
        "The affliction only compels behavior without an actual possessing entity or metaphysical foothold.",
        "The stronger fit is soul_binding or compulsion because takeover is not really part of the disease model.",
      ],
      adjacentTags: ["compulsion", "soul_binding"],
    },
    soul_binding: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Pins, traps, anchors, or otherwise entangles the victim's soul with an object, oath, place, or hostile power.",
      appliesWhen: [
        "The affliction's metaphysical identity depends on the victim's soul being anchored, trapped, pledged, or externally entangled.",
        "A user would retrieve it for ghost anchors, cursed bindings, oath-linked doom, or similar soul-tether effects.",
      ],
      doesNotApplyWhen: [
        "The affliction only marks, weakens, or spiritually corrupts the victim without actually binding the soul to something.",
        "The stronger fit is curse_marking or possession_seed rather than tethering or anchoring.",
      ],
      adjacentTags: ["curse_marking", "possession_seed"],
    },
    void_soul_corruption: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.DETERMINISTIC,
      description: "Attacks life force or the bond between body and soul through void or deathly corruption.",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"affliction">[];
