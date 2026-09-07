import { fireEvent, render, screen, within } from "@testing-library/react";
import type {
  HazardSurfaceActivityView,
  HazardSurfaceSourceMetadataFactView,
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

  it("presents Hidden Pit defenses, graph references, typed issues, and secondary provenance without internal identity", () => {
    const surface = hazardSurface("record_detail");
    const body = hazardBody(surface);
    body.relationships = [
      {
        relationship_id: "contains-internal-relationship",
        authored_order: 0,
        source_occurrence_id: "occurrence-action",
        target: { target_type: "entity", entity_id: "entity-action" },
      },
    ];
    const routine = body.activities?.[0];
    if (routine) {
      routine.self_effect = {
        label: "Reset mechanism",
        target_uuid: "Compendium.pf2e.effects.Item.internal-effect-id",
      };
      routine.rules = [
        {
          rule_type: "active_effect_like",
          authored_order: 0,
          mode: "override",
          path: "system.attributes.hp.value",
          value: false,
        },
        {
          rule_type: "aura",
          authored_order: 1,
          radius: 5,
          slug: "internal-aura-slug",
          traits: ["magical"],
        },
        {
          rule_type: "flat_modifier",
          authored_order: 2,
          selector: "internal-selector",
          value: 0,
        },
        { rule_type: "unsupported", authored_order: 3 },
      ];
    }
    body.unavailable_fields = [
      {
        state: "null",
        field: "defenses.hit_points.temporary",
        message: "Optional temporary HP was explicitly null.",
      },
    ];
    body.provenance.source_metadata = [
      typedSourceFact("token_name", "Hidden Pit", "/prototypeToken/name"),
      typedSourceFact("has_health", true, "/system/attributes/hasHealth"),
      typedSourceFact("temporary_maximum", 0, "/system/attributes/hp/tempmax"),
      {
        field: "save_detail",
        save: "fortitude",
        value: {
          state: "typed",
          source_path: "/system/saves/fortitude/saveDetail",
          value: "",
        },
      },
      {
        field: "item_rarity",
        entity_id: "entity-action",
        value: {
          state: "typed",
          source_path: "/items/0/system/traits/rarity",
          value: "common",
        },
      },
      {
        field: "item_lineage",
        entity_id: "entity-action",
        value: {
          state: "typed",
          source_path: "/items/0/_stats",
          value: {
            compendium_source: {
              state: "typed",
              source_path: "/items/0/_stats/compendiumSource",
              value: "Compendium.pf2e.hazards.Item.internal-source-id",
            },
          },
        },
      },
    ];
    surface.issues = [
      {
        code: "unmodeled",
        placement: "activity",
        message: "A populated hazard activity detail is not yet modeled.",
      },
      {
        code: "unsupported",
        placement: "rules",
        message: "An authored hazard rule is not available.",
      },
    ];
    surface.references = {
      outgoing: {
        state: "available",
        requested_limit: 8,
        records: [{ record_key: "spells:alarm", title: "Alarm", kind: "spell" }],
        edges: [
          {
            from_record_key: "hazards:hidden-pit",
            to_record_key: "spells:alarm",
            display_text: "Alarm",
            reference_text: "Alarm",
            source: {
              kind: "rich_content",
              visibility: "public",
              relation_kind: "references",
            },
          },
        ],
        total_records: 3,
        total_edges: 4,
        truncated: true,
      },
      backlinks: {
        state: "unavailable",
        requested_limit: 8,
        code: "internal_error",
        message: "Backlink lookup is temporarily unavailable.",
      },
    };
    const onReferencesOpen = vi.fn();

    const { container } = render(
      <RecordSurface
        onReference={onReference}
        onReferencesOpen={onReferencesOpen}
        surface={surface}
      />,
    );

    expect(screen.getByRole("heading", { name: "Hidden Pit" })).toBeInTheDocument();
    const defenses = screen.getByLabelText("Hazard defense statistics");
    expect(within(defenses).getByText("HP")).toBeInTheDocument();
    expect(within(defenses).getByText("30")).toBeInTheDocument();
    expect(within(defenses).getByText("BT")).toBeInTheDocument();
    expect(within(defenses).getByText("+0")).toBeInTheDocument();
    expect(within(defenses).queryByText(/temporary/i)).not.toBeInTheDocument();
    expect(screen.queryByText("Maximum HP")).not.toBeInTheDocument();
    expect(screen.queryByText("Shield")).not.toBeInTheDocument();
    expect(
      screen.getAllByText("A populated hazard activity detail is not yet modeled."),
    ).toHaveLength(1);
    expect(
      screen.getAllByText("An authored hazard rule is not available."),
    ).toHaveLength(1);
    expect(screen.queryByText("Unsupported authored rule")).toBeNull();
    expect(screen.queryByText("Optional temporary HP was explicitly null.")).toBeNull();

    const headings = [
      "Overview",
      "Detection & disable",
      "Defenses & Structure",
      "Activities",
      "Operation",
    ].map((name) => screen.getByRole("heading", { name }));
    for (const [current, next] of headings
      .slice(0, -1)
      .map((current, index) => [current, headings[index + 1]])) {
      expect(
        current.compareDocumentPosition(next) & Node.DOCUMENT_POSITION_FOLLOWING,
      ).toBeTruthy();
    }
    expect(screen.getByText("Thievery DC 12 opens the latch.")).toBeInTheDocument();
    expect(
      screen.getByText("The trap resets without inferred automation."),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "References" }));
    expect(onReferencesOpen).toHaveBeenCalledTimes(1);
    expect(screen.getByText("3 records · 4 references")).toBeInTheDocument();
    expect(screen.getAllByText("Limit 8")).toHaveLength(1);
    expect(screen.getByText("More available")).toBeInTheDocument();
    expect(
      screen.getByText("Backlink lookup is temporarily unavailable."),
    ).toBeInTheDocument();
    fireEvent.click(screen.getAllByRole("button", { name: "Alarm" })[0]);
    expect(onReference).toHaveBeenLastCalledWith("spells:alarm");

    openDisclosure("Source & provenance");
    expect(screen.getByText("Temporary maximum")).toBeInTheDocument();
    expect(screen.getByText("Component rarity")).toBeInTheDocument();
    expect(screen.getByText("Component lineage")).toBeInTheDocument();
    expect(screen.queryByText("Fortitude source note")).not.toBeInTheDocument();
    expect(screen.queryByText("internal-source-id", { exact: false })).toBeNull();
    expect(screen.queryByText("packs/hazards/hidden-pit.json")).toBeNull();
    expect(screen.queryByText(/Occurrence /)).toBeNull();
    expect(screen.queryByText(/entity-action|occurrence-action/)).toBeNull();
    expect(screen.queryByText(/Contains/i)).toBeNull();
    expect(screen.getByText("Active effect (Override): disabled")).toBeInTheDocument();
    expect(screen.getByText("Aura: 5 feet; magical")).toBeInTheDocument();
    expect(screen.getByText("Flat modifier: +0")).toBeInTheDocument();
    expect(
      screen.queryByText(
        /internal-effect-id|system\.attributes|internal-aura|internal-selector/,
      ),
    ).toBeNull();
    expect(container.querySelector("img[src]")).toBeNull();
  });

  it("keeps Dragon Pillar current/max HP and nonzero temporary HP in one hazard-owned placement", () => {
    const surface = hazardSurface("record_detail", {
      level: 6,
      title: "Dragon Pillar",
      traits: ["magical", "trap"],
    });
    const body = hazardBody(surface);
    body.defenses = {
      armor_class: 24,
      hardness: 14,
      hit_points: {
        current: 40,
        maximum: 56,
        temporary: 7,
        broken_threshold: 28,
      },
      saves: { fortitude: 17, reflex: 8, will: 0 },
    };
    const eyeBeam = strikeActivity("Eye Beam", "dragon-pillar-eye-beam", 0, 20);
    eyeBeam.attack_mode = "ranged";
    eyeBeam.action_cost = { cost_type: "actions", count: 1 };
    eyeBeam.traits = ["divine", "range-120"];
    eyeBeam.damage = [];
    body.activities = [eyeBeam];

    render(<RecordSurface onReference={onReference} surface={surface} />);

    const defenses = screen.getByLabelText("Hazard defense statistics");
    expect(within(defenses).getByText("40/56")).toBeInTheDocument();
    expect(within(defenses).getByText("+7 temporary")).toBeInTheDocument();
    expect(
      [...defenses.querySelectorAll("dt")].map((label) => label.textContent),
    ).toEqual(["AC", "HP", "Hardness", "BT", "Fort", "Ref", "Will"]);
    expect(within(defenses).getByText("+17")).toBeInTheDocument();
    expect(within(defenses).getByText("+8")).toBeInTheDocument();
    expect(within(defenses).getByText("+0")).toBeInTheDocument();
    expect(screen.queryByText("Maximum HP")).toBeNull();
    expect(screen.queryByText("Temporary HP")).toBeNull();
    expect(screen.getByText("Ranged Strike")).toBeInTheDocument();
    expect(screen.getByText("One action")).toHaveClass("sr-only");
    expect(screen.getByText("Attack +20")).toBeInTheDocument();
  });

  it("shows Acid Spray's typed melee mode and derived one-action cost without trait inference", () => {
    const surface = hazardSurface("record_detail", {
      level: 6,
      title: "Acid Spray Fountain",
      traits: ["mechanical", "trap"],
    });
    const body = hazardBody(surface);
    body.defenses = {
      armor_class: 24,
      hit_points: { current: 56, maximum: 56, temporary: 0, broken_threshold: 28 },
      saves: { fortitude: 14, reflex: 13, will: 0 },
    };
    const acidSpray = strikeActivity("Acid Spray", "acid-spray", 0, 14);
    acidSpray.attack_mode = "melee";
    acidSpray.action_cost = { cost_type: "actions", count: 1 };
    acidSpray.traits = [];
    acidSpray.damage = [
      { damage_id: "acid-spray-0", formula: "2d10+8", damage_type: "acid" },
      {
        damage_id: "acid-spray-1",
        formula: "2d4",
        damage_type: "acid",
        category: "persistent",
      },
    ];
    body.activities = [acidSpray];

    render(<RecordSurface onReference={onReference} surface={surface} />);

    expect(screen.getByText("Melee Strike")).toBeInTheDocument();
    expect(screen.getByText("One action")).toHaveClass("sr-only");
    expect(screen.getByText("Attack +14")).toBeInTheDocument();
    expect(screen.getByText("2d10+8 acid")).toBeInTheDocument();
    expect(screen.getByText("2d4 acid persistent")).toBeInTheDocument();
  });

  it("preserves Tree of Dreadful Dreams authored activity order without exposing owner IDs", () => {
    const surface = hazardSurface("record_detail", {
      level: 10,
      title: "Tree of Dreadful Dreams",
      traits: ["magical", "trap"],
    });
    const body = hazardBody(surface);
    body.activities = [
      actionActivity("Independent Limbs", "tree-limbs", 0, {
        cost_type: "passive",
      }),
      actionActivity("Attack of Opportunity (Special)", "tree-reaction", 1, {
        cost_type: "reaction",
      }),
      strikeActivity("Branch", "tree-branch", 2, 26),
      actionActivity("Constrict", "tree-constrict", 3, {
        cost_type: "actions",
        count: 1,
      }),
      actionActivity("Terrifying Visions", "tree-visions", 4, {
        cost_type: "passive",
      }),
    ];

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );

    expect(
      [...container.querySelectorAll(".ant-collapse-header-text strong")].map(
        (heading) => heading.textContent,
      ),
    ).toEqual([
      "Independent Limbs",
      "Attack of Opportunity (Special)",
      "Branch",
      "Constrict",
      "Terrifying Visions",
    ]);
    expect(screen.getByText("One action")).toHaveClass("sr-only");
    expect(screen.getByText("Reaction")).toHaveClass("sr-only");
    expect(
      screen.queryByText(/tree-branch|tree-constrict|Entity |Occurrence /),
    ).toBeNull();
  });

  it("keeps False Door consumable content and links readable but nonexecuting with one limitation", () => {
    const surface = hazardSurface("record_detail", {
      level: 12,
      title: "False Door Trap",
      traits: ["magical", "mechanical"],
    });
    const body = hazardBody(surface);
    body.activities = [
      {
        occurrence_id: "false-door-venom-occurrence",
        entity_id: "false-door-venom-entity",
        authored_order: 0,
        source_ordinal: 1,
        identity_stability: "stable_source_identity",
        label: "Purple Worm Venom",
        activity_type: "unsupported_child",
        child_type: "consumable",
        traits: ["alchemical", "consumable", "injury", "poison"],
        content: [
          {
            ...activityContent("Purple Worm Venom remains safe reference content."),
            blocks: [
              {
                block_type: "paragraph",
                spans: [
                  { span_type: "text", text: "The venom can cause " },
                  {
                    span_type: "reference",
                    label: "Enfeebled",
                    record_key: "conditions:enfeebled",
                    embedded: false,
                  },
                  { span_type: "text", text: "." },
                ],
              },
            ],
          },
        ],
      },
    ];
    surface.issues = [
      {
        code: "unmodeled",
        placement: "activity",
        message:
          "Consumable inventory mechanics are not modeled for Purple Worm Venom.",
      },
    ];

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );

    expect(screen.getByText("Content only")).toBeInTheDocument();
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" &&
          element.textContent === "The venom can cause Enfeebled.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getAllByText(
        "Consumable inventory mechanics are not modeled for Purple Worm Venom.",
      ),
    ).toHaveLength(1);
    fireEvent.click(screen.getByRole("link", { name: "Enfeebled" }));
    expect(onReference).toHaveBeenLastCalledWith("conditions:enfeebled");
    expect(container.querySelector(".action-glyph")).toBeNull();
    expect(screen.queryByText("Source type: consumable")).toBeNull();
    expect(screen.queryByText(/false-door-venom-(entity|occurrence)/)).toBeNull();
  });

  it("omits absent sections and Missing, Null, and known-empty source facts", () => {
    const surface = hazardSurface("record_detail");
    const body = hazardBody(surface);
    body.complexity = undefined;
    body.size = undefined;
    body.detection = undefined;
    body.defenses = undefined;
    body.lifecycle = undefined;
    body.activities = undefined;
    body.content = undefined;
    body.unavailable_fields = [];
    body.provenance.source_metadata = [
      {
        field: "token_name",
        value: { state: "missing", source_path: "/prototypeToken/name" },
      },
      {
        field: "save_detail",
        save: "will",
        value: { state: "null", source_path: "/system/saves/will/saveDetail" },
      },
      {
        field: "strike_attack_effects_custom",
        entity_id: "silent-entity",
        value: {
          state: "typed",
          source_path: "/items/0/system/attackEffects/custom",
          value: "",
        },
      },
    ];
    surface.issues = [];
    surface.references = undefined;

    render(<RecordSurface onReference={onReference} surface={surface} />);

    for (const heading of [
      "Overview",
      "Detection & disable",
      "Defenses & Structure",
      "Activities",
      "Operation",
      "Additional content",
      "Data issues",
      "References",
    ]) {
      expect(screen.queryByRole("heading", { name: heading })).toBeNull();
    }
    openDisclosure("Source & provenance");
    expect(screen.queryByText("Token name")).toBeNull();
    expect(screen.queryByText("Will source note")).toBeNull();
    expect(screen.queryByText("Custom attack effect")).toBeNull();
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
    expect(
      screen.getByText("Initiative suggestion", { exact: false }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Hazard action content remains available."),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Lifecycle" })).toBeInTheDocument();
  });
});

