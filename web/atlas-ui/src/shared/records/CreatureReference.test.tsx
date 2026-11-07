import {
  fireEvent,
  render as renderWithContext,
  screen,
  within,
} from "@testing-library/react";
import { RouterTestContext } from "../../test/router";
import { ReferenceHelpProvider } from "./ReferenceHelp";
import { ActorActivity, ActorAwareness } from "./ActorReference";
import type { ActorActivityView, ReferenceHelpView } from "../../generated/atlas";
import { CreatureReference } from "./CreatureReference";
import {
  actorFixture,
  activityFixture,
  knownFact,
  numberFact,
} from "../../test/fixtures";

const render = (ui: React.ReactNode) =>
  renderWithContext(ui, { wrapper: RouterTestContext });

it("prioritizes complete prose in exploration and retains it after encounter tasks", () => {
  const actor = actorFixture();
  actor.movement_details = { state: "missing", value: null };
  const props = {
    actor,
    recordKey: "actors:ghoul",
    prose: <p>Complete source introduction.</p>,
    onReference: vi.fn(),
  };
  const { rerender } = render(<CreatureReference {...props} />);
  expect(screen.queryByText("Movement details")).toBeNull();
  expect(
    screen
      .getByText("Complete source introduction.")
      .compareDocumentPosition(screen.getByRole("heading", { name: "Defenses" })) &
      Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
  rerender(
    <CreatureReference
      {...props}
      encounter
      encounterContent={{
        participantKey: "p1",
        durability: <div>Existing HP controls</div>,
        conditions: <button>Add condition</button>,
      }}
    />,
  );
  expect(
    screen
      .getByText("Existing HP controls")
      .compareDocumentPosition(screen.getByText("Complete source introduction.")) &
      Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
  fireEvent.click(screen.getByRole("link", { name: "Actions & conditions" }));
  expect(
    screen.getByRole("button", { name: "Add condition" }).closest("section"),
  ).toHaveFocus();
});

it("keeps supplied strike rolls and casting annotations in the shared composition", () => {
  const actor = actorFixture();
  const strike = activityFixture();
  const casting = {
    ...activityFixture("Arcane spells"),
    kind: "casting_entry" as const,
  };
  actor.activities = knownFact([strike, casting]);
  const strikeAction = vi.fn();
  const resourceAction = vi.fn();
  render(
    <CreatureReference
      actor={actor}
      recordKey="actors:ghoul"
      prose={null}
      onReference={vi.fn()}
      encounter
      activityRolls={(activity) => (
        <button onClick={strikeAction}>Roll {activity.title}</button>
      )}
      castingAnnotations={{
        entry: () => <button onClick={resourceAction}>Restore slots</button>,
        rank: () => null,
        spell: () => ({ muted: false }),
      }}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Roll Jaws" }));
  fireEvent.click(screen.getByRole("button", { name: "Restore slots" }));
  expect(strikeAction).toHaveBeenCalledOnce();
  expect(resourceAction).toHaveBeenCalledOnce();
});

it("uses the same authored reading groups in reference and encounter with early statistics and retained action choices", () => {
  const actor = actorFixture();
  const activities: ActorActivityView[] = [
    {
      ...activityFixture("Passive ward"),
      kind: "ability",
      ability_group: "supporting",
      reading_order: 0,
    },
    {
      ...activityFixture("Offensive aura"),
      kind: "ability",
      ability_group: "primary",
      reading_order: 1,
    },
    {
      ...activityFixture("Swarming Bites"),
      kind: "ability",
      ability_group: "primary",
      reading_order: 0,
    },
    {
      ...activityFixture("Reactive Strike"),
      kind: "ability",
      ability_group: "reaction",
      reading_order: 0,
    },
    { ...activityFixture("Arcane spells"), kind: "casting_entry" },
    activityFixture("Jaws"),
  ];
  actor.activities = knownFact(
    activities.map((activity, index) => ({
      ...activity,
      navigation: {
        ...activity.navigation,
        embedded_path: [
          { collection: "/items", identity: { Stable: `activity-${index}` } },
        ],
      },
    })),
  );
  actor.movement_details = knownFact("suction");
  actor.skills = knownFact([
    {
      key: "athletics",
      label: "Athletics",
      modifier: { state: "value", value: 7, adjustment: null },
      note: knownFact(""),
      conditional: knownFact([]),
    },
  ]);
  const props = {
    actor,
    recordKey: "actors:ghoul",
    prose: <p>Creature introduction</p>,
    onReference: vi.fn(),
  };
  const view = render(<CreatureReference {...props} />);
  const check = () => {
    const titles = screen.getAllByRole("heading").map((h) => h.textContent);
    const ordered = [
      "Statistics",
      "Ability modifiers",
      "Skills",
      "Strikes",
      "Special abilities",
      "Spellcasting",
      "Reactions",
      "Passive abilities",
    ];
    expect(titles.filter((title) => ordered.includes(title ?? ""))).toEqual(ordered);
    expect(screen.getByRole("heading", { name: "Statistics", level: 3 })).toBeVisible();
    for (const name of ["Ability modifiers", "Skills"]) {
      expect(screen.getByRole("heading", { name, level: 4 })).toBeVisible();
    }
    expect(
      screen
        .getByText("Swarming Bites")
        .compareDocumentPosition(screen.getByText("Offensive aura")) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      screen
        .getByText("Perception")
        .compareDocumentPosition(
          screen.getByRole("heading", { name: "Ability modifiers" }),
        ) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(screen.getByText("suction")).toBeVisible();
  };
  check();
  view.rerender(<CreatureReference {...props} encounter />);
  check();
});

const senseHelp: ReferenceHelpView = {
  definition_key: "sense:darkvision",

  destination: {
    kind: "record",
    navigation: {
      record_key: "actions:darkvision",
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    },
  },
};

it("activates supplied categorical help while keeping actor qualifiers, modifiers and communication inline", () => {
  const actor = actorFixture();
  const onHelp = vi.fn();
  actor.senses = knownFact([
    {
      sense: knownFact({ value: "darkvision", label: "Darkvision", help: senseHelp }),
      acuity: knownFact("precise"),
      range_feet: numberFact(60),
    },
  ]);
  actor.languages = knownFact([
    {
      value: "common",
      label: "Common",
      help: { ...senseHelp, definition_key: "language:common" },
    },
    { value: "unknown", label: "Supplied language" },
  ]);
  actor.language_details = knownFact("Understands speech but cannot speak.");
  actor.abilities.value![0].help = { ...senseHelp, definition_key: "ability:str" };
  actor.skills = knownFact([
    {
      key: "athletics",
      label: "Athletics",
      help: { ...senseHelp, definition_key: "skill:athletics" },
      modifier: numberFact(8),
      note: knownFact("while climbing"),
      conditional: knownFact([]),
    },
  ]);
  actor.resistances = knownFact([
    {
      damage_type: knownFact({ value: "fire", label: "Fire", help: senseHelp }),
      magnitude: numberFact(5),
      exceptions: knownFact(["silver"]),
      double_against: knownFact(["magical fire"]),
    },
  ]);
  render(
    <ReferenceHelpProvider onHelp={onHelp}>
      <CreatureReference
        actor={actor}
        recordKey="actors:ghoul"
        prose={null}
        onReference={vi.fn()}
      />
    </ReferenceHelpProvider>,
  );
  expect(screen.getByText("Senses").parentElement).toHaveTextContent(
    "Darkvision precise 60 feet",
  );
  expect(screen.getByText("Perception").parentElement).toHaveTextContent("+7");
  expect(screen.getByText("Communication").parentElement).toHaveTextContent(
    "Understands speech but cannot speak.",
  );
  expect(screen.getByText("Resistances").parentElement).toHaveTextContent(
    "Fire 5 (except silver) (double against magical fire)",
  );
  expect(
    screen.getByRole("link", { name: "Athletics" }).closest(".record-fact"),
  ).toHaveTextContent("+8 · while climbing");
  expect(screen.queryByRole("link", { name: "Supplied language" })).toBeNull();
  fireEvent.click(screen.getByRole("link", { name: "Darkvision" }));
  expect(onHelp).toHaveBeenLastCalledWith(
    senseHelp,
    expect.any(HTMLElement),
    undefined,
  );
  fireEvent.click(screen.getByRole("link", { name: "Strength" }));
  expect(onHelp.mock.lastCall?.[0].definition_key).toBe("ability:str");
});

it("keeps runtime sense availability and Perception beside the supplied help", () => {
  const actor = actorFixture();
  actor.runtime = {
    senses: [
      {
        sense_type: { value: "darkvision", label: "Darkvision", help: senseHelp },
        precision: "precise",
        range_feet: 60,
        available: false,
        reason: "Blinded",
        perception: 4,
      },
    ],
    contextual_armor_class: [],
    action_budget: null,
    unapplied_effects: [],
  };
  render(
    <ReferenceHelpProvider onHelp={vi.fn()}>
      <ActorAwareness actor={actor} encounter />
    </ReferenceHelpProvider>,
  );
  expect(
    screen.getByRole("link", { name: "Darkvision" }).closest(".record-fact"),
  ).toHaveTextContent("precise · 60 feet · Unavailable (Blinded) · Perception +4");
});

it("keeps an owned ability's title destination primary with separate verified general help", () => {
  const activity = {
    ...activityFixture("Customized ward"),
    kind: "ability" as const,
    help: senseHelp,
  };
  const onReference = vi.fn();
  const onHelp = vi.fn();
  render(
    <ReferenceHelpProvider onHelp={onHelp}>
      <ActorActivity activity={activity} onReference={onReference} inline />
    </ReferenceHelpProvider>,
  );
  const title = within(screen.getByRole("heading")).getByRole("link", {
    name: "Customized ward",
  });
  fireEvent.click(title);
  expect(onReference).toHaveBeenCalledWith(
    activity.navigation.record_key,
    title,
    activity.navigation,
  );
  expect(onHelp).not.toHaveBeenCalled();
  const general = within(
    screen.getByText("General definition").parentElement!,
  ).getByRole("link");
  fireEvent.click(general);
  expect(onHelp).toHaveBeenCalledWith(senseHelp, general, undefined);
});
