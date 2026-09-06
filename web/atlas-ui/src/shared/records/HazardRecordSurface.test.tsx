import { fireEvent, render, screen } from "@testing-library/react";
import type {
  RecordSurfaceView,
  RuntimeFactProvenanceView,
} from "../../generated/atlas";
import { RecordSurface } from "./RecordSurface";

const onReference = vi.fn();
const runtimeProvenance: RuntimeFactProvenanceView = {
  source: { source_type: "canonical_record" },
};

describe("HazardRecordSurface", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders canonical hazard detail with zero saves, occurrence identity, and rich references", () => {
    render(
      <RecordSurface
        onReference={onReference}
        surface={hazardSurface("record_detail")}
      />,
    );

    expect(screen.getByRole("heading", { name: "Hidden Pit" })).toBeInTheDocument();
    expect(screen.getByText("Complex")).toBeInTheDocument();
    expect(screen.getByText("+0")).toBeInTheDocument();
    expect(screen.getAllByText("Routine").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("Occurrence occurrence-action")).toBeInTheDocument();
    expect(screen.getByText("Occurrence occurrence-strike")).toBeInTheDocument();
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" &&
          element.textContent === "Flat modifier: +0 (damage)",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" &&
          element.textContent === "Aura: 5 feet (fixture-aura)",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(/^Slug:/)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Alarm" }));
    expect(onReference).toHaveBeenCalledWith("spells:alarm");
    fireEvent.click(screen.getByRole("button", { name: /Data availability/ }));
    expect(
      screen.getByText("This canonical hazard field was explicitly null."),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /References & Source/ }));
    expect(screen.getByText("ORC")).toBeInTheDocument();
    expect(document.querySelector("img[src]")).toBeNull();
    expect(
      screen.queryByText("systems/pf2e/icons/hidden-pit.webp"),
    ).not.toBeInTheDocument();
  });

  it("keeps disabled hazard information and independently available activities visible", () => {
    const surface = hazardSurface("encounter_participant");
    surface.encounter = {
      hazard: {
        state: "disabled",
        detection_dc: numberView("Detection DC", 22),
        broken_threshold: numberView("Broken Threshold", 15),
        initiative_suggestion: {
          statistic: "stealth",
          modifier: numberView("Stealth", 12),
        },
        convenience_rule_id: "pf2e-hazard-conveniences",
        convenience_rule_version: 1,
      },
      vitals: {
        maximum_hp: numberView("Maximum HP", 30),
        current_hp: 30,
        temporary_hp: 0,
      },
      defenses: { armor_class: numberView("Armor Class", 22) },
      saves: { fortitude: numberView("Fortitude", 0) },
      activities: [
        {
          activity_id: "occurrence-action",
          label: "Routine",
          kind: "other",
          usage: "unlimited",
          availability: { available: true, provenance: runtimeProvenance },
          action_cost: {
            value: { kind: "actions", count: 1 },
            provenance: runtimeProvenance,
          },
          content: [activityContent()],
          provenance: runtimeProvenance,
        },
      ],
    };

    render(<RecordSurface onReference={onReference} surface={surface} />);

    expect(screen.getByText("Disabled")).toBeInTheDocument();
    expect(screen.getByText("Detection DC")).toBeInTheDocument();
    expect(screen.getByText("Initiative suggestion")).toBeInTheDocument();
    expect(
      screen.getByText("Hazard action content remains available."),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Lifecycle" })).toBeInTheDocument();
    expect(
      screen.queryByText(/record family is not available/i),
    ).not.toBeInTheDocument();
  });
});

function hazardSurface(profile: RecordSurfaceView["profile"]): RecordSurfaceView {
  return {
    metadata: {
      record_key: "hazards:hidden-pit",
      title: "Hidden Pit",
      kind: "hazard",
      kind_label: "Hazard",
      level: 1,
      rarity: "common",
      traits: ["mechanical", "trap"],
      source: {
        publication_title: "Pathfinder Core",
        pack_label: "Hazards",
        document_type: "Actor",
        record_type: "hazard",
      },
    },
    profile,
    presentation: {
      presentation_type: "hazard",
      body: {
        complexity: "complex",
        size: "large",
        detection: {
          stealth_modifier: 12,
          difficulty_class: 22,
          details: [
            {
              block_type: "paragraph",
              spans: [
                { span_type: "text", text: "A concealed opening guarded by " },
                {
                  span_type: "reference",
                  label: "Alarm",
                  record_key: "spells:alarm",
                  embedded: false,
                },
                { span_type: "text", text: "." },
              ],
            },
          ],
        },
        defenses: {
          armor_class: 22,
          hardness: 10,
          hit_points: { current: 30, maximum: 30, broken_threshold: 15 },
          saves: { fortitude: 0, reflex: 8 },
        },
        lifecycle: {
          description: paragraph("The stones conceal a deep pit."),
          disable: paragraph("Thievery DC 12 opens the latch."),
          routine: paragraph("The trap resets without inferred automation."),
        },
        activities: [
          {
            occurrence_id: "occurrence-action",
            entity_id: "entity-action",
            authored_order: 0,
            source_ordinal: 0,
            identity_stability: "stable_source_identity",
            label: "Routine",
            activity_type: "action",
            action_cost: { cost_type: "actions", count: 1 },
            rules: [
              {
                rule_type: "flat_modifier",
                authored_order: 0,
                value: 0,
                selector: "damage",
              },
              {
                rule_type: "aura",
                authored_order: 1,
                radius: 5,
                slug: "fixture-aura",
              },
            ],
            content: [activityContent()],
          },
          {
            occurrence_id: "occurrence-strike",
            entity_id: "entity-strike",
            authored_order: 1,
            source_ordinal: 1,
            identity_stability: "stable_source_identity",
            label: "Routine",
            activity_type: "strike",
            attack_bonus: 11,
            damage: [{ damage_id: "main", formula: "1d8", damage_type: "piercing" }],
          },
        ],
        unavailable_fields: [
          {
            state: "null",
            field: "defenses.hit_points.temporary",
            message: "This canonical hazard field was explicitly null.",
          },
        ],
        provenance: {
          source_path: "packs/hazards/hidden-pit.json",
          source_contract_version: "v1",
          source_system_version: "7",
          source_upstream_commit: "fixture",
          convenience_rule_id: "pf2e-hazard-conveniences",
          convenience_rule_version: 1,
          image: { state: "value", value: "systems/pf2e/icons/hidden-pit.webp" },
          publication_license: { state: "value", value: "ORC" },
        },
      },
    },
  };
}

function paragraph(text: string) {
  return [
    { block_type: "paragraph" as const, spans: [{ span_type: "text" as const, text }] },
  ];
}

function activityContent() {
  return {
    content_key: "activity-routine",
    role: "embedded_capability" as const,
    authored_order: 0,
    blocks: paragraph("Hazard action content remains available."),
    content_hash: "fixture-activity",
    visibility: "gm",
    provenance: {
      source_record_key: "hazards:hidden-pit",
      relative_source_path: "packs/hazards/hidden-pit.json",
      field_family: "embedded.description",
      nested_source_id: "entity-action",
    },
  };
}

function numberView(label: string, value: number) {
  return {
    label,
    base_value: value,
    adjusted_value: value,
    provenance: runtimeProvenance,
  };
}
