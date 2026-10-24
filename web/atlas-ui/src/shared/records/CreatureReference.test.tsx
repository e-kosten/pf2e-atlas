import { fireEvent, render, screen } from "@testing-library/react";
import { CreatureReference } from "./CreatureReference";
import { actorFixture, activityFixture, knownFact } from "../../test/fixtures";

it("prioritizes complete prose in exploration and retains it after encounter tasks", () => {
  const actor = actorFixture();
  const props = {
    actor,
    recordKey: "actors:ghoul",
    prose: <p>Complete source introduction.</p>,
    onReference: vi.fn(),
  };
  const { rerender } = render(<CreatureReference {...props} />);
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
