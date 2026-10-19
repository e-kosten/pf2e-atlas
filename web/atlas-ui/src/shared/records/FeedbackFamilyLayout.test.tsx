import { fireEvent, render, screen, within, waitFor } from "@testing-library/react";
import { ContentFields, RecordPresentation } from "./RecordPresentation";
import {
  actorFixture,
  activityFixture,
  detailFixture,
  knownFact,
  numberFact,
  unavailableFact,
} from "../../test/fixtures";

it("keeps encounter essentials before strikes and secondary facts available without changing reference density", async () => {
  const actor = actorFixture();
  actor.activities = knownFact([activityFixture("Jaws")]);
  actor.languages = knownFact(["Necril"]);
  actor.skills = knownFact([
    {
      key: "stealth",
      label: "Stealth",
      modifier: numberFact(0),
      note: unavailableFact("not_applicable"),
      conditional: knownFact([]),
    },
  ]);
  const detail = detailFixture("actors:ghoul", "Ghoul");
  detail.presentation.body = { kind: "creature", value: actor };
  const { rerender } = render(
    <RecordPresentation
      presentation={detail.presentation}
      encounter
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByText("AC")).toBeVisible();
  expect(screen.getByText("25 feet")).toBeVisible();
  expect(screen.getByText("Stealth")).toBeVisible();
  expect(screen.getByText("Necril")).toBeVisible();
  expect(screen.queryByRole("button", { name: "More creature facts" })).toBeNull();
  expect(
    screen
      .getByText("Necril")
      .compareDocumentPosition(screen.getByRole("link", { name: "Jaws" })) &
      Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
  expect(screen.getByText("Stealth").parentElement).toHaveTextContent("+0");
  expect(screen.getByText("Necril")).toBeVisible();
  expect(screen.getAllByText("AC")).toHaveLength(1);
  expect(screen.getAllByText("25 feet")).toHaveLength(1);
  rerender(
    <RecordPresentation presentation={detail.presentation} onReference={vi.fn()} />,
  );
  expect(screen.getByText("Stealth")).toBeVisible();
  expect(screen.getByText("Necril")).toBeVisible();
  expect(screen.queryByRole("button", { name: "More creature facts" })).toBeNull();
});

it.each([false, true])(
  "puts prepared flavor before creature facts in reference density compact=%s",
  (compact) => {
    const detail = detailFixture("actors:harrowkin", "Fabled Harrowkin");
    detail.presentation.body = { kind: "creature", value: actorFixture() };
    detail.presentation.content = [
      {
        locator: {
          record: detail.record.record_key,
          embedded_path: [],
          field: "/system/details/publicNotes",
        },
        role: "notes",
        source_fingerprint: null,
        body: { kind: "plain", text: "Living embodiments of harrow cards." },
      },
    ];
    const { rerender } = render(
      <RecordPresentation
        presentation={detail.presentation}
        onReference={vi.fn()}
        compact={compact}
      />,
    );
    expect(
      screen
        .getByText("Living embodiments of harrow cards.")
        .compareDocumentPosition(screen.getByRole("heading", { name: "Defenses" })) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    rerender(
      <RecordPresentation
        presentation={detail.presentation}
        onReference={vi.fn()}
        encounter
      />,
    );
    expect(screen.queryByText("Living embodiments of harrow cards.")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Source description" }));
    expect(
      screen
        .getByRole("heading", { name: "Defenses" })
        .compareDocumentPosition(
          screen.getByText("Living embodiments of harrow cards."),
        ) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  },
);

it("omits the service-selected empty hazard defense block without hiding IWR", () => {
  const actor = actorFixture();
  actor.immunities = knownFact([]);
  actor.weaknesses = knownFact([]);
  actor.resistances = knownFact([]);
  const detail = detailFixture("hazards:nightmare", "Waking Nightmare");
  detail.presentation.body = {
    kind: "hazard",
    value: {
      actor,
      complex: knownFact(true),
      stealth: numberFact(18),
      hardness: numberFact(0),
      defense_block: knownFact(false),
      show_saves: false,
    },
  };
  const { rerender } = render(
    <RecordPresentation presentation={detail.presentation} onReference={vi.fn()} />,
  );
  for (const label of [
    "AC",
    "HP",
    "Hardness",
    "Fortitude",
    "Reflex",
    "Will",
    "Defenses",
  ]) {
    expect(screen.queryByText(label)).toBeNull();
  }
  actor.immunities = knownFact([
    {
      damage_type: knownFact("mental"),
      magnitude: { state: "not_applicable", value: null, adjustment: null },
      exceptions: knownFact([]),
      double_against: knownFact([]),
    },
  ]);
  rerender(
    <RecordPresentation presentation={detail.presentation} onReference={vi.fn()} />,
  );
  expect(screen.getByText("mental")).toBeVisible();
});

it("renders selected casting membership with parent context, ranks and exact customized navigation", () => {
  const entry = {
    ...activityFixture("Occult Innate Spells"),
    kind: "casting_entry" as const,
  };
  const spell = {
    ...activityFixture("Local Dimension Door"),
    kind: "spell" as const,
    base_rank: numberFact(4),
    authored_cast_rank: numberFact(5),
    casting_ranks: [5],
    casting_entry: entry.navigation,
    association: knownFact("raw-casting-id"),
    navigation: {
      ...entry.navigation,
      embedded_path: [{ collection: "items", identity: { Stable: "custom-spell" } }],
    },
  };
  entry.casting_context = {
    parent: { ...entry.navigation, embedded_path: [], source_fingerprint: null },
    parent_title: "Fabled Harrowkin",
    spells: knownFact([spell]),
  };
  const detail = detailFixture("actors:ghoul", "Occult Innate Spells");
  detail.presentation.body = { kind: "activity", value: entry };
  const onReference = vi.fn();
  render(
    <RecordPresentation presentation={detail.presentation} onReference={onReference} />,
  );
  expect(
    screen.getByRole("heading", { name: "Spells for Fabled Harrowkin" }),
  ).toBeVisible();
  expect(screen.queryByText("Base rank 4")).toBeNull();
  expect(screen.getByText("Rank 5")).toBeVisible();
  expect(screen.queryByText("raw-casting-id")).toBeNull();
  expect(screen.queryByText("No structured damage")).toBeNull();
  fireEvent.click(screen.getByRole("link", { name: "Local Dimension Door" }));
  expect(onReference).toHaveBeenCalledWith(
    spell.navigation.record_key,
    expect.any(HTMLElement),
    { ...spell.navigation, spell_rank: 5 },
  );
});

it("omits known-empty optional headings while preserving unavailable and control-only content", () => {
  render(
    <ContentFields
      onReference={vi.fn()}
      fields={[
        {
          locator: { record: "items:one", embedded_path: [], field: "empty" },
          role: "description",
          source_fingerprint: null,
          body: { kind: "html", html: " ", controls: [] },
        },
        {
          locator: { record: "items:one", embedded_path: [], field: "unknown" },
          role: "notes",
          source_fingerprint: null,
          body: { kind: "unavailable", state: "invalid" },
        },
        {
          locator: { record: "items:one", embedded_path: [], field: "controls" },
          role: "routine",
          source_fingerprint: null,
          body: {
            kind: "html",
            html: "",
            controls: [
              {
                ordinal: 0,
                control: {
                  kind: "command",
                  command: "r",
                  arguments: "1d20",
                  options: {},
                },
              },
            ],
          },
        },
      ]}
    />,
  );
  expect(screen.queryByRole("heading", { name: "Description" })).toBeNull();
  expect(screen.getByRole("heading", { name: "Notes" })).toBeVisible();
  expect(screen.getByText(/invalid/)).toBeVisible();
  expect(screen.getByRole("heading", { name: "Routine" })).toBeVisible();
});

it.each([false, true])(
  "keeps independently applicable hazard zero saves visible (knownAbsent=%s)",
  (knownAbsent) => {
    const actor = actorFixture();
    actor.armor_class = numberFact(0);
    actor.maximum_hp = numberFact(0);
    actor.saves = {
      fortitude: numberFact(0),
      reflex: numberFact(0),
      will: numberFact(0),
    };
    const detail = detailFixture("hazards:nightmare", "Waking Nightmare");
    detail.presentation.body = {
      kind: "hazard",
      value: {
        actor,
        complex: knownFact(true),
        stealth: numberFact(18),
        hardness: numberFact(0),
        defense_block: knownAbsent ? knownFact(false) : unavailableFact("invalid"),
        show_saves: true,
      },
    };
    render(
      <RecordPresentation presentation={detail.presentation} onReference={vi.fn()} />,
    );
    expect(
      within(screen.getByText("Will").parentElement!).getByText("+0"),
    ).toBeVisible();
    expect(screen.queryByText("AC") === null).toBe(knownAbsent);
    if (knownAbsent) {
      expect(screen.queryByText("HP")).toBeNull();
      expect(screen.queryByText("Hardness")).toBeNull();
      expect(screen.queryByRole("button", { name: "Defense details" })).toBeNull();
    } else {
      expect(screen.getByText("Defense block").parentElement).toHaveTextContent(
        "invalid",
      );
    }
  },
);

it.each([false, true])(
  "preserves hazard condition explanations without creature facts or repeated budgets (hasBudget=%s)",
  async (hasBudget) => {
    const actor = actorFixture();
    const count = (label: string, value: bigint) => ({
      label,
      base_value: value,
      adjusted_value: value,
      segments: [],
      adjustments: [],
      suppressed_adjustments: [],
    });
    actor.runtime = {
      action_budget: hasBudget
        ? {
            actions: count("Actions", 0n),
            reactions: count("Reactions", 0n),
            can_act: { available: false, reason: "Unconscious blocks actions." },
            can_react: { available: false, reason: "Unconscious blocks reactions." },
            notes: [
              {
                source: "unconscious",
                label: "Unconscious",
                reason: "The supplied action budget is unavailable until recovery.",
              },
            ],
          }
        : null,
      unapplied_effects: [
        {
          source: "frightened",
          label: "Frightened 1",
          reason: "The hazard condition penalty has no supported projection.",
        },
      ],
    };
    const detail = detailFixture("hazards:nightmare", "Waking Nightmare");
    detail.presentation.body = {
      kind: "hazard",
      value: {
        actor,
        complex: knownFact(true),
        stealth: numberFact(18),
        hardness: numberFact(0),
        defense_block: knownFact(true),
        show_saves: true,
      },
    };
    const { rerender } = render(
      <RecordPresentation
        presentation={detail.presentation}
        encounter
        onReference={vi.fn()}
      />,
    );
    expect(screen.queryByText(/hazard condition penalty/)).toBeNull();
    if (hasBudget) {
      expect(screen.getByText("Actions").parentElement).toHaveTextContent("0");
      expect(screen.getByText("Reactions").parentElement).toHaveTextContent("0");
      expect(screen.getByText("Can act").parentElement).toHaveTextContent(
        "Unconscious blocks actions.",
      );
      expect(screen.getByText("Can react").parentElement).toHaveTextContent(
        "Unconscious blocks reactions.",
      );
    }
    fireEvent.click(screen.getByRole("button", { name: "Runtime details" }));
    await waitFor(() =>
      expect(screen.getByText(/hazard condition penalty/)).toBeVisible(),
    );
    if (hasBudget) {
      expect(screen.getByText(/supplied action budget/)).toBeVisible();
      for (const label of ["Actions", "Reactions", "Can act", "Can react"])
        expect(screen.getAllByText(label)).toHaveLength(1);
    }
    expect(screen.getAllByText("AC")).toHaveLength(1);
    expect(screen.queryByText("Languages")).toBeNull();
    expect(screen.queryByText("Perception details")).toBeNull();
    expect(
      screen.queryByRole("heading", { name: "Skills and ability modifiers" }),
    ).toBeNull();
    actor.runtime = { action_budget: null, unapplied_effects: [] };
    rerender(
      <RecordPresentation
        presentation={detail.presentation}
        encounter
        onReference={vi.fn()}
      />,
    );
    expect(screen.queryByRole("button", { name: "Runtime details" })).toBeNull();
  },
);

it.each([
  [1, numberFact(1), "Rank 1"],
  [1, numberFact(3), "Rank 3"],
  [0, numberFact(0), "Rank 0"],
  [1, { state: "missing", value: null, adjustment: null }, "Rank unavailable"],
  [1, { state: "invalid", value: null, adjustment: null }, "Rank unavailable"],
] as const)(
  "shows casting facts without substituting base rank %s",
  (base, cast, caption) => {
    const entry = {
      ...activityFixture("Innate spells"),
      kind: "casting_entry" as const,
    };
    const spell = {
      ...activityFixture("Fear"),
      kind: "spell" as const,
      base_rank: numberFact(base),
      authored_cast_rank: cast,
      casting_ranks: cast.state === "value" ? [Number(cast.value)] : [],
      casting_entry: entry.navigation,
    };
    entry.casting_context = {
      parent: entry.navigation,
      parent_title: "Mage",
      spells: knownFact([spell]),
    };
    const detail = detailFixture("actors:mage", "Innate spells");
    detail.presentation.body = { kind: "activity", value: entry };
    render(
      <RecordPresentation presentation={detail.presentation} onReference={vi.fn()} />,
    );
    expect(screen.getByText(caption)).toBeVisible();
    expect(screen.queryByText(/^Base rank/)).toBeNull();
    expect(screen.getByRole("link", { name: "Fear" })).toHaveAttribute("href");
  },
);

it("combines Lore and other skills with capitalized labels and signed modifiers", () => {
  const lore = {
    ...activityFixture("Dragon Lore"),
    kind: "lore" as const,
    lore_modifier: numberFact(10),
  };
  const actor = actorFixture();
  actor.activities = knownFact([lore]);
  actor.skills = knownFact([
    {
      key: "arcana",
      label: "arcana",
      modifier: numberFact(8),
      note: unavailableFact("not_applicable"),
      conditional: knownFact([]),
    },
  ]);
  const detail = detailFixture("actors:mage", "Mage");
  detail.presentation.body = { kind: "creature", value: actor };
  const onReference = vi.fn();
  render(
    <RecordPresentation presentation={detail.presentation} onReference={onReference} />,
  );
  expect(screen.queryByRole("heading", { name: "Lore" })).toBeNull();
  expect(screen.getAllByRole("heading", { name: "Skills" })).toHaveLength(1);
  expect(screen.getByText("Arcana").parentElement).toHaveTextContent("+8");
  expect(screen.getAllByText("Dragon Lore")).toHaveLength(1);
  expect(screen.getByText("Dragon Lore").parentElement).toHaveTextContent(
    "Dragon Lore +10",
  );
  expect(screen.queryByRole("link", { name: "Dragon Lore" })).toBeNull();
});
