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
  contentReferenceChildLocator,
  contentReferenceLabel,
  contentReferenceRecordKey,
  encounter,
  level = 1,
  profile = "record_detail",
  recordKey = "actors:goblin",
  referenceLabel,
  referenceRecordKey,
  title = "Goblin Warrior",
  traits = ["goblin", "humanoid"],
}: {
  contentReferenceChildLocator?: string;
  contentReferenceLabel?: string;
  contentReferenceRecordKey?: string;
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
          ...(contentReferenceLabel && contentReferenceRecordKey
            ? [
                {
                  content_key: "fixture-child-reference",
                  role: "supplemental_rules" as const,
                  authored_order: 1,
                  label: "Related content",
                  blocks: [
                    {
                      block_type: "paragraph" as const,
                      spans: [
                        {
                          span_type: "reference" as const,
                          label: contentReferenceLabel,
                          record_key: contentReferenceRecordKey,
                          ...(contentReferenceChildLocator
                            ? { child_locator: contentReferenceChildLocator }
                            : {}),
                          embedded: false,
                        },
                      ],
                    },
                  ],
                  content_hash: "fixture-child-reference-hash",
                  visibility: "public" as const,
                  provenance: {
                    source_record_key: recordKey,
                    relative_source_path: "fixture.json",
                    field_family: "fixture.child-reference",
                  },
                },
              ]
            : []),
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

export const heroPointDeckRecordKey = "rollable-tables:zgZoI7h0XjjJrrNK";
export const rollTableResultOneLocator = "v1~t~s~4531636a674171465a497a436a447555";
export const rollTableResultTwoLocator = "v1~t~s~726573756c742d74776f";

export function rollTableRecordDetailFixture(): RecordDetailView {
  const missing = { state: "missing" } as const;
  const known = <T>(value: T) => ({ state: "known", value }) as const;
  const resultOf = (
    locator: string,
    sourceId: string,
    sourceOrdinal: number,
    text: string,
  ) => ({
    entry_type: "result" as const,
    result: {
      locator,
      identity_stability: "stable_source_id" as const,
      source_id: known(sourceId),
      source_ordinal: sourceOrdinal,
      result_kind: known("text"),
      text: known([
        {
          block_type: "paragraph" as const,
          spans: [{ span_type: "text" as const, text }],
        },
      ]),
      collection: missing,
      document_id: missing,
      weight: known("1"),
      range: known({ first: sourceOrdinal + 1, last: sourceOrdinal + 1 }),
      drawn: known(false),
      image: missing,
      source_metadata: { flags: missing },
    },
  });

  return {
    surface: {
      metadata: {
        record_key: heroPointDeckRecordKey,
        title: "Hero Point Deck",
        kind: "roll_table",
        kind_label: "Roll Table",
      },
      profile: "record_detail",
      presentation: {
        presentation_type: "roll_table",
        body: {
          source_id: "table-id",
          description: missing,
          results: known([
            resultOf(
              rollTableResultOneLocator,
              "E1cjgAqFZIzCjDuU",
              0,
              "First result content.",
            ),
            resultOf(
              rollTableResultTwoLocator,
              "result-two",
              1,
              "Second result content.",
            ),
          ]),
          formula: known("1d2"),
          replacement: known(false),
          display_roll: known(true),
          image: missing,
          source_metadata: {
            folder: missing,
            sort: known(0),
            ownership: missing,
            flags: missing,
            stats: missing,
          },
          provenance: {
            source_path: "packs/rollable-tables/hero-point-deck.json",
            source_contract_version: "pf2e-serialized-source/v1",
            source_system_version: "6.12.4",
            source_upstream_commit: "4cbdaa37",
          },
        },
      },
    },
  };
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
    reset: { available: true },
    record_view: creatureSurfaceFixture({
      encounter,
      profile: "encounter_participant",
    }),
    ...overrides,
  };
}
