import { render, screen, within } from "@testing-library/react";
import {
  ActorActivities,
  ActorCombatBudget,
  ActorFacts,
  ActorSecondaryFacts,
} from "./ActorReference";
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

it("separates current reactions and Stunned restrictions from the next-turn action forecast", () => {
  const actor = actorFixture();
  actor.runtime = runtime();
  const budget = actor.runtime.action_budget!;
  budget.actions = {
    ...budget.actions,
    label: "Actions",
    base_value: 3n,
    adjusted_value: 2n,
  };
  budget.reactions = {
    ...budget.reactions,
    label: "Reactions",
    base_value: 1n,
    adjusted_value: 0n,
  };
  budget.can_act = { available: false, source: "encounter_state", reason: "Stunned." };
  budget.can_react = {
    available: false,
    source: "encounter_state",
    reason: "Stunned.",
  };
  budget.extra_action_unavailable_reason = "Lost to Stunned";
  render(<ActorCombatBudget actor={actor} />);
  const current = screen.getByRole("region", { name: "Right now" });
  expect(within(current).getByText("Reactions")).toBeVisible();
  expect(within(current).getAllByText("No · Stunned")).toHaveLength(2);
  expect(within(current).queryByText("Actions")).toBeNull();
  const forecast = screen.getByRole("region", { name: "At turn start" });
  expect(within(forecast).getByText("Actions")).toBeVisible();
  expect(within(forecast).getByText("Lost to Stunned")).toBeVisible();
  expect(within(forecast).queryByText("Reactions")).toBeNull();
});
