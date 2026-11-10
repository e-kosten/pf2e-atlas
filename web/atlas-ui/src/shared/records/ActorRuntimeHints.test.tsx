import { render, screen, within } from "@testing-library/react";
import { ActorActivities, ActorSkills } from "./ActorReference";
import { actorFixture, activityFixture, knownFact } from "../../test/fixtures";
import type { ActorRuntimeView, RuntimeCountView } from "../../generated/atlas";

function runtime(): ActorRuntimeView {
  const count = (label: string): RuntimeCountView => ({
    label,
    base_value: 0,
    adjusted_value: 0,
    segments: [],
    adjustments: [],
    suppressed_adjustments: [],
  });
  return {
    senses: [],
    contextual_armor_class: [],
    action_budget: {
      quickened_sources: [],
      notes: [],
      actions: count("Actions"),
      reactions: count("Reactions"),
      can_act: {
        available: false,
        source: "participant_status",
        reason: "Marked defeated.",
      },
      can_react: {
        available: false,
        source: "participant_status",
        reason: "Marked defeated.",
      },
    },
  };
}

it("places the effective casting hint beside spellcasting without a duplicate runtime note", () => {
  const actor = actorFixture();
  actor.runtime = runtime();
  actor.runtime.action_budget = null;
  actor.runtime.casting_hint = "Spellcasting requires a DC 10 flat check.";
  actor.activities = knownFact([
    { ...activityFixture("Arcane spells"), kind: "casting_entry" },
  ]);
  render(
    <>
      <ActorActivities actor={actor} encounter onReference={vi.fn()} />
    </>,
  );
  const section = screen
    .getByRole("heading", { name: "Spellcasting" })
    .closest("section")!;
  expect(
    within(section).getByText("Spellcasting requires a DC 10 flat check."),
  ).toBeInTheDocument();
  expect(screen.queryByRole("heading", { name: "Runtime" })).toBeNull();
  expect(screen.getAllByText(/Spellcasting requires a DC 10 flat check/)).toHaveLength(
    1,
  );
});

it("keeps Lore variant labels with the Lore modifier", () => {
  const actor = actorFixture();
  actor.activities = knownFact([
    {
      ...activityFixture("Sailing Lore"),
      kind: "lore",
      lore_modifier: { state: "value", value: 14, adjustment: null },
      notes: [
        { source: "Sailing Lore", label: "Storm navigation", reason: "terrain:storm" },
      ],
    },
  ]);
  render(<ActorSkills actor={actor} encounter />);
  expect(screen.getByText("Sailing Lore").parentElement).toHaveTextContent(
    "+14 · Storm navigation",
  );
  expect(screen.queryByText("terrain:storm")).toBeNull();
});
