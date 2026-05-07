import { afterEach, describe, expect, it } from "vitest";

import { loadTestService } from "../helpers/pf2e-fixture.js";
import { cleanupCreatedRoots, createFixture, createHardFilterFixture } from "../helpers/pf2e-service-fixture.js";

function anyString(): unknown {
  return expect.any(String);
}

function arrayContaining(values: unknown[]): unknown {
  return expect.arrayContaining(values);
}

function objectContaining(value: Record<string, unknown>): unknown {
  return expect.objectContaining(value);
}

function stringContaining(value: string): unknown {
  return expect.stringContaining(value);
}

describe("Pf2eDataService / Rules and Vocabulary", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("supports batch lookup, record fetch, and unified rule graph retrieval", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(service.lookup("Refocus")).toEqual(
      expect.objectContaining({
        matchType: "exact",
        match: expect.objectContaining({ name: "Refocus" }),
      }),
    );

    const lookups = service.lookupMany([{ name: "Refocus" }, { name: "Deep Focus" }], { coreOnly: true });
    expect(lookups.map((result) => result.match?.name)).toEqual(["Refocus", "Deep Focus"]);
    expect(lookups.map((result) => result.matchType)).toEqual(["exact", "exact"]);

    const records = service.getRecordsByKeys(["actions:action-refocus-1", "feats-srd:feat1"]);
    expect(records.map((record) => record.name)).toEqual(["Refocus", "Deep Focus"]);

    const defaultGraph = service.getRuleGraph(["conditionitems:Blinded"], { maxOutgoingPerPrimary: 5 });
    expect(defaultGraph.outgoing.records.map((record) => record.name)).toEqual(["Dazzled", "Seek"]);
    expect(defaultGraph.backlinks.records).toHaveLength(0);
    expect(defaultGraph.edges.every((edge) => edge.direction === "outgoing")).toBe(true);

    const combinedGraph = service.getRuleGraph(["feats-srd:feat1", "actions:action-refocus-1"], {
      includeOutgoing: true,
      includeBacklinks: true,
      maxOutgoingPerPrimary: 5,
      maxBacklinksPerPrimary: 10,
    });
    expect(combinedGraph.outgoing.records.map((record) => record.name)).toEqual(["Refocus"]);
    expect(combinedGraph.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(combinedGraph.edges).toHaveLength(3);

    const backlinksOnly = service.getRuleGraph(["actions:action-refocus-1"], {
      includeOutgoing: false,
      includeBacklinks: true,
      maxBacklinksPerPrimary: 10,
    });
    expect(backlinksOnly.outgoing.records).toHaveLength(0);
    expect(backlinksOnly.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(backlinksOnly.backlinks.edges.every((edge) => edge.direction === "backlink")).toBe(true);
    expect(backlinksOnly.backlinks.records.some((record) => record.type === "spell")).toBe(false);

    const emptyGraph = service.getRuleGraph(["actions:action-refocus-1"], {
      includeOutgoing: false,
      includeBacklinks: false,
    });
    expect(emptyGraph.outgoing.records).toHaveLength(0);
    expect(emptyGraph.backlinks.records).toHaveLength(0);
    expect(emptyGraph.edges).toHaveLength(0);
  });

  it("exposes generic incoming reference edges without the curated rule-graph backlink allowlist", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);
    const refocus = service.lookup("Refocus").match;

    expect(refocus).toBeTruthy();

    const relations = service.getReferenceEdges([refocus!.recordKey], {
      includeOutgoing: false,
      includeIncoming: true,
    });

    expect(relations.outgoing.records).toHaveLength(0);
    expect(relations.incoming.records.map((record) => record.name)).toEqual(
      expect.arrayContaining(["Deep Focus", "Meditative Well", "Focus Burst"]),
    );
    expect(relations.incoming.records.some((record) => record.category === "spell")).toBe(true);

    const curatedBacklinks = service.getRuleGraph([refocus!.recordKey], {
      includeOutgoing: false,
      includeBacklinks: true,
      maxBacklinksPerPrimary: 10,
    });
    expect(curatedBacklinks.backlinks.records.map((record) => record.name)).not.toContain("Focus Burst");
  });

  it("exposes indexed search vocabulary for agent planning", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const vocabulary = service.getSearchVocabulary({ traitLimitPerCategory: 4 });
    expect(vocabulary.categories.map((entry) => entry.value)).toEqual(
      arrayContaining(["creature", "spell", "rule", "feat"]),
    );
    expect(vocabulary.subcategories.map((entry) => entry.value)).toEqual(
      arrayContaining(["condition", "action", "trap"]),
    );
    expect(vocabulary.subcategories.map((entry) => entry.value)).not.toContain("primal");
    expect(vocabulary.rarities.map((entry) => entry.value)).toEqual(
      arrayContaining(["common", "uncommon", "rare", "unique"]),
    );
    expect(vocabulary.sizes.map((entry) => entry.value)).toEqual(arrayContaining(["med", "sm", "lg"]));
    expect(vocabulary.traditions.map((entry) => entry.value)).toContain("primal");
    expect(vocabulary.spellKinds.map((entry) => entry.value)).toContain("focus");
    expect(
      vocabulary.commonTraitsByCategory.find((entry) => entry.category === "creature")?.traits.length,
    ).toBeGreaterThan(0);
    expect(
      vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "equipment")?.tags.length,
    ).toBeGreaterThan(0);
    expect(
      vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "spell")?.tags.length,
    ).toBeGreaterThan(0);
    expect(
      vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "hazard")?.tags.length,
    ).toBeGreaterThan(0);
    expect(
      vocabulary.commonDerivedTagsByCategory.find((entry) => entry.category === "affliction")?.tags.length,
    ).toBeGreaterThan(0);
    expect(vocabulary.derivedTagOntologyFamilies).toEqual(
      arrayContaining([
        objectContaining({
          category: "creature",
          family: "regional_setting",
          axis: "setting",
          description: stringContaining("broad Golarion macro-regions"),
        }),
        objectContaining({
          category: "creature",
          family: "habitat_setting",
          axis: "setting",
          description: stringContaining("habitat tags"),
        }),
        objectContaining({
          category: "hazard",
          family: "problem_shape",
          axis: "problem",
          description: stringContaining("investigation, timing"),
        }),
        objectContaining({
          category: "spell",
          family: "control",
          axis: "battlefield",
          description: stringContaining("denying movement"),
        }),
        objectContaining({
          category: "spell",
          family: "consultation",
          axis: "utility",
          description: stringContaining("ask for guidance"),
        }),
        objectContaining({
          category: "spell",
          family: "revelation",
          axis: "utility",
          description: stringContaining("Detection and revelation"),
        }),
        objectContaining({
          category: "equipment",
          family: "movement_traversal",
          axis: "utility",
          description: stringContaining("move, climb, navigate"),
        }),
        objectContaining({
          category: "affliction",
          family: "response_profile",
          axis: "response",
          description: stringContaining("response problem"),
        }),
        objectContaining({
          category: "affliction",
          family: "resolution_profile",
          axis: "response",
          description: stringContaining("remedies"),
        }),
        objectContaining({
          category: "equipment",
          family: "play_pattern",
          axis: "party_role",
          description: stringContaining("play-pattern-facing"),
        }),
        objectContaining({
          category: "equipment",
          family: "resolution",
          axis: "utility",
          description: stringContaining("break curses"),
        }),
        objectContaining({
          category: "creature",
          family: "corruption_profile",
          axis: "specialization",
          description: stringContaining("corruption and taint"),
        }),
        objectContaining({
          category: "spell",
          family: "resolution",
          axis: "utility",
          description: stringContaining("solve curses"),
        }),
        objectContaining({
          category: "spell",
          family: "expedition",
          axis: "utility",
          description: stringContaining("travel"),
        }),
      ]),
    );
    expect(vocabulary.derivedTagOntologyTags).toEqual(
      arrayContaining([
        objectContaining({
          category: "creature",
          family: "site_setting",
          tag: "urban_setting",
          assignmentMode: "editorial",
        }),
        objectContaining({
          category: "spell",
          family: "infiltration",
          tag: "infiltration",
          assignmentMode: "composite",
        }),
        objectContaining({
          category: "spell",
          family: "consultation",
          tag: "consultation",
          assignmentMode: "composite",
        }),
        objectContaining({
          category: "spell",
          family: "communication",
          tag: "telepathic_communication",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "spell",
          family: "communication",
          tag: "communication",
          assignmentMode: "composite",
        }),
        objectContaining({
          category: "spell",
          family: "expedition",
          tag: "expedition",
          assignmentMode: "composite",
        }),
        objectContaining({
          category: "spell",
          family: "support",
          tag: "anti_poison",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "spell",
          family: "resolution",
          tag: "ritual_appeasement",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "spell",
          family: "revelation",
          tag: "truth_reveal",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "spell",
          family: "revelation",
          tag: "revelation",
          assignmentMode: "composite",
        }),
        objectContaining({
          category: "spell",
          family: "control",
          tag: "countermagic",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "equipment",
          family: "access_bypass",
          tag: "barrier_bypass",
          assignmentMode: "deterministic",
        }),
        objectContaining({
          category: "equipment",
          family: "party_role",
          tag: "defender_support",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "equipment",
          family: "party_role",
          tag: "scout_support",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "equipment",
          family: "play_pattern",
          tag: "shield_support",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "equipment",
          family: "resolution",
          tag: "resolution",
          assignmentMode: "composite",
        }),
        objectContaining({
          category: "equipment",
          family: "resolution",
          tag: "source_revelation",
          assignmentMode: "deterministic",
        }),
        objectContaining({
          category: "equipment",
          family: "resolution",
          tag: "ritual_appeasement",
          assignmentMode: "deterministic",
        }),
        objectContaining({
          category: "hazard",
          family: "countermeasure_profile",
          tag: "contamination_cleanup_countermeasure",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "hazard",
          family: "problem_shape",
          tag: "source_tracing",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "hazard",
          family: "function",
          tag: "sentinel_guardian",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "affliction",
          family: "response_profile",
          tag: "outbreak_management",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "affliction",
          family: "resolution_profile",
          tag: "cursebreaking_resolution",
          assignmentMode: "hybrid",
        }),
        objectContaining({
          category: "creature",
          family: "corruption_profile",
          tag: "fungal_infested",
          assignmentMode: "hybrid",
        }),
      ]),
    );
    expect(vocabulary.derivedTagCatalog).toEqual(
      arrayContaining([
        objectContaining({
          category: "equipment",
          family: "movement_traversal",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "navigation", description: anyString() }),
            objectContaining({ value: "transport", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "party_role",
          axis: "party_role",
          tags: arrayContaining([
            objectContaining({ value: "defender_support", description: anyString() }),
            objectContaining({ value: "scout_support", description: anyString() }),
            objectContaining({ value: "face_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "play_pattern",
          axis: "party_role",
          tags: arrayContaining([
            objectContaining({ value: "shield_support", description: anyString() }),
            objectContaining({ value: "action_economy_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "equipment",
          family: "infiltration",
          axis: "utility",
          tags: arrayContaining([objectContaining({ value: "stealth_support", description: anyString() })]),
        }),
        objectContaining({
          category: "equipment",
          family: "resolution",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "resolution", assignmentMode: "composite" }),
            objectContaining({ value: "ritual_appeasement", description: anyString() }),
            objectContaining({ value: "sanctification", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "infiltration",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "stealth_support", description: anyString() }),
            objectContaining({ value: "infiltration", assignmentMode: "composite" }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "consultation",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "consultation", assignmentMode: "composite" }),
            objectContaining({ value: "omen_guidance", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "communication",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "communication", assignmentMode: "composite" }),
            objectContaining({ value: "telepathic_communication", description: anyString() }),
            objectContaining({ value: "message_delivery", description: anyString() }),
            objectContaining({ value: "translation_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "expedition",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "expedition", assignmentMode: "composite" }),
            objectContaining({ value: "field_shelter", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "resolution",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "resolution", assignmentMode: "composite" }),
            objectContaining({ value: "ritual_appeasement", description: anyString() }),
            objectContaining({ value: "source_revelation", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "security",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "alarm", description: anyString() }),
            objectContaining({ value: "security", assignmentMode: "composite" }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "revelation",
          axis: "utility",
          tags: arrayContaining([
            objectContaining({ value: "truth_reveal", description: anyString() }),
            objectContaining({ value: "revelation", assignmentMode: "composite" }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "support",
          axis: "support",
          tags: arrayContaining([
            objectContaining({ value: "anti_poison", description: anyString() }),
            objectContaining({ value: "anti_disease", description: anyString() }),
            objectContaining({ value: "quickened_support", description: anyString() }),
            objectContaining({ value: "eidolon_support", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "spell",
          family: "impact",
          axis: "effect",
          tags: arrayContaining([
            objectContaining({ value: "persistent_damage", description: anyString() }),
            objectContaining({ value: "burst_damage", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "function",
          axis: "encounter",
          tags: arrayContaining([
            objectContaining({ value: "zone_denial", description: anyString() }),
            objectContaining({ value: "sentinel_guardian", description: anyString() }),
            objectContaining({ value: "forced_separation_hazard", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "countermeasure_profile",
          axis: "resolution",
          tags: arrayContaining([
            objectContaining({
              value: "quarantine_containment_countermeasure",
              description: anyString(),
            }),
            objectContaining({ value: "contamination_cleanup_countermeasure", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "hazard",
          family: "problem_shape",
          axis: "problem",
          tags: arrayContaining([
            objectContaining({ value: "observation_first", description: anyString() }),
            objectContaining({ value: "source_tracing", description: anyString() }),
            objectContaining({ value: "layered_resolution", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "affliction",
          family: "response_profile",
          axis: "response",
          tags: arrayContaining([
            objectContaining({ value: "outbreak_management", description: anyString() }),
            objectContaining({ value: "cure_clock_urgency", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "affliction",
          family: "resolution_profile",
          axis: "response",
          tags: arrayContaining([
            objectContaining({ value: "countermagic_resolution", description: anyString() }),
            objectContaining({ value: "quarantine_containment_resolution", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "planar_setting",
          axis: "setting",
          tags: arrayContaining([
            objectContaining({ value: "astral_setting", description: anyString() }),
            objectContaining({ value: "upper_plane_setting", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "scene_role",
          axis: "npc_role",
          tags: arrayContaining([
            objectContaining({ value: "enforcer_npc", description: anyString() }),
            objectContaining({ value: "infiltrator_npc", description: anyString() }),
            objectContaining({ value: "guardian_npc", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "threat_profile",
          axis: "encounter",
          tags: arrayContaining([
            objectContaining({ value: "prey_control_threat", description: anyString() }),
            objectContaining({ value: "reinforcement_threat", description: anyString() }),
            objectContaining({ value: "infiltration_threat", description: anyString() }),
          ]),
        }),
        objectContaining({
          category: "creature",
          family: "corruption_profile",
          axis: "specialization",
          tags: arrayContaining([
            objectContaining({ value: "blight_tainted", description: anyString() }),
            objectContaining({ value: "void_tainted", description: anyString() }),
          ]),
        }),
      ]),
    );
  });

  it("lists live filter values across supported fields and scopes", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(
      service.listFilterValues({
        field: "traits",
        category: "hazard",
        subcategory: "trap",
      }),
    ).toEqual({
      field: "traits",
      values: [
        { value: "trap", count: 16 },
        { value: "magical", count: 9 },
        { value: "mechanical", count: 8 },
      ],
    });

    expect(
      service
        .listFilterValues({
          field: "derivedTags",
          category: "equipment",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining([
        "alarm",
        "beneficial",
        "offensive",
        "climbing",
        "lock_bypass",
        "mental_recovery",
        "anti_fear",
        "anti_confusion",
        "anti_paralysis",
        "anti_petrification",
        "anti_bleed",
        "curse_removal",
        "carry_support",
        "tracking",
        "anti_tracking",
        "restraint_escape",
        "restraint_capture",
        "mobility_impairment",
        "sensory_impairment",
        "mental_impairment",
        "sedation",
        "spell_payload",
      ]),
    );
    expect(
      service
        .listFilterValues({
          field: "derivedTags",
          category: "spell",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining([
        "alarm",
        "countermagic",
        "disguise",
        "eidolon_support",
        "initiative_support",
        "illumination",
        "mobility",
        "navigation",
        "persistent_damage",
        "reconnaissance",
        "senses_support",
        "security",
        "social_infiltration",
        "wayfinding",
        "mental_impairment",
        "sensory_impairment",
        "forced_movement",
        "restraint_capture",
        "anti_fear",
        "anti_confusion",
        "anti_paralysis",
        "anti_petrification",
        "anti_bleed",
        "curse_removal",
      ]),
    );
    expect(
      service
        .listFilterValues({
          field: "derivedTags",
          category: "hazard",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining([
        "alarm",
        "mobility_impairment",
        "restraint_capture",
        "respiratory_hazard",
        "water_hazard",
        "spawned_attackers",
        "navigation_disruption",
        "illusion_assault",
        "overhead_strike",
      ]),
    );
    expect(
      service
        .listFilterValues({
          field: "derivedTags",
          category: "affliction",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining([
        "healing_suppression",
        "mental_impairment",
        "mobility_impairment",
        "sedation",
        "respiratory_impairment",
        "transformative_corruption",
        "rot_decay",
        "infestation_implant",
        "compulsion",
      ]),
    );
    expect(
      service
        .listFilterValues({
          field: "derivedTags",
          category: "creature",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining([
        "aquatic_setting",
        "freshwater_setting",
        "coastal_setting",
        "astral_setting",
        "ethereal_setting",
        "plane_of_fire_setting",
        "plane_of_air_setting",
        "plane_of_water_setting",
        "plane_of_earth_setting",
        "elemental_plane_setting",
        "first_world_setting",
        "boneyard_setting",
        "heaven_setting",
        "nirvana_setting",
        "elysium_setting",
        "upper_plane_setting",
        "hell_setting",
        "abyss_setting",
        "abaddon_setting",
        "lower_plane_setting",
        "axis_setting",
        "shadow_plane_setting",
        "maelstrom_setting",
        "cosmic_framework_setting",
        "island_setting",
        "jungle_setting",
        "sky_setting",
        "temple_setting",
        "small_settlement_setting",
        "rural_setting",
        "civic_npc",
        "enforcer_npc",
        "carnival_show",
        "living_toy",
        "living_artwork",
        "trickster_mischief",
      ]),
    );

    expect(
      service.listFilterValues({
        field: "families",
        category: "creature",
      }),
    ).toEqual({
      field: "families",
      values: [
        { value: "ghost", count: 1 },
        { value: "lich", count: 1 },
        { value: "mythic", count: 1 },
        { value: "seafarer", count: 1 },
        { value: "vampire", count: 1 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "subcategories",
        category: "hazard",
      }),
    ).toEqual({
      field: "subcategories",
      values: [
        { value: "trap", count: 16 },
        { value: "haunt", count: 3 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "publicationTitle",
        category: "spell",
      }),
    ).toEqual({
      field: "publicationTitle",
      values: [
        { value: "Pathfinder Player Core", count: 18 },
        { value: "Pathfinder Dark Archive", count: 6 },
        { value: "Pathfinder Secrets of Magic", count: 3 },
        { value: "Pathfinder Rage of Elements", count: 1 },
      ],
    });

    expect(
      service
        .listFilterValues({
          field: "sourceCategory",
          category: "creature",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["core", "rules", "adventure"]);

    expect(
      service.listFilterValues({
        field: "traditions",
        category: "spell",
      }),
    ).toEqual({
      field: "traditions",
      values: [
        { value: "arcane", count: 17 },
        { value: "occult", count: 17 },
        { value: "divine", count: 11 },
        { value: "primal", count: 10 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "spellKinds",
        category: "spell",
      }),
    ).toEqual({
      field: "spellKinds",
      values: [
        { value: "cantrip", count: 1 },
        { value: "focus", count: 1 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "saveType",
        category: "spell",
      }),
    ).toEqual({
      field: "saveType",
      values: [
        { value: "fortitude", count: 1 },
        { value: "reflex", count: 1 },
        { value: "will", count: 1 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "areaType",
        category: "spell",
      }),
    ).toEqual({
      field: "areaType",
      values: [
        { value: "burst", count: 1 },
        { value: "cone", count: 1 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "durationUnit",
        category: "spell",
      }),
    ).toEqual({
      field: "durationUnit",
      values: [
        { value: "minute", count: 2 },
        { value: "unlimited", count: 1 },
      ],
    });

    expect(
      service
        .listFilterValues({
          field: "weaponGroup",
          category: "equipment",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["bomb", "sword"]));

    expect(
      service
        .listFilterValues({
          field: "usage",
          category: "equipment",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["held-in-one-hand"]);

    expect(
      service
        .listFilterValues({
          field: "hands",
          category: "equipment",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["1"]);

    expect(
      service
        .listFilterValues({
          field: "actionCost",
          category: "spell",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["2"]);

    expect(
      service
        .listFilterValues({
          field: "rangeValue",
          category: "spell",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["30", "60"]);

    expect(
      service
        .listFilterValues({
          field: "areaValue",
          category: "spell",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["30", "5"]);

    expect(
      service
        .listFilterValues({
          field: "sustained",
          category: "spell",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["false", "true"]));

    expect(
      service
        .listFilterValues({
          field: "basicSave",
          category: "spell",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["false", "true"]));

    expect(
      service
        .listFilterValues({
          field: "damageTypes",
          category: "equipment",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["positive", "slashing"]));

    expect(
      service
        .listFilterValues({
          field: "itemCategory",
          category: "equipment",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["backpack", "consumable", "equipment", "weapon"]));

    expect(
      service
        .listFilterValues({
          field: "baseItem",
          category: "equipment",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["alchemical-bomb", "longsword"]));

    expect(
      service
        .listFilterValues({
          field: "rarity",
          category: "creature",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["common", "uncommon", "rare", "unique"]);

    expect(
      service
        .listFilterValues({
          field: "size",
          category: "creature",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["med", "lg", "sm", "huge", "tiny"]);

    expect(
      service
        .listFilterValues({
          field: "sources",
          category: "creature",
        })
        .values.map((entry) => entry.value),
    ).toEqual(["core", "rules", "adventure"]);

    expect(
      service.listFilterValues({
        field: "packs",
        category: "spell",
      }),
    ).toEqual({
      field: "packs",
      values: [
        { value: "Spells", count: 19 },
        { value: "Spells SRD", count: 9 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "categories",
        scopes: [
          { category: "hazards", subcategories: ["traps"] },
          { category: "rules", subcategories: ["conditions"] },
        ],
      }),
    ).toEqual({
      field: "categories",
      values: [
        { value: "hazard", count: 16 },
        { value: "rule", count: 6 },
      ],
    });

    expect(() =>
      service.listFilterValues({
        field: "traits",
        scopes: [{ category: "feat", subcategories: ["action"] }],
      }),
    ).toThrow(/does not belong to category "feat"/i);
  });

  it("discovers live actor and item metric keys and scalar values through filter-value listing", async () => {
    const fixture = await createHardFilterFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    expect(
      service
        .listFilterValues({
          field: "actorMetrics",
          category: "creature",
          metricPrefix: "ability",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["ability.cha.mod", "ability.dex.mod", "ability.int.mod", "ability.str.mod"]));

    expect(
      service
        .listFilterValues({
          field: "actorMetrics",
          category: "creature",
          metricPrefix: "speed",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["speed.fly.value", "speed.land.value"]));

    expect(
      service
        .listFilterValues({
          field: "actorMetrics",
          category: "creature",
          metricPrefix: "sense",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["sense.scent.range"]));

    expect(
      service.listFilterValues({
        field: "actorMetrics",
        category: "creature",
        metric: "save.best",
      }),
    ).toEqual({
      field: "actorMetrics",
      values: [
        { value: "fort", count: 1 },
        { value: "ref", count: 1 },
        { value: "will", count: 1 },
      ],
    });

    expect(
      service.listFilterValues({
        field: "actorMetrics",
        category: "creature",
        metric: "skill.arcana.proficient",
      }),
    ).toEqual({
      field: "actorMetrics",
      values: [
        { value: "false", count: 2 },
        { value: "true", count: 1 },
      ],
    });

    expect(
      service
        .listMetricCatalogKeys({
          field: "actorMetrics",
          category: "creature",
          metricPrefix: "save",
        })
        ?.values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["save.best", "save.fort.mod", "save.ref.mod", "save.will.mod", "save.worst"]));

    expect(
      service.listMetricCatalogValues({
        field: "actorMetrics",
        category: "creature",
        metric: "save.best",
      }),
    ).toEqual({
      field: "actorMetrics",
      values: [
        { value: "fort", count: 1 },
        { value: "ref", count: 1 },
        { value: "will", count: 1 },
      ],
    });

    expect(
      service.listMetricCatalogValues({
        field: "actorMetrics",
        category: "creature",
        metric: "ability.int.mod",
      }),
    ).toEqual({
      field: "actorMetrics",
      values: [],
    });

    expect(() =>
      service.listFilterValues({
        field: "actorMetrics",
        category: "creature",
        metric: "ability.int.mod",
      }),
    ).toThrow(/text and boolean metrics/i);

    expect(
      service
        .listFilterValues({
          field: "actorMetrics",
          category: "hazard",
          metricPrefix: "stealth",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["stealth.dc", "stealth.mod"]));

    expect(
      service
        .listFilterValues({
          field: "actorMetrics",
          category: "hazard",
          metricPrefix: "disable",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining([
        "disable.dc.max",
        "disable.dc.min",
        "disable.crafting.dc.min",
        "disable.crafting.rank.min",
        "disable.thievery.dc.min",
        "disable.thievery.rank.min",
        "disable.religion.dc.min",
        "disable.religion.rank.min",
      ]),
    );

    expect(
      service.listFilterValues({
        field: "actorMetrics",
        category: "hazard",
        metric: "save.best",
      }),
    ).toEqual({
      field: "actorMetrics",
      values: [
        { value: "fort", count: 1 },
        { value: "will", count: 1 },
      ],
    });

    expect(
      service
        .listFilterValues({
          field: "itemMetrics",
          category: "equipment",
          metricPrefix: "weapon",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining(["weapon.damage_dice", "weapon.damage_die_faces", "weapon.range_increment", "weapon.reload"]),
    );

    expect(
      service
        .listFilterValues({
          field: "itemMetrics",
          category: "equipment",
          metricPrefix: "shield",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["shield.ac_bonus", "shield.bt", "shield.hardness", "shield.hp"]));

    expect(
      service
        .listFilterValues({
          field: "itemMetrics",
          category: "equipment",
          metricPrefix: "armor",
        })
        .values.map((entry) => entry.value),
    ).toEqual(
      arrayContaining([
        "armor.ac_bonus",
        "armor.check_penalty",
        "armor.dex_cap",
        "armor.speed_penalty",
        "armor.strength",
      ]),
    );

    expect(
      service
        .listFilterValues({
          field: "senses",
          category: "creature",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["darkvision", "scent"]));

    expect(
      service
        .listFilterValues({
          field: "disableSkills",
          category: "hazard",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["crafting", "religion", "thievery"]));

    expect(
      service
        .listFilterValues({
          field: "isComplex",
          category: "hazard",
        })
        .values.map((entry) => entry.value),
    ).toEqual(arrayContaining(["false", "true"]));

    expect(() =>
      service.listFilterValues({
        field: "itemMetrics",
        category: "equipment",
        metric: "weapon.reload",
      }),
    ).toThrow(/text and boolean metrics/i);
  });

  it("collects rule question context without synthesis", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const result = service.collectRuleQuestionContext({
      rules: ["Refocus"],
      includeBacklinks: true,
      maxOutgoingPerPrimary: 5,
      maxBacklinksPerPrimary: 5,
    });

    expect(result.primary[0]?.match?.name).toBe("Refocus");
    expect(result.outgoing.records).toHaveLength(0);
    expect(result.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(result.edges).toHaveLength(2);
  });

  it("collects natural-language rule question context using canonical graph edges", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);

    const service = await loadTestService(fixture);

    const result = service.collectRuleQuestionContext({
      question: "How does Deep Focus interplay with Refocus?",
      includeBacklinks: true,
      maxOutgoingPerPrimary: 5,
      maxBacklinksPerPrimary: 5,
    });

    expect(result.primary.map((entry) => entry.match?.name)).toEqual(["Deep Focus", "Refocus"]);
    expect(result.outgoing.records.map((record) => record.name)).toEqual(["Refocus"]);
    expect(result.backlinks.records.map((record) => record.name)).toEqual(["Deep Focus", "Meditative Well"]);
    expect(result.edges).toHaveLength(3);
  });
});
