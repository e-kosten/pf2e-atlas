import { fireEvent, render, screen, within } from "@testing-library/react";
import type {
  CreatureSurfaceContentView,
  RecordSurfaceView,
} from "../../generated/atlas";
import {
  creatureSurfaceFixture,
  encounterRuntimeFixture,
} from "../../test/recordFixtures";
import { RecordSurface } from "./RecordSurface";

const onReference = vi.fn();

describe("RecordSurface", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders description before mechanics with record kind in identity metadata", () => {
    const { container } = renderSurface();

    const description = screen.getByRole("heading", { name: "Description & Lore" });
    const mechanics = screen.getByRole("heading", { name: "Actions & Abilities" });
    expect(
      description.compareDocumentPosition(mechanics) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      screen.getByRole("heading", { name: "Dream-Coven Envoy" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Level 9")).toBeInTheDocument();
    expect(screen.getByText("Creature")).toBeInTheDocument();
    expect(
      container.querySelector(".creature-sheet__snapshot"),
    ).not.toBeInTheDocument();
  });

  it("renders each core statistic exactly once", () => {
    renderSurface();

    for (const label of ["Perception", "HP", "AC", "Fortitude", "Reflex", "Will"]) {
      expect(screen.getAllByText(label, { exact: true })).toHaveLength(1);
    }
    const defenses = screen.getByRole("heading", {
      name: "Defenses & Vitals",
    }).parentElement;
    expect(defenses).not.toBeNull();
    const defenseStats = defenses!.querySelector(".creature-sheet__defense-stats");
    expect(defenseStats).toBeVisible();
    expect(defenseStats?.children).toHaveLength(5);
    for (const label of ["HP", "AC", "Fortitude", "Reflex", "Will"]) {
      const statLabel = within(defenses!).getByText(label, { exact: true });
      expect(statLabel).toBeInTheDocument();
      expect(statLabel.closest("dl")).toBe(defenseStats);
      expect(statLabel.closest(".record-key-value-list")).toBeNull();
    }
    expect(within(defenses!).getByText("Cold Iron 5")).toBeInTheDocument();
    expect(
      within(defenses!).queryByText("Perception", { exact: true }),
    ).not.toBeInTheDocument();

    const senses = screen.getByRole("heading", {
      name: "Senses & Languages",
    }).parentElement;
    expect(senses).not.toBeNull();
    expect(
      within(senses!).getByText("Perception", { exact: true }),
    ).toBeInTheDocument();
    expect(
      within(senses!).getByText("Perception", { exact: true }).closest("dl"),
    ).toHaveClass("creature-sheet__perception-stat");
    expect(
      within(senses!)
        .getByText("Perception", { exact: true })
        .closest(".record-key-value-list"),
    ).toBeNull();
    for (const label of ["HP", "AC", "Fortitude", "Reflex", "Will"]) {
      expect(
        within(senses!).queryByText(label, { exact: true }),
      ).not.toBeInTheDocument();
    }
  });

  it("keeps all movement modes together outside defenses", () => {
    renderSurface();

    const movement = screen.getByRole("heading", { name: "Movement" }).parentElement;
    expect(movement).not.toBeNull();
    expect(within(movement!).getByText("Speed")).toBeInTheDocument();
    expect(within(movement!).getByText("Fly")).toBeInTheDocument();
    expect(within(movement!).getByText("25 ft")).toBeInTheDocument();
    expect(within(movement!).getByText("40 ft")).toBeInTheDocument();
    expect(movement?.querySelectorAll(".record-key-value-list__row")).toHaveLength(2);
    for (const label of ["Speed", "Fly"]) {
      expect(within(movement!).getByText(label).parentElement).toHaveClass(
        "record-key-value-list__row",
      );
    }
    expect(movement?.querySelector(".creature-sheet__movement")).toBeNull();
  });

  it("keeps sparse fact sections semantic without empty shells", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.presentation.body.defenses = {
      ...surface.presentation.body.defenses!,
      weaknesses: undefined,
    };
    surface.presentation.body.abilities = undefined;
    surface.presentation.body.skills = [
      {
        component_id: "stealth",
        authored_order: 0,
        kind: "stealth",
        label: "Stealth",
        modifier: 5,
      },
    ];

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );
    const sections = container.querySelectorAll(
      ".creature-sheet__facts-grid > .creature-sheet__panel",
    );
    expect(sections).toHaveLength(4);
    expect(sections[0]).toHaveClass("creature-sheet__panel--defenses");
    expect(sections[1]).toHaveClass("creature-sheet__panel--senses");
    expect(sections[2]).toHaveClass("creature-sheet__panel--movement");
    expect(sections[3]).toHaveClass("creature-sheet__panel--skills");
  });

  it("expands typed activity content inline with check DC and divider structure", () => {
    const { container } = renderSurface();

    expect(screen.queryByText("View rules")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("Dream Bargain"));
    expect(screen.getByText("Will DC 28")).toBeInTheDocument();
    expect(container.querySelector(".creature-sheet__activity hr")).toBeInTheDocument();
  });

  it("uses grouped spellcasting and one disclosure per standalone occurrence", () => {
    renderSurface();

    fireEvent.click(screen.getByText("Occult Innate Spells"));
    expect(screen.getByText("5th")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Dream Message" })).toBeInTheDocument();

    const standaloneHeading = screen.getByRole("heading", {
      name: "Standalone Spells & Rituals",
    });
    const standaloneDisclosures = within(standaloneHeading.parentElement!).getAllByRole(
      "button",
      { name: /Control Weather/ },
    );
    expect(standaloneDisclosures).toHaveLength(2);
    expect(standaloneDisclosures[0]).toHaveAttribute("aria-expanded", "false");
    expect(standaloneDisclosures[1]).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("You alter the weather.")).not.toBeInTheDocument();

    fireEvent.click(standaloneDisclosures[0]!);
    expect(screen.getByText("You alter the weather.")).toBeInTheDocument();
    expect(screen.queryByText("A second ritual occurrence.")).not.toBeInTheDocument();

    fireEvent.click(standaloneDisclosures[1]!);
    expect(screen.getByText("A second ritual occurrence.")).toBeInTheDocument();
  });

  it("places record identity only in the inline provenance disclosure", () => {
    renderSurface();

    expect(screen.queryByText("concept:f1-record")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("References & Source"));
    expect(screen.getByText("Record ID")).toBeInTheDocument();
    expect(screen.getByText("concept:f1-record")).toBeInTheDocument();
    expect(screen.queryByText("source contract")).not.toBeInTheDocument();
    expect(screen.queryByText("upstream commit")).not.toBeInTheDocument();
  });

  it("uses aligned key-value grids only for natural definition groups", () => {
    const { container } = renderSurface();

    fireEvent.click(screen.getByText("References & Source"));
    for (const label of [
      "Immunities",
      "Weaknesses",
      "Resistances",
      "Senses",
      "Languages",
      "Record ID",
    ]) {
      const row = screen.getByText(label, { exact: true }).parentElement;
      expect(row).toHaveClass("record-key-value-list__row");
      expect(row?.children).toHaveLength(2);
    }
    expect(screen.getByText("Record ID").closest(".record-key-value-list")).toHaveClass(
      "record-key-value-list--provenance",
    );
    expect(
      container.querySelectorAll(".record-key-value-list").length,
    ).toBeGreaterThanOrEqual(3);
    for (const label of ["AC", "HP", "Fortitude", "Reflex", "Will", "Perception"]) {
      expect(
        screen.getByText(label, { exact: true }).closest(".record-key-value-list"),
      ).toBeNull();
    }
  });

  it("keeps description secondary in the encounter profile", () => {
    const surface = detailedSurfaceFixture();
    surface.profile = "encounter_participant";
    surface.encounter = encounterRuntimeFixture();

    render(<RecordSurface onReference={onReference} surface={surface} />);

    expect(screen.getByRole("button", { name: /Description & Lore/ })).toHaveAttribute(
      "aria-expanded",
      "false",
    );
    expect(
      screen.getByRole("heading", { name: "Combat Snapshot" }),
    ).toBeInTheDocument();
  });
});

