import { ConfigProvider } from "antd";
import { fireEvent, render, screen } from "@testing-library/react";
import { EncounterTurnReview } from "./EncounterTurnReview";
import type { EncounterTurnReviewView } from "../../generated/atlas";
import { encounterParticipant } from "./encounterTestFixtures";
function outgoing(boundary_key: string, participant_name: string) {
  return {
    participant_removed: false,
    damage_applied: false,
    recovered_episode_ids: [],
    boundary_key,
    participant_name,
    participant_key: boundary_key,
    round_number: 2n,
    reminders: [
      {
        kind: "duration_reminder" as const,
        participant_key: boundary_key,
        participant_name,
        condition_id: 1n,
        condition_name: "Condition",
        duration: { amount: 1n, unit: "minutes" as const, timing: null },
      },
    ],
    persistent_damage_groups: [],
  };
}
function setup() {
  const onHandled = vi.fn();
  const onCancel = vi.fn();
  const data: EncounterTurnReviewView = {
    encounter_key: "encounter",
    expected_revision: 9n,
    current_participant_key: "incoming",
    next_participant_key: "next",
    reminders: [],
    outgoing: outgoing("latest", "Latest"),
    next_turn_unavailable_reason: null,
  };
  const props = {
    open: true,
    boundaryKey: "latest",
    data,
    participants: [],
    pending: false,
    disabled: false,
    loading: false,
    error: null,
    receipt: "",
    onRetry: vi.fn(),
    onCancel,
    encounterRef: "encounter",
    generation: null,
    damageTypes: [],
    onLifecycle: vi.fn(),
    onApplyPersistent: vi.fn(),
    onRecoverPersistent: vi.fn(),
    onRemove: vi.fn(),
  };
  const view = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnReview {...props} />
    </ConfigProvider>,
  );
  return { props, onHandled, onCancel, view };
}
it("shows only the current notification and the ordinary close affordance", () => {
  const { onHandled, onCancel } = setup();
  expect(
    screen.queryByRole("combobox", { name: "Outgoing turn" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "Handled at table" }),
  ).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Close" }));
  expect(onCancel).toHaveBeenCalledOnce();
  expect(onHandled).not.toHaveBeenCalled();
});
it("keeps removed participants' original identity and sources accessible without actionable HP or recovery controls", () => {
  const { props, view, onHandled } = setup();
  view.rerender(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnReview
        {...props}
        data={{
          ...props.data,
          outgoing: {
            ...props.data.outgoing!,
            participant_removed: true,
            persistent_damage_groups: [
              {
                episode_id: "fire",
                damage_type: "fire",
                selected_source_id: 7n,
                sources: [
                  {
                    condition_id: 7n,
                    formula: "1d6",
                    multiplier: "normal",
                    recovery_dc: 15n,
                  },
                ],
              },
            ],
          },
        }}
      />
    </ConfigProvider>,
  );
  expect(screen.getByText("Participant removed")).toBeInTheDocument();
  expect(screen.getByText("Persistent Damage · 1d6 fire")).toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: /Recover|Apply persistent damage/ }),
  ).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Close" }));
  expect(onHandled).not.toHaveBeenCalled();
});

it("offers direct removal beside an expired condition and retains its notice after removal", () => {
  const { props, view } = setup();
  const reminder = {
    kind: "duration_ended" as const,
    participant_key: "latest",
    participant_name: "Latest",
    condition_id: 5n,
    condition_name: "Blinded",
  };
  const onRemove = vi.fn().mockResolvedValue(undefined);
  const participant = encounterParticipant({
    participant_key: "latest",
    conditions: [
      {
        condition_id: 5n,
        name: "Blinded",
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        immunity_override: false,
        parameters: { kind: "none" },
        created_at: "",
        updated_at: "",
      },
    ],
  });
  const renderNotice = (present: boolean, pending = false) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnReview
        {...props}
        onRemove={onRemove}
        pending={pending}
        participants={[
          { ...participant, conditions: present ? participant.conditions : [] },
        ]}
        data={{
          ...props.data,
          outgoing: {
            ...props.data.outgoing!,
            reminders: present
              ? [reminder]
              : [{ ...reminder, kind: "resolved", result: "Removed" }],
          },
        }}
      />
    </ConfigProvider>
  );
  view.rerender(renderNotice(true));
  const remove = screen.getByRole("button", { name: "Remove Blinded from Latest" });
  expect(remove).toHaveTextContent("Remove condition");
  expect(screen.getByText("Duration ended")).toBeVisible();
  remove.focus();
  fireEvent.click(remove);
  expect(onRemove).toHaveBeenCalledExactlyOnceWith(reminder, "latest");
  view.rerender(renderNotice(true, true));
  expect(remove).toBeDisabled();
  view.rerender(renderNotice(false));
  expect(screen.queryByText("Duration ended")).toBeNull();
  expect(screen.getByRole("status")).toHaveTextContent("Removed");
  expect(screen.getByRole("status")).toHaveFocus();
  expect(screen.getByText("Blinded")).toBeVisible();
  expect(
    screen.queryByRole("button", { name: "Remove Blinded from Latest" }),
  ).toBeNull();
});

it("shows the supplied current Frightened value without guessing an unavailable value", () => {
  const { props, view } = setup();
  const renderValue = (condition_value: bigint | null) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnReview
        {...props}
        data={{
          ...props.data,
          outgoing: {
            ...props.data.outgoing!,
            reminders: [
              {
                kind: "condition_recovery",
                participant_key: "latest",
                participant_name: "Latest",
                condition_ids: [1n],
                condition_name: "Frightened",
                condition_value,
              },
            ],
          },
        }}
      />
    </ConfigProvider>
  );
  view.rerender(renderValue(2n));
  expect(screen.getByRole("heading", { name: "Frightened 2" })).toBeVisible();
  view.rerender(renderValue(null));
  expect(screen.getByRole("heading", { name: "Frightened" })).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Frightened 0" })).toBeNull();
});
