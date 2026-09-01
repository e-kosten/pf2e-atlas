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

const conceptSurface: RecordSurfaceView = {
  metadata: {
    record_key: "concept:f1-dream-coven-envoy",
    title: "Dream-Coven Envoy",
    kind: "creature",
    kind_label: "Creature",
    level: 9,
    rarity: "uncommon",
    traits: ["fiend", "hag", "humanoid", "unholy"],
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
      vitals: { hit_points: 170, provenance: factProvenance },
      defenses: {
        armor_class: 28,
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
        fortitude: { component_id: "fortitude", modifier: 19 },
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
        },
        {
          occurrence_id: "dream-bargain",
          authored_order: 1,
          activity_type: "action",
          label: "Dream Bargain",
          traits: ["curse", "occult"],
          action_cost: { cost_type: "actions", count: 2 },
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
          label: "Occult Innate Spells",
          tradition: "occult",
          preparation: "innate",
          difficulty_class: 28,
          attack_modifier: 20,
          spells: [
            {
              occurrence_id: "dream-message",
              authored_order: 0,
              label: "Dream Message",
              rank: 5,
              target_record_key: "spells:dream-message",
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
