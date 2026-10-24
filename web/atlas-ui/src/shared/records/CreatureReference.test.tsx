import { fireEvent, render, screen } from "@testing-library/react";
import type { ActorActivityView } from "../../generated/atlas";
import { CreatureReference } from "./CreatureReference";
import { actorFixture, activityFixture, knownFact } from "../../test/fixtures";

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
  fireEvent.click(screen.getByRole("link", { name: "Conditions" }));
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

it("uses the same authored reading groups in reference and encounter without moving supporting stats ahead of action choices", () => {
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
  const props = {
    actor,
    recordKey: "actors:ghoul",
    prose: <p>Creature introduction</p>,
    onReference: vi.fn(),
  };
  const view = render(<CreatureReference {...props} />);
  const check = () => {
    const titles = screen.getAllByRole("heading").map((h) => h.textContent);
    expect(titles.indexOf("Strikes")).toBeLessThan(titles.indexOf("Special abilities"));
    expect(
      screen
        .getByText("Swarming Bites")
        .compareDocumentPosition(screen.getByText("Offensive aura")) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(titles.indexOf("Special abilities")).toBeLessThan(
      titles.indexOf("Spellcasting"),
    );
    expect(titles.indexOf("Spellcasting")).toBeLessThan(titles.indexOf("Reactions"));
    expect(titles.indexOf("Reactions")).toBeLessThan(
      titles.indexOf("Passive abilities"),
    );
    expect(titles.indexOf("Passive abilities")).toBeLessThan(
      titles.indexOf("Statistics"),
    );
    expect(screen.getByText("suction")).toBeVisible();
  };
  check();
  view.rerender(<CreatureReference {...props} encounter />);
  check();
});
