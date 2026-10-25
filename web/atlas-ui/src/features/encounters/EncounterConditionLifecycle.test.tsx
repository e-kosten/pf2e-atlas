import { encounterParticipant } from "./encounterTestFixtures";
import { useState } from "react";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import {
  EncounterConditionRecovery,
  EncounterTurnRecoveryActions,
  useConditionRecoveryControls,
} from "./EncounterConditionLifecycle";
import type {
  ConditionLifecycleOptionView,
  ConditionLifecycleResultView,
  ConditionRecoveryOfferView,
  EncounterParticipantView,
} from "../../generated/atlas";
const api = vi.hoisted(() => ({ getEncounterConditionLifecycleActions: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
const condition = {
  condition_id: 1n,
  name: "Condition",
  value: 2n,
  scope: { kind: "everyone" as const },
  extra_action: { kind: "unspecified" as const },
  immunity_override: false,
  parameters: { kind: "none" as const },
  created_at: "now",
  updated_at: "now",
};
function option(action: ConditionRecoveryOfferView): ConditionLifecycleOptionView {
  return {
    ...action,
    condition_ids: [1n],
    label: "Recovery",
    available: true,
    unavailable_reason: null,
  };
}
function Rows({ actor }: { actor: EncounterParticipantView }) {
  const controls = useConditionRecoveryControls();
  return (
    <>
      {[
        ...actor.conditions,
        ...(controls.retainedCondition ? [controls.retainedCondition] : []),
      ].map((source) => (
        <div key={source.condition_id.toString()} data-testid="condition-row">
          {source.name}
          {controls.conditionActions(source.condition_id)}
        </div>
      ))}
      {controls.notice}
    </>
  );
}
function setup(action: ConditionLifecycleOptionView, removes = false) {
  api.getEncounterConditionLifecycleActions.mockResolvedValue({
    encounter_revision: 4n,
    actions: [action],
  });
  const onApply = vi.fn().mockResolvedValue({
    encounter_key: "encounter",
    request_id: "result",
    check: null,
  });
  function Harness() {
    const [actor, setActor] = useState(
      encounterParticipant({ conditions: [condition] }),
    );
    const [revision, setRevision] = useState(4n);
    return (
      <EncounterConditionRecovery
        encounterRef="encounter"
        revision={revision}
        participant={actor}
        disabled={false}
        onApply={async (request) => {
          const result = await onApply(request);
          if (removes) {
            api.getEncounterConditionLifecycleActions.mockResolvedValue({
              encounter_revision: 5n,
              actions: [],
            });
            setActor({ ...actor, conditions: [] });
            setRevision(5n);
          }
          return result;
        }}
      >
        <Rows actor={actor} />
      </EncounterConditionRecovery>
    );
  }
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <Harness />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  return onApply;
}
async function open() {
  fireEvent.click(await screen.findByRole("button", { name: "Recover Condition" }));
  await waitFor(() =>
    expect(screen.getByRole("dialog", { name: "Recovery" })).toBeVisible(),
  );
}
it("shows recovery guidance without forms, another clear button, or writes", async () => {
  const onApply = setup(
    option({
      kind: "reminder",
      condition_id: 1n,
      text: "Treat Wounds or rest at full HP for 10 minutes.",
    }),
  );
  await open();
  const popup = screen.getByRole("dialog", { name: "Recovery" });
  expect(within(popup).getByText(/Treat Wounds/)).toBeVisible();
  expect(within(popup).queryByRole("button")).toBeNull();
  expect(within(popup).queryByRole("spinbutton")).toBeNull();
  expect(onApply).not.toHaveBeenCalled();
});
it("uses a direct table outcome for Dying without a result-entry form", async () => {
  const onApply = setup(
    option({ kind: "dying_recovery", check: { dc: 12n, modifier: 0n } }),
  );
  await open();
  expect(screen.getByRole("button", { name: "Roll · DC 12" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Success" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: {
          kind: "dying_recovery",
          check: { kind: "table_degree", degree: "success" },
        },
      }),
    ),
  );
  expect(screen.queryByRole("combobox")).toBeNull();
});
it.each([
  { dc: null, modifier: 5n },
  { dc: 20n, modifier: null },
])("does not ask for missing Retch facts", async (check) => {
  const onApply = setup(option({ kind: "retch", condition_id: 1n, check, issues: [] }));
  await open();
  expect(screen.queryByRole("button", { name: /^Roll ·/ })).toBeNull();
  expect(screen.queryByRole("spinbutton")).toBeNull();
  expect(screen.queryByRole("checkbox")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Critical success" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: {
          kind: "retch",
          condition_id: 1n,
          check: { kind: "table_degree", degree: "critical_success" },
        },
      }),
    ),
  );
});
it("keeps known Retch as an optional integrated check", async () => {
  const onApply = setup(
    option({
      kind: "retch",
      condition_id: 1n,
      check: { dc: 20n, modifier: 5n },
      issues: [],
    }),
  );
  await open();
  fireEvent.click(screen.getByRole("button", { name: "Roll · DC 20" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: {
          kind: "retch",
          condition_id: 1n,
          check: { kind: "roll" },
        },
      }),
    ),
  );
});
it("shows unresolved restrictions as hints rather than action questions", async () => {
  setup(
    option({
      kind: "retch",
      condition_id: 1n,
      check: { dc: 20n, modifier: 5n },
      issues: [
        {
          kind: "needs_fact",
          source_condition_id: 2n,
          fact: "relates_to_fascination",
          reason: "Check whether this relates to the fascination.",
          subject: null,
          condition_name: "Fascinated",
          controller_key: null,
          instructions: null,
        },
      ],
    }),
  );
  await open();
  expect(
    screen.getByText("Check whether this relates to the fascination."),
  ).toBeVisible();
  expect(screen.queryByRole("combobox")).toBeNull();
  expect(screen.queryByRole("button", { name: /^Roll ·/ })).toBeNull();
  expect(screen.getByRole("button", { name: "Success" })).toBeEnabled();
});
it("reduces eligible Frightened directly without a minimum questionnaire", async () => {
  const onApply = setup(option({ kind: "frightened_end_turn" }));
  await open();
  fireEvent.click(screen.getByRole("button", { name: "Reduce by 1" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({ action: { kind: "frightened_end_turn" } }),
    ),
  );
  expect(screen.queryByRole("spinbutton")).toBeNull();
});
it("uses the server's Stunned loss and retains the row until the result closes", async () => {
  const onApply = setup(
    option({ kind: "stunned_actions_lost", condition_id: 1n, actions_lost: 2n }),
    true,
  );
  await open();
  fireEvent.click(screen.getByRole("button", { name: "Apply reduction" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: { kind: "stunned_actions_lost", condition_id: 1n, actions_lost: 2n },
      }),
    ),
  );
  await screen.findByText("Saved");
  expect(screen.getByTestId("condition-row")).toBeVisible();
  fireEvent.keyDown(screen.getByRole("dialog", { name: "Recovery" }), {
    key: "Escape",
  });
  await waitFor(() => expect(screen.queryByTestId("condition-row")).toBeNull());
});
it("uses the damage event's Confused scope without asking for qualifying facts or sources", async () => {
  const onApply = setup(
    option({
      kind: "confused_damage_recovery",
      consequence_key: "damage",
      source_condition_ids: [1n, 2n],
      qualifying_damage: null,
      check: { dc: 11n, modifier: 0n },
    }),
  );
  await open();
  expect(screen.getByText(/After damage from an attack or spell/)).toBeVisible();
  expect(screen.queryByRole("combobox")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Roll · DC 11" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: {
          kind: "confused_damage_recovery",
          consequence_key: "damage",
          check: { kind: "roll" },
        },
      }),
    ),
  );
});

