import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const afflictionResponseProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.RESPONSE_RESOLUTION_PROFILE, {
    antidote_resolution: {
      concept: "poison_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because antitoxins, antidotes, neutralizing medicine, or poison-specific treatment are central to solving it.",
      adjacentTags: ["countermagic_resolution", "cure_clock_urgency"],
    },
    countermagic_resolution: {
      concept: "active_magic_counteraction",
      label: "countermagic",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because counteracting, suppressing, or dispelling an active magical affliction is central to solving it.",
      adjacentTags: ["antidote_resolution", "cursebreaking_resolution"],
    },
    cursebreaking_resolution: {
      concept: "curse_remediation",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because lifting a curse, breaking a doom, or ending a supernatural binding is central to recovery.",
      adjacentTags: ["countermagic_resolution", "ritual_appeasement_resolution"],
    },
    exorcism_resolution: {
      concept: "hostile_presence_expulsion",
      label: "exorcism",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because banishing, cleansing, or spiritually expelling a hostile presence is central to solving the affliction.",
      adjacentTags: ["cursebreaking_resolution", "ritual_appeasement_resolution"],
    },
    quarantine_containment_resolution: {
      concept: "outbreak_containment",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because isolation, contact control, and containment are core to preventing further spread while treatment proceeds.",
      adjacentTags: ["outbreak_management", "quarantine_risk"],
    },
    ritual_appeasement_resolution: {
      concept: "ritual_appeasement",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because restitution, offerings, ritual respect, or meeting a spiritual demand is central to ending the affliction.",
      adjacentTags: ["cursebreaking_resolution", "exorcism_resolution"],
    },
    source_cleanup_resolution: {
      concept: "source_cleanup",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because the contaminated site, cursed source, infected carrier chain, or environmental origin must be found and cleaned up.",
      adjacentTags: ["source_tracing", "quarantine_containment_resolution"],
    },
    surgical_extraction_resolution: {
      concept: "parasite_removal",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because removing eggs, larvae, parasites, implants, or invasive growth from the body is a central answer path.",
      adjacentTags: ["source_cleanup_resolution", "infestation_implant"],
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.AFFLICTION.RESPONSE_RESPONSE_PROFILE, {
    cure_clock_urgency: {
      concept: "time_critical_resolution",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates immediate pressure to diagnose and cure the affliction before a fast-moving catastrophic endpoint arrives.",
      adjacentTags: ["terminal_collapse", "delayed_onset"],
    },
    outbreak_management: {
      concept: "outbreak_containment",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved as a disease or curse that creates a wider containment, treatment, and community-management problem rather than only an isolated victim.",
      adjacentTags: ["community_outbreak", "quarantine_risk"],
    },
    quarantine_risk: {
      concept: "outbreak_containment",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Creates a strong need to isolate victims, restrict contact, or manage who can safely enter or leave an affected area.",
      adjacentTags: ["community_outbreak", "inhaled_exposure"],
    },
    source_tracing: {
      concept: "source_discovery",
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Naturally retrieved because finding the contaminated source, carrier chain, cursed origin, or initial spread event is central to solving the problem.",
      adjacentTags: ["carrier_vector", "community_outbreak"],
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"affliction">[];
