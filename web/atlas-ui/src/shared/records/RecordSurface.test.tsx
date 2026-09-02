import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type {
  CreatureSurfaceContentView,
  RecordSurfaceView,
} from "../../generated/atlas";
import {
  creatureSurfaceFixture,
  encounterRuntimeFixture,
  occurrenceProvenance,
} from "../../test/recordFixtures";
import { RecordSurface } from "./RecordSurface";

const onReference = vi.fn();

describe("RecordSurface", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders description before mechanics with record kind in identity metadata", () => {
    const { container } = renderSurface();

    const description = screen.getByRole("heading", { name: "Overview" });
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
      expect(
        screen
          .getAllByText(label, { exact: true })
          .filter((element) => element.tagName === "DT"),
      ).toHaveLength(1);
    }
    const defenses = screen.getByRole("heading", {
      name: "Defenses & Vitals",
    }).parentElement;
    expect(defenses).not.toBeNull();
    const defenseStats = defenses!.querySelector(".creature-sheet__defense-stats");
    expect(defenseStats).toBeVisible();
    expect(defenseStats?.children).toHaveLength(6);
    for (const label of ["HP", "AC", "Hardness", "Fortitude", "Reflex", "Will"]) {
      const statLabel = within(defenseStats as HTMLElement).getByText(label, {
        exact: true,
      });
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

  it("renders typed profile, corrected ability modifiers, skill variants, and source context", () => {
    const { container } = renderSurface();

    const profile = container.querySelector('dl[aria-label="Creature profile"]');
    expect(screen.getByText("Medium")).toBeInTheDocument();
    expect(screen.getByText("Elite")).toBeInTheDocument();
    expect(screen.getByText("Initiative")).toBeInTheDocument();
    expect(profile).not.toBeNull();

    const abilities = screen.getByRole("heading", {
      name: "Ability Modifiers",
    }).parentElement!;
    for (const [label, value] of [
      ["Str", "+4"],
      ["Dex", "+3"],
      ["Con", "+4"],
      ["Int", "+5"],
      ["Wis", "+4"],
      ["Cha", "+5"],
    ]) {
      expect(within(abilities).getByText(label)).toBeInTheDocument();
      expect(within(abilities).getAllByText(value).length).toBeGreaterThan(0);
    }

    const skills = screen.getByRole("heading", { name: "Skills" }).parentElement!;
    expect(within(skills).getByText("Occultism")).toBeInTheDocument();
    expect(within(skills).getByText("Dreams")).toBeInTheDocument();
    expect(within(skills).getByText("dreams")).toBeInTheDocument();
    expect(within(skills).getByText("Source key")).toBeInTheDocument();
    expect(within(skills).getByText("occultism")).toBeInTheDocument();
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
    expect(within(activityHeading!).getByText("Two actions")).toBeInTheDocument();
    expect(activityHeading?.children).toHaveLength(2);
    fireEvent.click(screen.getByText("Dream Bargain"));
    expect(screen.getByText("Will DC 28")).toBeInTheDocument();
    expect(container.querySelector(".creature-sheet__activity hr")).toBeInTheDocument();
  });

  it("opens typed spell content from compact occurrence links", async () => {
    renderSurface();

    expect(screen.getByRole("heading", { name: "Spells" })).toBeInTheDocument();
    expect(screen.queryByText("Details", { exact: true })).not.toBeInTheDocument();
    const innateGroup = screen.getByRole("button", {
      name: /Occult Innate Spells/,
    });
    const covenGroup = screen.getByRole("button", { name: /Coven Spells/ });
    const standaloneGroup = screen.getByRole("button", { name: /Standalone/ });
    expect(innateGroup).toHaveAttribute("aria-expanded", "true");
    expect(covenGroup).toHaveAttribute("aria-expanded", "true");
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "true");
    expect(screen.getAllByText("5th")).toHaveLength(2);
    fireEvent.click(innateGroup);
    expect(innateGroup).toHaveAttribute("aria-expanded", "false");
    expect(covenGroup).toHaveAttribute("aria-expanded", "true");
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(innateGroup);
    expect(innateGroup).toHaveAttribute("aria-expanded", "true");
    const innateLink = screen.getAllByRole("link", { name: "Dream Message" })[0]!;
    expect(innateLink).toHaveAttribute("aria-haspopup", "dialog");
    expect(innateLink).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByText("The innate message reaches a sleeper."),
    ).not.toBeInTheDocument();
    fireEvent.pointerDown(innateLink);
    fireEvent.focus(innateLink);
    fireEvent.click(innateLink);
    expect(innateLink).toHaveAttribute("aria-expanded", "true");
    const innatePopover = screen.getByRole("dialog", {
      name: "Dream Message spell details",
    });
    expect(
      within(innatePopover).getByText("The innate message reaches a sleeper."),
    ).toBeInTheDocument();
    expect(within(innatePopover).getByText(/^5th ·/)).toBeInTheDocument();
    fireEvent.keyDown(innateLink, { key: "Escape" });
    expect(innateLink).toHaveAttribute("aria-expanded", "false");
    await waitFor(() => expect(document.activeElement).toBe(innateLink));
    fireEvent.click(innateLink);
    fireEvent.click(
      screen.getByRole("button", {
        name: "Open spell record",
      }),
    );
    expect(onReference.mock.calls[onReference.mock.calls.length - 1]?.[0]).toBe(
      "spells:dream-message",
    );

    expect(screen.getAllByRole("link", { name: "Dream Message" })).toHaveLength(2);
    const covenSpell = screen.getAllByRole("link", { name: "Dream Message" })[1]!;
    fireEvent.click(covenSpell);
    const covenPopover = screen.getByRole("dialog", {
      name: "Dream Message spell details",
    });
    expect(
      within(covenPopover).getByText("The coven repeats the same authored spell."),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("The innate message reaches a sleeper."),
    ).not.toBeInTheDocument();

    fireEvent.click(standaloneGroup);
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "false");
    expect(innateGroup).toHaveAttribute("aria-expanded", "true");
    expect(covenGroup).toHaveAttribute("aria-expanded", "true");
    fireEvent.click(standaloneGroup);
    expect(standaloneGroup).toHaveAttribute("aria-expanded", "true");
    const standalone = standaloneGroup.closest<HTMLElement>(".ant-collapse-item")!;
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

    fireEvent.click(standaloneLinks[0]!);
    const standalonePopover = screen.getByRole("dialog", {
      name: "Control Weather spell details",
    });
    expect(within(standalonePopover).getByText("8th")).toBeInTheDocument();
    expect(
      within(standalonePopover).getByText("You alter the weather."),
    ).toBeInTheDocument();
    expect(screen.queryByText("A second ritual occurrence.")).not.toBeInTheDocument();

    fireEvent.mouseDown(document.body);
    await waitFor(() =>
      expect(standaloneLinks[0]).toHaveAttribute("aria-expanded", "false"),
    );

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

  it("renders residual Heartstone content once in Overview without reclaiming occurrence content", () => {
    renderSurface();

    const overview = screen
      .getByRole("heading", { name: "Overview" })
      .closest("section");
    expect(overview).not.toBeNull();
    expect(within(overview!).getAllByText("Heartstone")).toHaveLength(1);
    expect(within(overview!).queryByText("Dream Bargain")).not.toBeInTheDocument();
    expect(
      screen.queryByText("The heartstone lets the envoy use ethereal jaunt."),
    ).not.toBeInTheDocument();

    fireEvent.click(within(overview!).getByRole("button", { name: "Heartstone" }));
    expect(
      screen.getAllByText("The heartstone lets the envoy use ethereal jaunt."),
    ).toHaveLength(1);

    fireEvent.click(screen.getByText("Dream Bargain"));
    expect(screen.getAllByText("The envoy offers a bargain.")).toHaveLength(1);
    expect(
      within(overview!).queryByText("The envoy offers a bargain."),
    ).not.toBeInTheDocument();
  });

  it("renders hardness, shield facts, and save details without duplicating owners", () => {
    renderSurface();

    const defenses = screen.getByRole("heading", {
      name: "Defenses & Vitals",
    }).parentElement!;
    expect(within(defenses).getByText("8")).toBeInTheDocument();
    const shield = within(defenses).getByRole("heading", {
      name: "Shield",
    }).parentElement!;
    expect(within(shield).getByText("+2")).toBeInTheDocument();
    expect(within(shield).getByText("5")).toBeInTheDocument();
    expect(within(shield).getByText("20")).toBeInTheDocument();
    expect(within(shield).getByText("10")).toBeInTheDocument();
    expect(within(defenses).getByText("+1 against curses")).toBeInTheDocument();
    expect(within(defenses).getByText("+2 against traps")).toBeInTheDocument();
    expect(within(defenses).getByText("+1 against magic")).toBeInTheDocument();
  });

  it("renders typed action details and strike effects with the shared action glyph", () => {
    const { container } = renderSurface();

    const claw = screen
      .getByText("Claw")
      .closest<HTMLElement>(".creature-sheet__activity")!;
    expect(within(claw).getByText("One action")).toBeVisible();
    expect(within(claw).getByText("Grab")).toBeInTheDocument();
    expect(within(claw).getByText("offensive")).toBeInTheDocument();

    const bargain = screen
      .getByText("Dream Bargain")
      .closest<HTMLElement>(".creature-sheet__activity")!;
    for (const value of [
      "Two actions",
      "1 per day",
      "The envoy can see the target.",
      "One dream token",
      "3 maximum",
      "Effect: Dream veil",
      "offensive",
    ]) {
      expect(within(bargain).getByText(value)).toBeInTheDocument();
    }
    expect(container.querySelectorAll(".action-glyph__mark")).toHaveLength(2);
  });

  it("renders ranked slots, occurrence context, ritual DC, gear, and Lore", () => {
    renderSurface();

    expect(screen.getByText("2 slots")).toBeInTheDocument();
    expect(
      screen.getByText("At will · Group innate · Slot 5 · 1 use"),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Rituals" })).toBeInTheDocument();
    expect(screen.getByText("Ritual DC")).toBeInTheDocument();
    expect(screen.getByText("31")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Equipment & Gear" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Silver key")).toBeInTheDocument();
    expect(
      screen.getByText("Quantity 1 · Level 9 · Usage held · 2 maximum uses"),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Lore" })).toBeInTheDocument();
    expect(screen.getByText("Dream Lore")).toBeInTheDocument();
    expect(screen.getByText("+20")).toBeInTheDocument();
  });

  it("keeps optional absence and known-empty data silent", () => {
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    surface.presentation.body.unavailable_domains = {
      skills: { causes: [] },
      equipment: { causes: [] },
    };
    surface.presentation.body.unmodeled_skills = [];

    render(<RecordSurface onReference={onReference} surface={surface} />);

    expect(
      screen.queryByRole("button", { name: "Data availability" }),
    ).not.toBeInTheDocument();
  });

  it("renders only legitimate availability causes with exact hostile skill copy", () => {
    const hostileKey = '<img src=x onerror="alert(1)">\n+20';
    const surface = detailedSurfaceFixture();
    if (surface.presentation.presentation_type !== "creature") {
      throw new Error("Fixture must be a creature surface");
    }
    const provenance = {
      owner: "canonical_creature" as const,
      field: "skills" as const,
    };
    surface.presentation.body.unavailable_domains = {
      defenses: {
        causes: [
          {
            state: "missing",
            field: "shield_hardness",
            provenance,
            message: "Shield hardness is unavailable.",
          },
        ],
      },
      spellcasting: {
        causes: [
          {
            state: "null",
            field: "spell_slot_maximum",
            provenance,
            message: "Spell slot maximum is unavailable.",
          },
        ],
      },
      skills: {
        causes: [
          {
            state: "unsupported",
            field: "unmodeled_skill",
            component_id: "opaque:skill:internal",
            provenance,
            unmodeled_skill: {
              component_id: "opaque:skill:internal",
              authored_order: 0,
              authored_key: hostileKey,
              base: { state: "null" },
              reason: "unknown_authored_key",
            },
            message: "The source supplied an unrecognized skill key.",
          },
        ],
      },
    };

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );
    const control = screen.getByRole("button", { name: "Data availability" });
    expect(control).toHaveAttribute("aria-expanded", "false");
    fireEvent.click(control);

    expect(screen.getByText("Missing")).toBeInTheDocument();
    expect(screen.getByText("Null")).toBeInTheDocument();
    expect(screen.getByText("Unsupported")).toBeInTheDocument();
    expect(screen.getByText("Shield hardness is unavailable.")).toBeInTheDocument();
    expect(screen.getByText("Spell slot maximum is unavailable.")).toBeInTheDocument();
    expect(screen.getByText("Unmodeled skill entry")).toBeInTheDocument();
    const authoredKey = container.querySelector(".creature-sheet__authored-value code");
    expect(authoredKey).toBeInTheDocument();
    expect(authoredKey?.textContent).toBe(hostileKey);
    expect(
      screen.getAllByText("The source supplied an unrecognized skill key.", {
        exact: true,
      }),
    ).toHaveLength(1);
    expect(container.querySelector("img")).not.toBeInTheDocument();
    expect(screen.queryByText("opaque:skill:internal")).not.toBeInTheDocument();
    expect(screen.queryByText(/meaning was inferred/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/not modeled/i)).not.toBeInTheDocument();
  });

  it("renders compact teaser and all six B3 scan facts in a responsive semantic grid", () => {
    const surface = detailedSurfaceFixture();
    surface.profile = "search_compact";

    const { container } = render(
      <RecordSurface onReference={onReference} surface={surface} />,
    );

    expect(
      screen.getByText("A dream-coven envoy that bargains before battle."),
    ).toBeVisible();
    const facts = container.querySelector(".record-surface-search__facts")!;
    expect(facts.tagName).toBe("DL");
    for (const label of ["Perception", "AC", "HP", "Fort", "Ref", "Will"]) {
      expect(
        within(facts as HTMLElement).getByText(label, { exact: true }),
      ).toBeVisible();
    }
    expect(
      container.querySelector(".record-surface--search-compact"),
    ).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Overview" })).not.toBeInTheDocument();
  });

  it("navigates only exact verified Air edition counterparts", () => {
    const airMephit = detailedSurfaceFixture();
    airMephit.metadata.title = "Air Mephit";
    airMephit.metadata.edition = {
      status: "legacy",
      counterparts: [
        {
          role: "remastered_counterpart",
          record_key: "monster-core:air-scamp",
          title: "Air Scamp",
        },
      ],
    };
    const { rerender } = render(
      <RecordSurface onReference={onReference} surface={airMephit} />,
    );
    expect(screen.getByText("This record uses legacy rules.")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "View remastered Air Scamp" }));
    expect(onReference).toHaveBeenLastCalledWith("monster-core:air-scamp");

    const airScamp = detailedSurfaceFixture();
    airScamp.metadata.title = "Air Scamp";
    airScamp.metadata.edition = {
      status: "remaster",
      counterparts: [
        {
          role: "legacy_counterpart",
          record_key: "bestiary:air-mephit",
          title: "Air Mephit",
        },
      ],
    };
    rerender(<RecordSurface onReference={onReference} surface={airScamp} />);
    expect(screen.getByText("This record uses remastered rules.")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "View legacy Air Mephit" }));
    expect(onReference).toHaveBeenLastCalledWith("bestiary:air-mephit");
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
      for (const element of screen.getAllByText(label, { exact: true })) {
        expect(element.closest(".record-key-value-list")).toBeNull();
      }
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
  surface.metadata.edition = {
    status: "legacy",
    counterparts: [],
  };
  surface.presentation.body = {
    ...surface.presentation.body,
    teaser: "A dream-coven envoy that bargains before battle.",
    size: {
      value: "medium",
      provenance: { owner: "canonical_creature", field: "size" },
    },
    adjustment: {
      value: "elite",
      provenance: { owner: "canonical_creature", field: "adjustment" },
    },
    initiative: {
      statistic: "perception",
      provenance: { owner: "canonical_creature", field: "initiative" },
    },
    defenses: {
      ...surface.presentation.body.defenses!,
      hardness: 8,
      shield: {
        armor_class_bonus: 2,
        broken_threshold: 10,
        hardness: 5,
        maximum_hit_points: 20,
      },
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
    saves: {
      ...surface.presentation.body.saves!,
      fortitude: {
        ...surface.presentation.body.saves!.fortitude!,
        details: "+1 against curses",
      },
      reflex: {
        ...surface.presentation.body.saves!.reflex!,
        details: "+2 against traps",
      },
      all_saves_note: "+1 against magic",
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
    abilities: {
      strength: 4,
      dexterity: 3,
      constitution: 4,
      intelligence: 5,
      wisdom: 4,
      charisma: 5,
      provenance: { owner: "canonical_creature", field: "legacy_abilities" },
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
    skills: [
      {
        component_id: "occultism",
        authored_order: 0,
        kind: "occultism",
        label: "Occultism",
        modifier: 22,
        variants: [
          {
            component_id: "dreams",
            authored_order: 0,
            label: "Dreams",
            modifier: 24,
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
    activities: [
      ...(surface.presentation.body.activities ?? []).map((activity) => ({
        ...activity,
        attack_effects: ["Grab"],
        category: "offensive",
      })),
      {
        occurrence_id: "dream-bargain",
        authored_order: 1,
        provenance: occurrenceProvenance,
        activity_type: "action",
        label: "Dream Bargain",
        action_cost: { cost_type: "actions", count: 2 },
        category: "offensive",
        frequency: { maximum: 1, period: "day" },
        requirements: "The envoy can see the target.",
        cost: "One dream token",
        uses: { maximum: 3 },
        self_effect: { label: "Effect", value: "Dream veil" },
        content: [activityContent()],
      },
    ],
    spellcasting: [
      {
        occurrence_id: "innate",
        authored_order: 0,
        provenance: occurrenceProvenance,
        label: "Occult Innate Spells",
        tradition: "occult",
        preparation: "innate",
        difficulty_class: 28,
        attack_modifier: 20,
        slots: [{ rank: 5, maximum: 2 }],
        spells: [
          {
            occurrence_id: "dream-message",
            authored_order: 0,
            provenance: occurrenceProvenance,
            label: "Dream Message",
            target_record_key: "spells:dream-message",
            rank: 5,
            context: {
              contextual_label: "At will",
              group: "innate",
              slot: "5",
              uses: { maximum: 1 },
            },
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
        provenance: occurrenceProvenance,
        label: "Coven Spells",
        tradition: "occult",
        spells: [
          {
            occurrence_id: "coven-dream-message",
            authored_order: 1,
            provenance: occurrenceProvenance,
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
        provenance: occurrenceProvenance,
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
        provenance: occurrenceProvenance,
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
    rituals: {
      difficulty_class: 31,
      provenance: { owner: "canonical_creature", field: "embedded_entities" },
    },
    equipment: [
      {
        occurrence_id: "silver-key",
        authored_order: 0,
        provenance: occurrenceProvenance,
        label: "Silver key",
        traits: ["magical"],
        level: 9,
        usage: "held",
        quantity: 1,
        uses: { maximum: 2 },
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
      ...(surface.presentation.body.content ?? []),
      spellContent(
        "heartstone",
        "Heartstone",
        "The heartstone lets the envoy use ethereal jaunt.",
      ),
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
