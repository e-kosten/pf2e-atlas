import { Alert, Tabs, Typography } from "antd";
import { useState } from "react";
import type { RecordDetailView, RecordSurfaceView } from "../../generated/atlas";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { useRecordDetail } from "../../shared/records/useRecordDetail";

const NIGHT_HAG_KEY = "pathfinder-bestiary:WQy7HBUcgDLsfVJd";
const GIANT_RAT_KEY = "pathfinder-monster-core:iIJPJcDT8wlJ8z5M";

type SampleKey = "concept" | "night-hag" | "giant-rat";

const labels: Record<SampleKey, string> = {
  concept: "Concept mock",
  "night-hag": "Night Hag",
  "giant-rat": "Giant Rat",
};

export function PresentationMocksView() {
  const [active, setActive] = useState<SampleKey>(initialSample);
  const nightHag = useRecordDetail(NIGHT_HAG_KEY);
  const giantRat = useRecordDetail(GIANT_RAT_KEY);
  const sample =
    active === "concept"
      ? {
          detail: { surface: conceptSurface } satisfies RecordDetailView,
          error: null,
          loading: false,
          stale: false,
        }
      : active === "night-hag"
        ? {
            detail: nightHag.data,
            error: nightHag.error,
            loading: nightHag.isLoading,
            stale: nightHag.isFetching && Boolean(nightHag.data),
          }
        : {
            detail: giantRat.data,
            error: giantRat.error,
            loading: giantRat.isLoading,
            stale: giantRat.isFetching && Boolean(giantRat.data),
          };

  return (
    <main className="surface-mocks surface-mocks--f1">
      <header className="surface-mocks__header">
        <div>
          <p className="eyebrow">F1 candidate direction</p>
          <h1>PF2e creature record</h1>
          <p>
            A curated record-detail hierarchy with description before mechanics,
            contextual activity content, grouped spells, and a distinct encounter
            reading order.
          </p>
        </div>
      </header>
      <Alert
        message={
          active === "concept"
            ? "Labeled concept mock — renderer-authentic, not a source record"
            : `Provenance-bound real record — ${labels[active]}`
        }
        showIcon
        type={active === "concept" ? "warning" : "success"}
      />
      <Tabs
        activeKey={active}
        items={(Object.keys(labels) as SampleKey[]).map((key) => ({
          key,
          label: labels[key],
        }))}
        onChange={(key) => {
          const next = key as SampleKey;
          setActive(next);
          const url = new URL(window.location.href);
          url.searchParams.set("sample", next);
          history.replaceState(null, "", url);
        }}
      />
      <section
        aria-label={`${labels[active]} creature record`}
        className="surface-mocks__record"
      >
        <RecordDetailPane
          detail={sample.detail}
          emptyMessage="This evidence record is unavailable."
          errors={[sample.error]}
          loading={sample.loading}
          loadingMessage={`Loading ${labels[active]}`}
          onReference={() => undefined}
          stale={sample.stale}
        />
      </section>
      <Typography.Paragraph className="surface-mocks__provenance" type="secondary">
        The concept is explicitly non-authentic. Night Hag and Giant Rat load unchanged
        from the signed Stage F record-detail payloads.
      </Typography.Paragraph>
    </main>
  );
}

function initialSample(): SampleKey {
  const requested = new URLSearchParams(window.location.search).get("sample");
  return requested === "night-hag" || requested === "giant-rat" ? requested : "concept";
}

const factProvenance = {
  owner: "canonical_creature" as const,
  field: "defenses" as const,
};

const occurrenceProvenance = {
  identity_stability: "stable_nested_source_id" as const,
};