function hazardSurface(
  profile: RecordSurfaceView["profile"],
  {
    level = 1,
    title = "Hidden Pit",
    traits = ["mechanical", "trap"],
  }: { level?: number; title?: string; traits?: string[] } = {},
): RecordSurfaceView {
  return {
    metadata: {
      record_key: `hazards:${title.toLowerCase().replace(/ /g, "-")}`,
      title,
      kind: "hazard",
      kind_label: "Hazard",
      level,
      rarity: "common",
      traits,
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
          details: paragraph("A concealed opening is difficult to notice."),
        },
        defenses: {
          armor_class: 22,
          hardness: 10,
          hit_points: {
            current: 30,
            maximum: 30,
            temporary: 0,
            broken_threshold: 15,
          },
          saves: { fortitude: 0, reflex: 8 },
        },
        lifecycle: {
          description: paragraph("The stones conceal a deep pit."),
          disable: paragraph("Thievery DC 12 opens the latch."),
          routine: paragraph("The trap resets without inferred automation."),
        },
        activities: [
          actionActivity("Routine", "occurrence-action", 0, {
            cost_type: "actions",
            count: 1,
          }),
          strikeActivity("Spikes", "occurrence-strike", 1, 11),
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
          source_metadata: [],
        },
      },
    },
  };
}