it("offers Frightened reduction directly in turn reminders without opening a dice popover", async () => {
  const action = {
    ...option({ kind: "frightened_end_turn" }),
    label: "Reduce Frightened by 1",
  };
  api.getEncounterConditionLifecycleActions.mockResolvedValue({
    encounter_revision: 4n,
    actions: [action],
  });
  const onApply = vi.fn().mockImplementation(async () => {
    api.getEncounterConditionLifecycleActions.mockResolvedValue({
      encounter_revision: 5n,
      actions: [],
    });
    return { encounter_key: "encounter", request_id: "reduce", check: null };
  });
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterConditionRecovery
          encounterRef="encounter"
          boundaryKey="outgoing"
          contextualTurnActions
          revision={4n}
          participant={encounterParticipant({ conditions: [condition] })}
          disabled={false}
          onApply={onApply}
        >
          <EncounterTurnRecoveryActions />
        </EncounterConditionRecovery>
      </ConfigProvider>
    </QueryClientProvider>,
  );
  const button = await screen.findByRole("button", {
    name: "Reduce by 1",
  });
  expect(screen.queryByRole("dialog")).toBeNull();
  fireEvent.click(button);
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledExactlyOnceWith({
      boundary_key: "outgoing",
      expected_encounter_revision: 4n,
      participant_key: "actor",
      action: { kind: "frightened_end_turn" },
    }),
  );
  await waitFor(() =>
    expect(screen.queryByRole("button", { name: "Reduce by 1" })).toBeNull(),
  );
});

