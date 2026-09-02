import type {
  EncounterParticipantView,
  EncounterRuntimeView,
  RecordDetailView,
  RecordSummaryView,
  RecordSurfaceProfileView,
  RecordSurfaceView,
} from "../generated/atlas";

const canonicalProvenance = {
  owner: "canonical_creature" as const,
  field: "defenses" as const,
};

export const occurrenceProvenance = {
  identity_stability: "stable_nested_source_id" as const,
};

const runtimeProvenance = {
  source: { source_type: "canonical_record" as const },
};

export function creatureSurfaceFixture({
  encounter,
  level = 1,
  profile = "record_detail",
  recordKey = "actors:goblin",
  referenceLabel,
  referenceRecordKey,
  title = "Goblin Warrior",
  traits = ["goblin", "humanoid"],
}: {
  encounter?: EncounterRuntimeView;
  level?: number;
  profile?: RecordSurfaceProfileView;
  recordKey?: string;
  referenceLabel?: string;
  referenceRecordKey?: string;
  title?: string;
  traits?: string[];
} = {}): RecordSurfaceView {
  return {
    metadata: {
      record_key: recordKey,
      title,
      kind: "creature",
      kind_label: "Creature",
      level,
      traits,
      source: {
        publication_title: "Pathfinder Bestiary",
        pack_label: "Bestiary",
        document_type: "Actor",
        record_type: "npc",
      },
    },
    profile,
    presentation: {
      presentation_type: "creature",
      body: {
        vitals: { hit_points: 30, provenance: canonicalProvenance },
        defenses: { armor_class: 18, provenance: canonicalProvenance },
        saves: {
          fortitude: { component_id: "fortitude", modifier: 8 },
          reflex: { component_id: "reflex", modifier: 10 },
          will: { component_id: "will", modifier: 6 },
          provenance: canonicalProvenance,
        },
        awareness: {
          perception: 8,
          languages: ["Common"],
          provenance: { ...canonicalProvenance, field: "perception" },
        },
        movement: [
          {
            component_id: "land",
            authored_order: 0,
            mode: "land",
            label: "Speed",
            speed_feet: 25,
          },
        ],
        activities: [
          {
            occurrence_id: "claw",
            authored_order: 0,
            provenance: occurrenceProvenance,
            activity_type: "strike",
            label: "Claw",
            action_cost: { cost_type: "actions", count: 1 },
            rolls: [
              { roll_id: "attack", label: "Attack", kind: "attack", modifier: 12 },
            ],
            damage: [
              {
                damage_id: "main",
                formula: "1d6+2",
                damage_type: "slashing",
              },
            ],
          },
        ],
        content: [
          {
            content_key: "description",
            role: "primary_description",
            authored_order: 0,
            label: "Description",
            blocks: [
              {
                block_type: "paragraph",
                spans: [
                  {
                    span_type: "text",
                    text: "A quick goblin scout with a cruel claw.",
                  },
                ],
              },
            ],
            content_hash: "fixture-description",
            visibility: "public",
            provenance: {
              source_record_key: recordKey,
              relative_source_path: "fixture.json",
              field_family: "fixture.description",
            },
          },
        ],
        ...(referenceLabel && referenceRecordKey
          ? {
              spellcasting: [
                {
                  occurrence_id: "innate",
                  authored_order: 0,
                  provenance: occurrenceProvenance,
                  label: "Innate Spells",
                  spells: [
                    {
                      occurrence_id: "linked-spell",
                      authored_order: 0,
                      provenance: occurrenceProvenance,
                      label: referenceLabel,
                      target_record_key: referenceRecordKey,
                    },
                  ],
                },
              ],
            }
          : {}),
      },
    },
    ...(encounter ? { encounter } : {}),
  };
}

export function encounterRuntimeFixture({
  actions = 3,
  currentHp = 30,
  maximumHp = 40,
  reactions = 1,
  temporaryHp = 0,
}: {
  actions?: number;
  currentHp?: number;
  maximumHp?: number;
  reactions?: number;
  temporaryHp?: number;
} = {}): EncounterRuntimeView {
  const numberFact = (label: string, value: number) => ({
    label,
    base_value: value,
    adjusted_value: value,
    provenance: runtimeProvenance,
  });
  return {
    vitals: {
      maximum_hp: numberFact("Maximum HP", maximumHp),
      current_hp: currentHp,
      temporary_hp: temporaryHp,
    },
    defenses: { armor_class: numberFact("Armor Class", 18) },
    saves: {
      fortitude: numberFact("Fortitude", 8),
      reflex: numberFact("Reflex", 10),
      will: numberFact("Will", 6),
    },
    awareness: { perception: numberFact("Perception", 8) },
    action_budget: {
      actions: numberFact("Actions", actions),
      reactions: numberFact("Reactions", reactions),
      can_act: { available: true },
      can_react: { available: true },
    },
    conditions: [],
  };
}

export function recordDetailFixture(
  overrides: Parameters<typeof creatureSurfaceFixture>[0] = {},
): RecordDetailView {
  return { surface: creatureSurfaceFixture(overrides) };
}

export function recordSummaryFixture(
  recordKey = "actors:goblin",
  title = "Goblin Warrior",
): RecordSummaryView {
  return {
    surface: creatureSurfaceFixture({
      profile: "search_compact",
      recordKey,
      title,
    }),
  };
}

export function encounterParticipantFixture(
  overrides: Partial<EncounterParticipantView> = {},
): EncounterParticipantView {
  const encounter = encounterRuntimeFixture();
  return {
    participant_key: "participant-1",
    record_key: "actors:goblin",
    participant_kind: "creature",
    participant_variant: "normal",
    status: "active",
    position: 0,
    display_name: "Goblin Warrior",
    side: "enemy",
    initiative: 12,
    initiative_order: 0,
    defeated: false,
    hidden: false,
    note_hint: null,
    record_view: creatureSurfaceFixture({
      encounter,
      profile: "encounter_participant",
    }),
    ...overrides,
  };
}