function hazardBody(surface: RecordSurfaceView) {
  if (surface.presentation.presentation_type !== "hazard") {
    throw new Error("Fixture must be a hazard surface");
  }
  return surface.presentation.body;
}

function actionActivity(
  label: string,
  occurrenceId: string,
  authoredOrder: number,
  actionCost: NonNullable<HazardSurfaceActivityView["action_cost"]>,
): HazardSurfaceActivityView {
  return {
    occurrence_id: occurrenceId,
    entity_id: `${occurrenceId}-entity`,
    authored_order: authoredOrder,
    source_ordinal: authoredOrder,
    identity_stability: "stable_source_identity",
    label,
    activity_type: "action",
    action_cost: actionCost,
    content: [activityContent()],
  };
}

function strikeActivity(
  label: string,
  occurrenceId: string,
  authoredOrder: number,
  attackBonus: number,
): HazardSurfaceActivityView {
  return {
    occurrence_id: occurrenceId,
    entity_id: `${occurrenceId}-entity`,
    authored_order: authoredOrder,
    source_ordinal: authoredOrder,
    identity_stability: "stable_source_identity",
    label,
    activity_type: "strike",
    attack_bonus: attackBonus,
    damage: [
      {
        damage_id: `${occurrenceId}-damage`,
        formula: "1d8",
        damage_type: "piercing",
      },
    ],
  };
}

function typedSourceFact(
  field: "token_name" | "has_health" | "temporary_maximum",
  value: string | boolean | number,
  sourcePath: string,
): HazardSurfaceSourceMetadataFactView {
  return {
    field,
    value: { state: "typed", source_path: sourcePath, value },
  } as HazardSurfaceSourceMetadataFactView;
}

function paragraph(text: string) {
  return [
    { block_type: "paragraph" as const, spans: [{ span_type: "text" as const, text }] },
  ];
}

function activityContent(text = "Hazard action content remains available.") {
  return {
    content_key: `activity-${text}`,
    role: "embedded_capability" as const,
    authored_order: 0,
    blocks: paragraph(text),
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

function openDisclosure(label: string) {
  const header = screen
    .getByText(label, { exact: true })
    .closest<HTMLElement>(".ant-collapse-header");
  if (!header) throw new Error(`Missing ${label} disclosure header`);
  fireEvent.click(header);
}
