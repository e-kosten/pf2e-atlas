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

  it("keeps sparse fact sections ordered for the staggered wide grid", () => {
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
    const activityHeading = screen
      .getByText("Dream Bargain")
      .closest<HTMLElement>(".creature-sheet__activity-heading");
    expect(activityHeading).not.toBeNull();
    expect(within(activityHeading!).getByText("2 actions")).toBeInTheDocument();
    expect(activityHeading?.children).toHaveLength(2);
    fireEvent.click(screen.getByText("Dream Bargain"));
    expect(screen.getByText("Will DC 28")).toBeInTheDocument();
    expect(container.querySelector(".creature-sheet__activity hr")).toBeInTheDocument();
  });

  it("opens typed spell content from compact occurrence links", () => {
    renderSurface();

    expect(screen.getByRole("heading", { name: "Spells" })).toBeInTheDocument();
    expect(screen.queryByText("Details", { exact: true })).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("Occult Innate Spells"));
    expect(screen.getByText("5th")).toBeInTheDocument();
    const innateLink = screen.getByRole("link", { name: "Dream Message" });
    expect(innateLink).toHaveAttribute("aria-haspopup", "dialog");
    expect(innateLink).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByText("The innate message reaches a sleeper."),
    ).not.toBeInTheDocument();
    fireEvent.focus(innateLink);
    expect(innateLink).toHaveAttribute("aria-expanded", "true");
    const innatePopover = screen.getByRole("dialog", {
      name: "Dream Message spell details",
    });
    expect(
      within(innatePopover).getByText("The innate message reaches a sleeper."),
    ).toBeInTheDocument();
    expect(within(innatePopover).getByText("5th")).toBeInTheDocument();
    fireEvent.click(
      within(innatePopover).getByRole("link", { name: "Open spell record" }),
    );
    expect(onReference.mock.calls[onReference.mock.calls.length - 1]?.[0]).toBe(
      "spells:dream-message",
    );
    fireEvent.keyDown(innateLink, { key: "Escape" });
    expect(innateLink).toHaveAttribute("aria-expanded", "false");
    expect(document.activeElement).toBe(innateLink);

    fireEvent.click(screen.getByText("Coven Spells"));
    expect(screen.getAllByRole("link", { name: "Dream Message" })).toHaveLength(2);
    const covenSpell = screen.getAllByRole("link", { name: "Dream Message" })[1]!;
    fireEvent.focus(covenSpell);
    const covenPopover = screen.getByRole("dialog", {
      name: "Dream Message spell details",
    });
    expect(
      within(covenPopover).getByText("The coven repeats the same authored spell."),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("The innate message reaches a sleeper."),
    ).not.toBeInTheDocument();

    const standaloneHeading = screen.getByRole("heading", {
      name: "Standalone",
    });
    const standalone = standaloneHeading.parentElement!;
    expect(within(standalone).getByText("8th")).toBeInTheDocument();
    expect(
      within(standalone).getAllByRole("link", { name: "Control Weather" }),
    ).toHaveLength(2);
    const standaloneLinks = within(standalone).getAllByRole("link", {
      name: "Control Weather",
    });
    expect(standaloneLinks[0]).toHaveAttribute("aria-expanded", "false");
    expect(standaloneLinks[1]).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("You alter the weather.")).not.toBeInTheDocument();

    fireEvent.focus(standaloneLinks[0]!);
    const standalonePopover = screen.getByRole("dialog", {
      name: "Control Weather spell details",
    });
    expect(within(standalonePopover).getByText("8th")).toBeInTheDocument();
    expect(
      within(standalonePopover).getByText("You alter the weather."),
    ).toBeInTheDocument();
    expect(screen.queryByText("A second ritual occurrence.")).not.toBeInTheDocument();

    fireEvent.mouseDown(document.body);
    fireEvent.click(document.body);
    expect(standaloneLinks[0]).toHaveAttribute("aria-expanded", "false");

    fireEvent.click(standaloneLinks[1]!);
    expect(
      within(
        screen.getByRole("dialog", { name: "Control Weather spell details" }),
      ).getByText("A second ritual occurrence."),
    ).toBeInTheDocument();
  });

  it("aligns creature and encounter resources without changing encounter vitals", () => {
    const { unmount } = renderSurface();
    const recordResources = screen.getByRole("heading", {
      name: "Resources",
    }).parentElement;
    expect(recordResources).not.toBeNull();
    const recordRow = within(recordResources!).getByText("Dream tokens").parentElement;
    expect(recordRow).toHaveClass("record-key-value-list__row");
    expect(within(recordRow!).getByText("3")).toBeInTheDocument();
    unmount();

    const surface = detailedSurfaceFixture();
    surface.profile = "encounter_participant";
    const runtime = encounterRuntimeFixture({ temporaryHp: 2 });
    const numberFact = (label: string, value: number) => ({
      label,
      base_value: value,
      adjusted_value: value,
      provenance: { source: { source_type: "canonical_record" as const } },
    });
    runtime.resources = [
      {
        resource_id: "dream-token",
        label: "Dream tokens",
        current: numberFact("Current dream tokens", 1),
        maximum: numberFact("Maximum dream tokens", 3),
      },
    ];
    surface.encounter = runtime;
    render(<RecordSurface onReference={onReference} surface={surface} />);

    const encounterResources = screen.getByRole("heading", {
      name: "Resources",
    }).parentElement;
    const encounterRow = within(encounterResources!).getByText(
      "Dream tokens",
    ).parentElement;
    expect(encounterRow).toHaveClass("record-key-value-list__row");
    expect(within(encounterRow!).getByText("1 / 3")).toBeInTheDocument();
    expect(
      screen.getByText("+2 temporary").closest(".record-surface__runtime-vitals"),
    ).not.toBeNull();
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
            content: [
              spellContent(
                "innate-dream-message",
                "Dream Message",
                "The innate message reaches a sleeper.",
              ),
            ],
          },
        ],
      },
      {
        occurrence_id: "coven",
        authored_order: 1,
        label: "Coven Spells",
        tradition: "occult",
        spells: [
          {
            occurrence_id: "coven-dream-message",
            authored_order: 1,
            label: "Dream Message",
            target_record_key: "spells:dream-message",
            rank: 5,
            content: [
              spellContent(
                "coven-dream-message",
                "Dream Message",
                "The coven repeats the same authored spell.",
              ),
            ],
          },
        ],
      },
    ],
    standalone_spells: [
      {
        occurrence_id: "control-weather-repeat",
        authored_order: 2,
        label: "Control Weather",
        target_record_key: "spells:control-weather",
        rank: 8,
        content: [
          spellContent(
            "control-weather-repeat",
            "Control Weather",
            "A second ritual occurrence.",
          ),
        ],
      },
      {
        occurrence_id: "control-weather",
        authored_order: 1,
        label: "Control Weather",
        target_record_key: "spells:control-weather",
        rank: 8,
        content: [
          spellContent("control-weather", "Control Weather", "You alter the weather."),
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
    provenance: {
      source_path: "fixture.json",
      source_contract_version: "hidden-contract",
      source_system_version: "hidden-system",
      source_upstream_commit: "hidden-commit",
    },
  };
  return surface;
}

function spellContent(
  contentKey: string,
  label: string,
  text: string,
): CreatureSurfaceContentView {
  return {
    content_key: contentKey,
    role: "embedded_capability",
    authored_order: 0,
    label,
    blocks: [
      {
        block_type: "paragraph",
        spans: [{ span_type: "text", text }],
      },
    ],
    content_hash: contentKey,
    visibility: "public",
    provenance: {
      source_record_key: "concept:f1-record",
      relative_source_path: "fixture.json",
      field_family: "fixture.spell",
    },
  };
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