it("keeps a failed turn-recovery read visible with Retry", async () => {
  api.getEncounterConditionLifecycleActions.mockRejectedValue(new Error("Unavailable"));
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <EncounterConditionRecovery
        encounterRef="encounter"
        boundaryKey="outgoing"
        revision={4n}
        participant={encounterParticipant()}
        disabled={false}
        contextualTurnActions
        onApply={vi.fn()}
      >
        <EncounterTurnRecoveryActions />
      </EncounterConditionRecovery>
    </QueryClientProvider>,
  );
  expect(await screen.findByText("Recovery choices unavailable")).toBeVisible();
  expect(screen.queryByRole("button", { name: "Reduce by 1" })).toBeNull();
  api.getEncounterConditionLifecycleActions.mockResolvedValue({
    encounter_revision: 4n,
    actions: [
      {
        ...option({ kind: "frightened_end_turn" }),
        label: "Reduce Frightened by 1",
      },
    ],
  });
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByRole("button", { name: "Reduce by 1" })).toBeEnabled();
  expect(screen.queryByText("Recovery choices unavailable")).toBeNull();
});

it("retains the pending reduction control until the owner's canonical refresh finishes", async () => {
  const action = {
    ...option({ kind: "frightened_end_turn" }),
    label: "Reduce Frightened by 1",
  };
  api.getEncounterConditionLifecycleActions.mockResolvedValue({
    encounter_revision: 4n,
    actions: [action],
  });
  let finish!: () => void;
  const onApply = vi.fn(
    () =>
      new Promise<ConditionLifecycleResultView>((resolve) => {
        finish = () =>
          resolve({ encounter_key: "encounter", request_id: "reduce", check: null });
      }),
  );
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const show = (revision: bigint) => (
    <QueryClientProvider client={client}>
      <EncounterConditionRecovery
        encounterRef="encounter"
        boundaryKey="outgoing"
        revision={revision}
        participant={encounterParticipant()}
        disabled={false}
        contextualTurnActions
        onApply={onApply}
      >
        <EncounterTurnRecoveryActions />
      </EncounterConditionRecovery>
    </QueryClientProvider>
  );
  const view = render(show(4n));
  const button = await screen.findByRole("button", { name: /Reduce by 1$/ });
  button.focus();
  fireEvent.click(button);
  api.getEncounterConditionLifecycleActions.mockResolvedValue({
    encounter_revision: 5n,
    actions: [],
  });
  view.rerender(show(5n));
  await waitFor(() =>
    expect(
      client.getQueryData([
        "condition-lifecycle-actions",
        "encounter",
        "actor",
        "5",
        "outgoing",
      ]),
    ).toEqual({ encounter_revision: 5n, actions: [] }),
  );
  expect(screen.getByRole("button", { name: /Reduce by 1$/ })).toBe(button);
  expect(button).toBeDisabled();
  finish();
  await waitFor(() =>
    expect(screen.queryByRole("button", { name: /Reduce by 1$/ })).toBeNull(),
  );
});
