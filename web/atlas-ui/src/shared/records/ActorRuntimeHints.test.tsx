import { render, screen, within } from "@testing-library/react";
import { ActorActivities, ActorFacts, ActorSecondaryFacts } from "./ActorReference";
import { actorFixture, activityFixture, knownFact } from "../../test/fixtures";
import type { ActorRuntimeView, RuntimeCountView } from "../../generated/atlas";

function runtime(): ActorRuntimeView {
  const count = (label: string): RuntimeCountView => ({
    label,
    base_value: 0n,
    adjusted_value: 0n,
    segments: [],
    adjustments: [],
    suppressed_adjustments: [],
  });
  return {
    senses: [],
    contextual_armor_class: [],
    unapplied_effects: [],
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

it("omits repeated participant status but retains condition restrictions", () => {
  const actor = actorFixture();
  actor.runtime = runtime();
  const { rerender } = render(<ActorFacts actor={actor} encounter />);
  expect(screen.queryByText(/Marked defeated/)).not.toBeInTheDocument();
  actor.runtime.action_budget!.can_act = {
    available: false,
    source: "encounter_state",
    reason: "Stunned.",
  };
  rerender(<ActorFacts actor={actor} encounter />);
  expect(screen.getByText("No · Stunned")).toBeInTheDocument();
});

it("places the effective casting hint beside spellcasting without a duplicate runtime note", () => {
  const actor = actorFixture();
  actor.runtime = runtime();
  actor.runtime.action_budget = null;
  actor.runtime.unapplied_effects = [
    {
      source: "condition:stupefied",
      label: "Stupefied",
      reason: "Spellcasting requires a DC 10 flat check.",
    },
  ];
  actor.activities = knownFact([
    { ...activityFixture("Arcane spells"), kind: "casting_entry" },
  ]);
  render(
    <>
      <ActorSecondaryFacts actor={actor} />
      <ActorActivities actor={actor} encounter onReference={vi.fn()} />
    </>,
  );
  const section = screen
    .getByRole("heading", { name: "Spellcasting" })
    .closest("section")!;
  expect(
    within(section).getByText("Spellcasting requires a DC 10 flat check."),
  ).toBeInTheDocument();
  expect(screen.getAllByText(/Spellcasting requires a DC 10 flat check/)).toHaveLength(
    1,
  );
});