function renderSurface() {
  return render(
    <RecordSurface onReference={onReference} surface={detailedSurfaceFixture()} />,
  );
}

function detailedSurfaceFixture(): RecordSurfaceView {
  const surface = creatureSurfaceFixture({
    level: 9,
    recordKey: "concept:f1-record",
    title: "Dream-Coven Envoy",
    traits: ["fiend", "hag", "unholy"],
  });
  if (surface.presentation.presentation_type !== "creature") {
    throw new Error("Fixture must be a creature surface");
  }
  surface.presentation.body = {
    ...surface.presentation.body,
    defenses: {
      ...surface.presentation.body.defenses!,
      immunities: [
        {
          component_id: "sleep",
          authored_order: 0,
          kind: "sleep",
        },
      ],
      resistances: [
        {
          component_id: "fire",
          authored_order: 0,
          kind: "fire",
          amount: 5,
        },
      ],
      weaknesses: [
        {
          component_id: "cold-iron",
          authored_order: 0,
          kind: "cold iron",
          amount: 5,
        },
      ],
    },
    awareness: {
      ...surface.presentation.body.awareness!,
      senses: [
        {
          component_id: "darkvision",
          authored_order: 0,
          kind: "darkvision",
        },
      ],
      languages: ["Aklo", "Common"],
    },
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
      },
    ],
    activities: [
      ...(surface.presentation.body.activities ?? []),
      {
        occurrence_id: "dream-bargain",
        authored_order: 1,
        activity_type: "action",
        label: "Dream Bargain",
        action_cost: { cost_type: "actions", count: 2 },
        content: [activityContent()],
      },
    ],
    spellcasting: [
      {
        occurrence_id: "innate",
        authored_order: 0,
        label: "Occult Innate Spells",
        tradition: "occult",
        spells: [
          {
            occurrence_id: "dream-message",
            authored_order: 0,
            label: "Dream Message",
            target_record_key: "spells:dream-message",
            rank: 5,
          },
        ],
      },
    ],
    content: [
      ...(surface.presentation.body.content ?? []),
      {
        content_key: "control-weather-repeat",
        role: "embedded_capability",
        authored_order: 2,
        label: "Control Weather",
        blocks: [
          {
            block_type: "paragraph",
            spans: [{ span_type: "text", text: "A second ritual occurrence." }],
          },
        ],
        content_hash: "control-weather-repeat",
        visibility: "public",
        provenance: {
          source_record_key: "concept:f1-record",
          relative_source_path: "fixture.json",
          field_family: "fixture.spell",
        },
      },
      {
        content_key: "control-weather",
        role: "embedded_capability",
        authored_order: 1,
        label: "Control Weather",
        blocks: [
          {
            block_type: "paragraph",
            spans: [{ span_type: "text", text: "You alter the weather." }],
          },
        ],
        content_hash: "control-weather",
        visibility: "public",
        provenance: {
          source_record_key: "concept:f1-record",
          relative_source_path: "fixture.json",
          field_family: "fixture.spell",
        },
      },
    ],
    provenance: {
      source_path: "fixture.json",
      source_contract_version: "hidden-contract",
      source_system_version: "hidden-system",
      source_upstream_commit: "hidden-commit",
    },
  };
  return surface;
}

function activityContent(): CreatureSurfaceContentView {
  return {
    content_key: "dream-bargain-content",
    role: "embedded_capability",
    authored_order: 0,
    label: "Dream Bargain",
    blocks: [
      {
        block_type: "paragraph",
        spans: [{ span_type: "text", text: "The envoy offers a bargain." }],
      },
      {
        block_type: "paragraph",
        spans: [
          {
            span_type: "check",
            display: "Will DC 28",
            statistic: "will",
            difficulty_class: 28,
          },
        ],
      },
      { block_type: "divider" },
    ],
    content_hash: "dream-bargain",
    visibility: "public",
    provenance: {
      source_record_key: "concept:f1-record",
      relative_source_path: "fixture.json",
      field_family: "fixture.activity",
    },
  };
}
