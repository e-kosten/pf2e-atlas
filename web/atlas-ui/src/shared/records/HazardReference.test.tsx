import { render, screen } from "@testing-library/react";
import { HazardReference } from "./HazardReference";
import {
  actorFixture,
  activityFixture,
  knownFact,
  numberFact,
} from "../../test/fixtures";
import type { ActorActivityView } from "../../generated/atlas";

it.each([false, true])(
  "keeps hazard activation before routine and additional actions, retaining early description and defenses (encounter=%s)",
  (encounter) => {
    const actor = actorFixture();
    const activities: ActorActivityView[] = [
      {
        ...activityFixture("Echoes of Defeat"),
        kind: "ability",
        ability_group: "activation",
        action_cost: knownFact({ label: "Free action", glyph: "free_action" }),
      },
      {
        ...activityFixture("Pitfall"),
        kind: "ability",
        ability_group: "activation",
        action_cost: knownFact({ label: "Reaction", glyph: "reaction" }),
      },
      activityFixture("Javelin"),
      {
        ...activityFixture("Additional action"),
        kind: "ability",
        ability_group: "primary",
      },
    ];
    actor.activities = knownFact(
      activities.map((a, index) => ({
        ...a,
        reading_order: index,
        navigation: {
          ...a.navigation,
          embedded_path: [
            { collection: "/items", identity: { Stable: `owned-${index}` } },
          ],
        },
      })),
    );
    render(
      <HazardReference
        hazard={{
          actor,
          complex: knownFact(true),
          stealth: numberFact(12),
          hardness: numberFact(5),
          detection_dc: numberFact(22),
          broken_threshold: numberFact(20),
          defense_block: knownFact(true),
          show_saves: true,
        }}
        encounter={encounter}
        encounterContent={
          encounter
            ? {
                participantKey: "hazard",
                durability: <h3>Live HP and defenses</h3>,
                conditions: <h3>Current conditions</h3>,
              }
            : undefined
        }
        content={{
          description: <p>Complete hazard description and operating rules.</p>,
          detection: <p>Detection qualifications</p>,
          disable: <h3>Disable</h3>,
          routine: <h3>Routine</h3>,
          reset: <h3>Reset</h3>,
          other: null,
        }}
        onReference={vi.fn()}
      />,
    );
    const headings = screen.getAllByRole("heading").map((h) => h.textContent);
    const ordered = [
      "Detection",
      encounter ? "Live HP and defenses" : "Defenses",
      "Disable",
      ...(encounter ? ["Current conditions"] : []),
      "Initial activation",
      "Routine",
      "Strikes",
      "Special abilities",
      "Reset",
    ];
    expect(headings.filter((heading) => ordered.includes(heading ?? ""))).toEqual(
      ordered,
    );
    expect(screen.getByText("Detection DC").parentElement).toHaveTextContent("22");
    if (!encounter)
      expect(screen.getByText("Broken Threshold").parentElement).toHaveTextContent(
        "20",
      );
    expect(screen.getByText("Echoes of Defeat")).toBeVisible();
    expect(screen.getByText("Pitfall")).toBeVisible();
    expect(
      screen
        .getByText("Complete hazard description and operating rules.")
        .compareDocumentPosition(screen.getByRole("heading", { name: "Detection" })) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  },
);

it.each([false, true, null])(
  "retains hazard rules and conditions without treating the generic creature budget as a hazard allotment (complex=%s)",
  (complex) => {
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
      senses: [],
      contextual_armor_class: [],
      unapplied_effects: [],
      action_budget: {
        actions: count("Actions", 3n),
        reactions: count("Reactions", 1n),
        can_act: { available: true },
        can_react: { available: true },
        quickened_sources: [],
        notes: [],
      },
    };
    render(
      <HazardReference
        hazard={{
          actor,
          complex:
            complex === null ? { state: "missing", value: null } : knownFact(complex),
          stealth: numberFact(12),
          hardness: numberFact(5),
          detection_dc: { state: "not_applicable", value: null, adjustment: null },
          broken_threshold: { state: "not_applicable", value: null, adjustment: null },
          defense_block: knownFact(true),
          show_saves: true,
        }}
        encounter
        encounterContent={{
          participantKey: "hazard",
          durability: <button>Edit HP</button>,
          conditions: <h3>Conditions</h3>,
        }}
        content={{
          description: <p>Complete operating rules</p>,
          detection: null,
          disable: <p>Disable instructions</p>,
          routine:
            complex === true ? <p>(2 actions) Follow the authored routine.</p> : null,
          reset: <p>Reset instructions</p>,
          other: null,
        }}
        onReference={vi.fn()}
      />,
    );
    expect(screen.queryByRole("region", { name: "Turn" })).toBeNull();
    expect(screen.queryByText("At turn start")).toBeNull();
    expect(screen.queryByText("Reactions now")).toBeNull();
    expect(screen.getByRole("heading", { name: "Conditions" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Edit HP" })).toBeEnabled();
    expect(screen.getByText("Reset instructions")).toBeVisible();
    if (complex === true)
      expect(
        screen.getByText("(2 actions) Follow the authored routine."),
      ).toBeVisible();
  },
);

it("omits the rules slot for a hazard with no visible activities or rules", () => {
  const actor = actorFixture();
  const absent = { ...numberFact(0), state: "not_applicable" as const, value: null };
  actor.saves.fortitude = absent;
  const hazard = {
    actor,
    complex: knownFact(false),
    stealth: absent,
    hardness: numberFact(0),
    detection_dc: absent,
    broken_threshold: numberFact(0),
    defense_block: knownFact(false),
    show_saves: true,
  };
  const content = {
    description: null,
    detection: null,
    disable: <p>Disable instructions</p>,
    routine: null,
    reset: null,
    other: null,
  };
  const props = {
    hazard,
    encounter: true,
    encounterContent: {
      participantKey: "hazard",
      durability: null,
      conditions: <p>Current conditions</p>,
    },
    onReference: vi.fn(),
  };
  const { container, rerender } = render(
    <HazardReference {...props} content={content} />,
  );
  expect(screen.getByText("Disable instructions")).toBeVisible();
  expect(screen.queryByText("AC")).toBeNull();
  expect(screen.queryByText("Stealth")).toBeNull();
  expect(screen.queryByText("Detection DC")).toBeNull();
  expect(screen.queryByText("Fortitude")).toBeNull();
  expect(screen.getByText("Will").parentElement).toHaveTextContent("+0");
  expect(container.querySelector(".actor-encounter-layout__rules")).toBeNull();

  rerender(
    <HazardReference
      {...props}
      content={{ ...content, reset: <p>Reset after 24 hours</p> }}
    />,
  );
  expect(screen.getByText("Reset after 24 hours")).toBeVisible();
});

it("omits Detection and durability entirely when only runtime content is visible", () => {
  const actor = actorFixture();
  const absent = { ...numberFact(0), state: "not_applicable" as const, value: null };
  actor.size = { state: "not_applicable", value: null };
  const { container } = render(
    <HazardReference
      hazard={{
        actor,
        complex: knownFact(false),
        stealth: absent,
        hardness: absent,
        detection_dc: absent,
        broken_threshold: absent,
        defense_block: knownFact(false),
        show_saves: false,
      }}
      encounter
      encounterContent={{
        participantKey: "hazard",
        durability: null,
        conditions: <p>Conditions</p>,
      }}
      content={{
        description: null,
        detection: null,
        disable: null,
        routine: null,
        reset: null,
        other: null,
      }}
      onReference={vi.fn()}
    />,
  );
  expect(screen.getByText("Conditions")).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Detection" })).toBeNull();
  expect(container.querySelector(".actor-encounter-layout__durability")).toBeNull();
});
