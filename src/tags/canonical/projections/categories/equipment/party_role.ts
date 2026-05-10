import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";
import { CANONICAL_VOCABULARY } from "../../../vocabulary.js";

export const equipmentPartyRoleProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.PARTY_ROLE_PARTY_ROLE, {
    caster_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to spellcasters for magical prep, casting reliability, spell defense, or spell-adjacent utility.",
    },
    companion_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to handlers of animal companions, mounts, familiars, or other creature-side support play patterns.",
    },
    defender_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description: "Especially valuable to shield users, bodyguards, line-holders, or other defender-style characters.",
    },
    emergency_recovery: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable as a panic-button item for sudden healing, escape, stabilization, or critical condition rescue.",
    },
    face_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to negotiators, deceivers, diplomats, or other socially forward characters who need influence, disguise, or presentation support.",
    },
    healer_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to medics, battlefield healers, or builds expected to stabilize, treat, or recover allies under pressure.",
    },
    ranged_striker_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to archers, gunners, or other builds that pressure from range through repeated attacks or reload workflows.",
    },
    scout_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to scouts, infiltrators, or advance-party play through quiet entry, recon, and information-gathering support.",
    },
    skirmisher_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to mobile flankers, hit-and-run melee characters, or other skirmisher-style builds.",
    },
  }),
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.EQUIPMENT.PARTY_ROLE_PLAY_PATTERN, {
    action_economy_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable because it compresses setup, speeds access, or meaningfully improves in-combat action efficiency.",
    },
    combat_maneuver_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to builds that rely on grappling, tripping, shoving, disarming, or other combat maneuvers to control enemies.",
    },
    companion_handling_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable when a character's tactical workflow depends on commanding, outfitting, transporting, or protecting companions and mounts.",
    },
    focus_magic_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to focus-spell-heavy or magic-routine-driven builds that want more reliable magical cadence and recovery.",
    },
    reload_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to ranged builds whose action flow depends on reloading efficiently or keeping ammunition ready.",
    },
    shield_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to shield-forward play through improved blocking, readiness, or defensive shield workflow.",
    },
    stealth_entry_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to quiet-entry, infiltration, burglary, or reconnaissance play that depends on avoiding notice and solving access problems discreetly.",
    },
    thrown_weapon_support: {
      assignmentMode: CANONICAL_VOCABULARY.ASSIGNMENT.MODE.HYBRID,
      description:
        "Especially valuable to builds that solve problems through thrown weapons, quick draws, or repeatable thrown-item pressure.",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"equipment">[];
