import { render, screen, within } from "@testing-library/react";
import {
  ActorActivities,
  ActorCombatBudget,
  ActorFacts,
  ActorSecondaryFacts,
} from "./ActorReference";
import { actorFixture, activityFixture, knownFact } from "../../test/fixtures";
import type {
  ActorRuntimeView,
  RuntimeCountView,
  ActorPresentationView,
} from "../../generated/atlas";

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
  expect(screen.getByText("Cannot act · Stunned")).toBeInTheDocument();
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
  // The HTTP contract carries JSON numbers even where generated Rust bindings use bigint.
  const wireActor: ActorPresentationView = JSON.parse(
    JSON.stringify(actor, (_key, value) =>
      typeof value === "bigint" ? Number(value) : value,
    ),
  );
  render(<ActorCombatBudget actor={wireActor} />);
  const turn = screen.getByRole("region", { name: "Turn" });
  expect(within(turn).getByText("Cannot act or react · Stunned")).toBeVisible();
  expect(within(turn).getByText("At turn start")).toBeVisible();
  expect(
    within(turn).getByRole("button", { name: /Show explanation for Actions:/ }),
  ).toHaveTextContent("2 actions");
  expect(within(turn).getByText("Lost to Stunned")).toBeVisible();
  expect(within(turn).queryByText("Reactions now")).toBeNull();
});

it("keeps available reactions distinct from the at-turn-start action forecast", () => {
  const actor = actorFixture();
  actor.runtime = runtime();
  const budget = actor.runtime.action_budget!;
  budget.can_act = { available: true, source: "encounter_state", reason: undefined };
  budget.can_react = { available: true, source: "encounter_state", reason: undefined };
  budget.reactions = { ...budget.reactions, base_value: 1n, adjusted_value: 1n };
  render(<ActorCombatBudget actor={actor} />);
  const turn = screen.getByRole("region", { name: "Turn" });
  expect(within(turn).getByText("Reactions now").parentElement).toHaveTextContent("1");
  expect(within(turn).getByText("At turn start")).toBeVisible();
  expect(within(turn).queryByText(/Cannot/)).toBeNull();
});

it("retains an unavailable reaction count beside a current restriction", () => {
  const actor = actorFixture();
  actor.runtime = runtime();
  const budget = actor.runtime.action_budget!;
  budget.can_react = {
    available: false,
    source: "encounter_state",
    reason: "Stunned.",
  };
  budget.reactions.adjusted_value = null;
  render(<ActorCombatBudget actor={actor} />);
  const turn = screen.getByRole("region", { name: "Turn" });
  expect(within(turn).getByText("Reactions now")).toBeVisible();
  expect(
    within(turn).getByRole("button", { name: /Show explanation for Reactions:/ }),
  ).toHaveTextContent("Needs review");
});
