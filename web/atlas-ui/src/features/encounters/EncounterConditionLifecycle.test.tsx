import { encounterParticipant } from "./encounterTestFixtures";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { EncounterConditionLifecycle } from "./EncounterConditionLifecycle";
import type {
  ConditionLifecycleOptionView,
  ConditionRecoveryOfferView,
} from "../../generated/atlas";
const api = vi.hoisted(() => ({
  getEncounterConditionLifecycleActions: vi.fn(),
  previewEncounterRetch: vi.fn(),
}));
vi.mock("../../api/atlasApi", () => api);
const participant = encounterParticipant();
function option(action: ConditionRecoveryOfferView): ConditionLifecycleOptionView {
  return {
    ...action,
    label: "Projected recovery",
    available: true,
    unavailable_reason: null,
  };
}
function setup(action: ConditionLifecycleOptionView, actor = participant) {
  api.getEncounterConditionLifecycleActions.mockResolvedValue({
    encounter_key: "encounter",
    encounter_revision: 4n,
    participant_key: "actor",
    actions: [action],
  });
  const onApply = vi.fn().mockResolvedValue({
    encounter_key: "encounter",
    request_id: "result",
    check: null,
  });
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterConditionLifecycle
          encounterRef="encounter"
          revision={4n}
          participant={actor}
          participants={[actor]}
          disabled={false}
          onApply={onApply}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  return onApply;
}
async function open() {
  fireEvent.click(await screen.findByText("Recovery and rest"));
  fireEvent.click(await screen.findByRole("button", { name: "Projected recovery" }));
}
async function choose(label: string, value: string) {
  fireEvent.mouseDown(await screen.findByRole("combobox", { name: label }));
  fireEvent.click(
    await screen.findByText(value, { selector: ".ant-select-item-option-content" }),
  );
}
it("renders a projected check and submits its typed table outcome", async () => {
  const onApply = setup({
    ...option({ kind: "dying_recovery", check: { dc: 13n, modifier: 0n } }),
  });
  await open();
  fireEvent.click(screen.getByText("Table outcome"));
  await choose("Check outcome", "Success");
  fireEvent.click(screen.getByRole("button", { name: "Record" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith({
      boundary_key: null,
      expected_encounter_revision: 4n,
      participant_key: "actor",
      action: {
        kind: "dying_recovery",
        check: { kind: "table_degree", degree: "success" },
      },
    }),
  );
});
it("requires a source-specific minimum instead of assuming a frightened floor", async () => {
  const actor = {
    ...participant,
    conditions: [
      {
        condition_id: 7n,
        name: "Alarm effect",
        scope: { kind: "everyone" as const },
        extra_action: { kind: "unspecified" as const },
        immunity_override: false,
        parameters: {
          kind: "frightened" as const,
          recovery: { kind: "source_dependent" as const },
        },
        created_at: "now",
        updated_at: "now",
      },
    ],
  };
  const onApply = setup(
    {
      ...option({ kind: "frightened_end_turn", decision_source_ids: [7n] }),
    },
    actor,
  );
  await open();
  await choose("Reduce Alarm effect", "Reduce");
  fireEvent.click(screen.getByRole("button", { name: "Record" }));
  expect(await screen.findByText("Enter the source minimum.")).toBeInTheDocument();
  expect(onApply).not.toHaveBeenCalled();
  fireEvent.change(
    screen.getByRole("spinbutton", { name: "Minimum for Alarm effect" }),
    { target: { value: "2" } },
  );
  fireEvent.click(screen.getByRole("button", { name: "Record" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: {
          kind: "frightened_end_turn",
          decisions: [{ condition_id: 7n, reduce: true, minimum: 2n }],
        },
      }),
    ),
  );
});
it("rechecks source-specific action answers and keeps known prohibitions authoritative", async () => {
  const issue = {
    source_condition_id: 3n,
    condition_name: "Source effect",
    kind: "needs_fact" as const,
    controller_key: null,
    subject: null,
    instructions: null,
    fact: "manipulate" as const,
    reason: "Does this use the affected action?",
  };
  api.previewEncounterRetch.mockImplementation(async (_ref, request) => ({
    encounter_key: "encounter",
    participant_key: "actor",
    expected_revision: 4n,
    expected_participant_revision: 1n,
    disruption: null,
    considers_target_ally: null,
    can_flank: true,
    targeting: null,
    confusion: null,
    quickened_use: null,
    issues: request.judgments.length
      ? request.judgments[0].value
        ? [
            {
              ...issue,
              kind: "prohibited",
              fact: null,
              reason: "The source prevents this action.",
            },
          ]
        : []
      : [issue],
  }));
  const onApply = setup({
    ...option({
      kind: "retch",
      condition_id: 7n,
      issues: [issue],
      check: { dc: 15n, modifier: 4n },
    }),
  });
  await open();
  fireEvent.click(screen.getByRole("checkbox", { name: "Actions spent" }));
  await choose(`${issue.reason} · Source 3`, "Yes");
  await screen.findByText(/The source prevents this action/);
  expect(screen.getByRole("button", { name: "Record" })).toBeDisabled();
  await choose(`${issue.reason} · Source 3`, "No");
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Record" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Record" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: expect.objectContaining({
          kind: "retch",
          judgments: [{ source_condition_id: 3n, fact: "manipulate", value: false }],
        }),
      }),
    ),
  );
});
it("declines a saved optional recovery check without inventing a failed roll", async () => {
  const onApply = setup({
    ...option({
      kind: "confused_damage_recovery",
      consequence_key: "damage-opportunity",
      source_condition_ids: [7n],
      qualifying_damage: true,
      check: { dc: 11n, modifier: 0n },
    }),
  });
  await open();
  fireEvent.click(screen.getByText("Decline check"));
  await waitFor(() =>
    expect(
      screen.queryByRole("combobox", { name: "Recovery sources" }),
    ).not.toBeInTheDocument(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Record" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: {
          kind: "confused_damage_recovery",
          consequence_key: "damage-opportunity",
          condition_ids: [],
          qualifying_damage: true,
          check: null,
        },
      }),
    ),
  );
});

it("records the server's Stunned action-loss forecast without an editable amount", async () => {
  const onApply = setup({
    ...option({ kind: "stunned_actions_lost", condition_id: 7n, actions_lost: 2n }),
  });
  await open();
  await waitFor(() => expect(screen.getByText("Actions lost: 2")).toBeVisible());
  expect(
    screen.queryByRole("spinbutton", { name: "Actions lost" }),
  ).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Record" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        action: { kind: "stunned_actions_lost", condition_id: 7n, actions_lost: 2n },
      }),
    ),
  );
});