const conceptSurface: RecordSurfaceView = {
  metadata: {
    record_key: "concept:f1-dream-coven-envoy",
    title: "Dream-Coven Envoy",
    kind: "creature",
    kind_label: "Creature",
    level: 9,
    rarity: "uncommon",
    traits: ["fiend", "hag", "humanoid", "unholy"],
    edition: {
      status: "legacy",
      counterparts: [
        {
          role: "remastered_counterpart",
          record_key: "concept:f1-remastered-envoy",
          title: "Dream-Coven Emissary",
        },
      ],
    },
    source: {
      publication_title: "F1 concept evidence",
      pack_label: "Concept mock",
      document_type: "Actor",
      record_type: "npc",
    },
  },
  profile: "record_detail",
  presentation: {
    presentation_type: "creature",
    body: {
      teaser: "A dream-stalking emissary who turns bargains into occult leverage.",
      size: { value: "medium", provenance: factProvenance },
      adjustment: { value: "elite", provenance: factProvenance },
      initiative: { statistic: "perception", provenance: factProvenance },
      vitals: { hit_points: 170, provenance: factProvenance },
      defenses: {
        armor_class: 28,
        armor_class_details: "+1 against dream effects",
        hardness: 8,
        shield: {
          armor_class_bonus: 2,
          broken_threshold: 10,
          hardness: 5,
          maximum_hit_points: 20,
        },
        immunities: [{ component_id: "sleep", authored_order: 0, kind: "sleep" }],
        weaknesses: [
          {
            component_id: "cold-iron",
            authored_order: 0,
            kind: "cold iron",
            amount: 10,
          },
        ],
        provenance: factProvenance,
      },
      saves: {
        fortitude: {
          component_id: "fortitude",
          modifier: 19,
          details: "+1 against curses",
        },
        reflex: { component_id: "reflex", modifier: 17 },
        will: { component_id: "will", modifier: 20 },
        provenance: factProvenance,
      },
      awareness: {
        perception: 18,
        senses: [
          {
            component_id: "darkvision",
            authored_order: 0,
            kind: "darkvision",
          },
          {
            component_id: "dream-scent",
            authored_order: 1,
            kind: "dream scent",
            acuity: "imprecise",
            range_feet: 60,
          },
        ],
        languages: ["Aklo", "Common", "Infernal"],
        provenance: { ...factProvenance, field: "perception" },
      },
      abilities: {
        strength: 4,
        dexterity: 3,
        constitution: 4,
        intelligence: 5,
        wisdom: 4,
        charisma: 5,
        provenance: { ...factProvenance, field: "legacy_abilities" },
      },
      skills: [
        {
          component_id: "deception",
          authored_order: 0,
          kind: "deception",
          label: "Deception",
          modifier: 20,
        },
        {
          component_id: "occultism",
          authored_order: 1,
          kind: "occultism",
          label: "Occultism",
          modifier: 22,
          variants: [
            {
              component_id: "dreams",
              authored_order: 0,
              modifier: 24,
              label: "Dreams",
              predicates: [{ predicate_type: "term", term: "dreams" }],
            },
          ],
          source_entries: [
            {
              authored_order: 0,
              authored_key: "occultism",
              modifier: { state: "value", value: 22 },
            },
          ],
        },
      ],
      movement: [
        {
          component_id: "land",
          authored_order: 0,
          mode: "land",
          label: "Speed",
          speed_feet: 25,
        },
        {
          component_id: "fly",
          authored_order: 1,
          mode: "fly",
          speed_feet: 40,
          details: "Only while ethereal",
        },
      ],
      activities: [
        {
          occurrence_id: "claw",
          authored_order: 0,
          provenance: occurrenceProvenance,
          activity_type: "strike",
          label: "Claw",
          traits: ["agile", "magical"],
          action_cost: { cost_type: "actions", count: 1 },
          rolls: [{ roll_id: "attack", label: "Attack", kind: "attack", modifier: 21 }],
          damage: [
            {
              damage_id: "claw-damage",
              formula: "2d8+10",
              damage_type: "slashing",
            },
          ],
          attack_effects: ["Grab"],
          category: "offensive",
        },
        {
          occurrence_id: "dream-bargain",
          authored_order: 1,
          provenance: occurrenceProvenance,
          activity_type: "action",
          label: "Dream Bargain",
          traits: ["curse", "occult"],
          action_cost: { cost_type: "actions", count: 2 },
          frequency: { maximum: 1, period: "PT1M", display: "1 per minute" },
          requirements: "The envoy can see the target.",
          cost: "One dream token",
          uses: { maximum: 3 },
          self_effect: { label: "Effect", value: "Dream veil" },
          category: "offensive",
          content: [
            {
              content_key: "concept:dream-bargain",
              role: "embedded_capability",
              authored_order: 0,
              label: "Dream Bargain",
              blocks: [
                {
                  block_type: "paragraph",
                  spans: [
                    {
                      span_type: "text",
                      text: "The envoy offers a perilous bargain to one creature within 30 feet.",
                    },
                  ],
                },
                {
                  block_type: "paragraph",
                  spans: [
                    {
                      span_type: "strong",
                      spans: [{ span_type: "text", text: "Saving Throw" }],
                    },
                    { span_type: "text", text: " " },
                    {
                      span_type: "check",
                      display: "Will DC 28",
                      statistic: "will",
                      difficulty_class: 28,
                    },
                  ],
                },
              ],
              content_hash: "concept-activity-not-source-authentic",
              visibility: "public",
              provenance: {
                source_record_key: "concept:f1-dream-coven-envoy",
                relative_source_path: "F1 concept mock",
                field_family: "concept.activity",
              },
            },
          ],
        },
      ],
      spellcasting: [
        {
          occurrence_id: "innate-occult",
          authored_order: 0,
          provenance: occurrenceProvenance,
          label: "Occult Innate Spells",
          tradition: "occult",
          preparation: "innate",
          difficulty_class: 28,
          attack_modifier: 20,
          slots: [
            { rank: 5, maximum: 2 },
            { rank: 4, maximum: 3 },
          ],
          spells: [
            {
              occurrence_id: "dream-message",
              authored_order: 0,
              provenance: occurrenceProvenance,
              label: "Dream Message",
              rank: 5,
              target_record_key: "spells:dream-message",
              context: {
                group: "innate",
                slot: "5",
                uses: { maximum: 1 },
                contextual_label: "Dream message",
              },
              content: [
                {
                  content_key: "concept:dream-message",
                  role: "embedded_capability",
                  authored_order: 0,
                  label: "Dream Message",
                  blocks: [
                    {
                      block_type: "paragraph",
                      spans: [
                        {
                          span_type: "text",
                          text: "The envoy sends a message through the target's dreams.",
                        },
                      ],
                    },
                  ],
                  content_hash: "concept-spell-not-source-authentic",
                  visibility: "public",
                  provenance: {
                    source_record_key: "concept:f1-dream-coven-envoy",
                    relative_source_path: "F1 concept mock",
                    field_family: "concept.spell",
                  },
                },
              ],
            },
            {
              occurrence_id: "nightmare",
              authored_order: 1,
              provenance: occurrenceProvenance,
              label: "Nightmare",
              rank: 4,
            },
          ],
        },
      ],
      standalone_spells: [
        {
          occurrence_id: "control-weather",
          authored_order: 0,
          provenance: occurrenceProvenance,
          label: "Control Weather",
          target_record_key: "spells:control-weather",
          rank: 8,
          content: [
            {
              content_key: "concept:control-weather",
              role: "embedded_capability",
              authored_order: 0,
              label: "Control Weather",
              blocks: [
                {
                  block_type: "paragraph",
                  spans: [
                    {
                      span_type: "text",
                      text: "The envoy reshapes the weather around its dream coven.",
                    },
                  ],
                },
              ],
              content_hash: "concept-standalone-spell-not-source-authentic",
              visibility: "public",
              provenance: {
                source_record_key: "concept:f1-dream-coven-envoy",
                relative_source_path: "F1 concept mock",
                field_family: "concept.standalone-spell",
              },
            },
          ],
        },
      ],
      resources: [
        {
          component_id: "dream-token",
          authored_order: 0,
          kind: "uses",
          label: "Dream tokens",
          maximum: 3,
        },
      ],
      rituals: { difficulty_class: 31, provenance: factProvenance },
      equipment: [
        {
          occurrence_id: "heartstone",
          authored_order: 0,
          provenance: occurrenceProvenance,
          label: "Heartstone",
          traits: ["magical"],
          level: 9,
          usage: "held",
          quantity: 1,
          uses: { maximum: 1 },
        },
      ],
      lore: [
        {
          occurrence_id: "dream-lore",
          authored_order: 0,
          provenance: occurrenceProvenance,
          label: "Dream Lore",
          modifier: 20,
        },
      ],
      content: [
        {
          content_key: "concept:description",
          role: "primary_description",
          authored_order: 0,
          label: "Description",
          blocks: [
            {
              block_type: "paragraph",
              spans: [
                {
                  span_type: "text",
                  text: "A dream-stalking emissary who bargains before battle and turns promises into occult leverage.",
                },
              ],
            },
            { block_type: "divider" },
            {
              block_type: "paragraph",
              spans: [
                {
                  span_type: "text",
                  text: "Its shifting silhouette is a visual concept only; every mechanic below exercises the accepted generated DTO.",
                },
              ],
            },
          ],
          content_hash: "concept-description-not-source-authentic",
          visibility: "public",
          provenance: {
            source_record_key: "concept:f1-dream-coven-envoy",
            relative_source_path: "F1 concept mock",
            field_family: "concept.description",
          },
        },
        {
          content_key: "concept:heartstone",
          role: "embedded_capability",
          authored_order: 1,
          label: "Heartstone",
          blocks: [
            {
              block_type: "paragraph",
              spans: [
                {
                  span_type: "text",
                  text: "The heartstone lets the envoy use ethereal jaunt.",
                },
              ],
            },
          ],
          content_hash: "concept-heartstone-not-source-authentic",
          visibility: "public",
          provenance: {
            source_record_key: "concept:f1-dream-coven-envoy",
            relative_source_path: "F1 concept mock",
            field_family: "concept.heartstone",
          },
        },
      ],
      provenance: {
        source_path: "F1 concept mock",
        source_contract_version: "concept",
        source_system_version: "concept",
        source_upstream_commit: "concept",
      },
    },
  },
};
