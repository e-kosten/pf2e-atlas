import { fireEvent, screen, waitFor } from "@testing-library/react";
import { EncounterConditionEditor as ConditionEditor } from "./EncounterConditionEditor";

import type { ComponentProps } from "react";
import { renderEncounter as render } from "./encounterTestFixtures";
const previewCondition = vi.fn().mockResolvedValue({
  encounter_key: "encounter",
  encounter_revision: 1n,
  consequences: [],
});
function EncounterConditionEditor(
  props: Omit<
    ComponentProps<typeof ConditionEditor>,
    "revision" | "onPreview" | "participantKey"
  > & { participantKey?: string },
) {
  return (
    <ConditionEditor
      revision={1n}
      participantKey="actor"
      onPreview={previewCondition}
      {...props}
    />
  );
}
async function apply(expected?: Record<string, unknown>) {
  if (expected)
    await waitFor(() =>
      expect(previewCondition).toHaveBeenLastCalledWith(
        expect.objectContaining(expected),
      ),
    );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
}

it("preserves duration Stunned without manufacturing a numeric value", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 1n,
        immunity_override: false,
        parameters: { kind: "none" },
        condition_key: "conditionitems:stunned",
        name: "Stunned",
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        duration: { amount: 1n, unit: "minutes", timing: null },
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[
        {
          condition_ref: "conditionitems:stunned",
          name: "Stunned",
          editor_kind: "ordinary",
          supports_duration_instead_of_value: true,
          scope_label: null,
          supports_extra_action: false,
          eligibility: { kind: "available" },
          has_value: true,
          default_value: 1n,
          automation_level: "automated",
          applies_to: [],
          categories: [],
        },
      ]}
      participants={[]}
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  expect(screen.queryByRole("spinbutton", { name: "Condition value" })).toBeNull();
  expect(screen.getByText("For a duration")).toBeVisible();
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        value: undefined,
        duration: { amount: 1n, unit: "minutes", timing: null },
      }),
    ),
  );
});

it("retains restricted extra actions and scoped participants independently of source and duration", async () => {
  const onSave = vi.fn().mockRejectedValue(new Error("Conflict"));
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 2n,
        immunity_override: false,
        parameters: { kind: "none" },
        condition_key: "conditionitems:example",
        name: "Example",
        scope: { kind: "selected_participants", participant_keys: ["opponent"] },
        extra_action: {
          kind: "restricted",
          actions: ["stride", "strike"],
          guidance: "Only with the granted weapon",
        },
        source_participant_key: "source",
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[
        {
          condition_ref: "conditionitems:example",
          name: "Example",
          editor_kind: "ordinary",
          supports_duration_instead_of_value: false,
          scope_label: "Against",
          supports_extra_action: true,
          eligibility: { kind: "available" },
          has_value: false,
          automation_level: "automated",
          applies_to: [],
          categories: [],
        },
      ]}
      participants={
        [
          { participant_key: "opponent", display_name: "Opponent" },
          { participant_key: "source", display_name: "Source" },
        ] as import("../../generated/atlas").EncounterParticipantView[]
      }
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  expect(screen.getByRole("combobox", { name: "Against" })).toBeVisible();
  expect(screen.queryByRole("combobox", { name: "Opponents" })).toBeNull();
  expect(screen.getByRole("textbox", { name: "Custom restriction" })).toBeVisible();
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        scope: { kind: "selected_participants", participant_keys: ["opponent"] },
        extra_action: {
          kind: "restricted",
          actions: ["stride", "strike"],
          guidance: "Only with the granted weapon",
        },
        source_participant_key: "source",
        duration: undefined,
      }),
    ),
  );
  expect(screen.getByRole("textbox", { name: "Custom restriction" })).toBeVisible();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Against" }));
  fireEvent.click(await screen.findByText("Everyone"));
  await apply({ scope: { kind: "everyone" } });
  await waitFor(() =>
    expect(onSave).toHaveBeenLastCalledWith(
      expect.objectContaining({ scope: { kind: "everyone" } }),
    ),
  );
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Against" }));
  const sourceOptions = await screen.findAllByText("Source");
  fireEvent.click(sourceOptions[sourceOptions.length - 1]);
  await apply({
    scope: { kind: "selected_participants", participant_keys: ["source"] },
  });
  await waitFor(() =>
    expect(onSave).toHaveBeenLastCalledWith(
      expect.objectContaining({
        scope: { kind: "selected_participants", participant_keys: ["source"] },
      }),
    ),
  );
});

it("offers a deliberate override through the submit action", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 3n,
        immunity_override: false,
        parameters: { kind: "none" },
        condition_key: "conditionitems:immune-example",
        name: "Example",
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[
        {
          condition_ref: "conditionitems:immune-example",
          name: "Example",
          editor_kind: "ordinary",
          supports_duration_instead_of_value: false,
          scope_label: null,
          supports_extra_action: false,
          eligibility: { kind: "immune", reason: "Immune to this condition" },
          has_value: false,
          automation_level: "automated",
          applies_to: [],
          categories: [],
        },
      ]}
      participants={[]}
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  expect(screen.queryByRole("textbox", { name: "GM override reason" })).toBeNull();
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({ immunity_override: true }),
    ),
  );
});

it("keeps optional details collapsed for basic entry and reveals them on request", async () => {
  render(
    <EncounterConditionEditor
      definitions={[]}
      participants={[]}
      disabled={false}
      pending={false}
      onSave={vi.fn()}
    />,
  );
  expect(screen.getByRole("button", { name: "Add duration" })).toBeVisible();
  expect(screen.queryByRole("textbox", { name: "Condition note" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: /^(Add|Hide) details$/ }));

  await waitFor(() =>
    expect(screen.getByRole("textbox", { name: "Condition note" })).toBeVisible(),
  );
});

it("defaults round timing to self and lets the GM choose another boundary and turn", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 4n,
        name: "Example",
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        immunity_override: false,
        parameters: { kind: "none" },
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[]}
      participants={
        [
          { participant_key: "a", display_name: "A" },
          { participant_key: "b", display_name: "B" },
        ] as import("../../generated/atlas").EncounterParticipantView[]
      }
      participantKey="a"
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Add duration" }));
  expect(
    await screen.findByRole("spinbutton", { name: "Duration amount" }),
  ).toHaveValue("1");
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        duration: {
          amount: 1n,
          unit: "rounds",
          timing: { event: "end", participant_key: "a" },
        },
      }),
    ),
  );
  fireEvent.click(screen.getByText("Starts on"));
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Whose turn" }));
  fireEvent.click(await screen.findByText("B’s turn"));
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        duration: {
          amount: 1n,
          unit: "rounds",
          timing: { event: "start", participant_key: "b" },
        },
      }),
    ),
  );
  const durationInput = screen.getByRole("spinbutton", { name: "Duration amount" });
  expect(fireEvent.keyDown(durationInput, { key: "-" })).toBe(false);
  expect(
    fireEvent.paste(durationInput, { clipboardData: { getData: () => "-3" } }),
  ).toBe(false);
  expect(durationInput).toHaveValue("1");
  // Out-of-range input from autofill or another input mechanism is clamped on blur.
  fireEvent.change(durationInput, { target: { value: "-3" } });
  fireEvent.blur(durationInput);
  expect(durationInput).toHaveValue("0");
  fireEvent.keyDown(durationInput, { key: "ArrowDown" });
  expect(durationInput).toHaveValue("0");
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        duration: {
          amount: 0n,
          unit: "rounds",
          timing: { event: "start", participant_key: "b" },
        },
      }),
    ),
  );
  expect(
    previewCondition.mock.calls.every(
      ([request]) => !request.duration || request.duration.amount >= 0n,
    ),
  ).toBe(true);
});

it.each(["pc", "creature"] as const)(
  "records a %s attitude without a confirmation step",
  async (participantKind) => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    render(
      <EncounterConditionEditor
        definitions={[
          {
            condition_ref: "conditionitems:test-attitude",
            name: "Attitude",
            editor_kind: "attitude",
            supports_duration_instead_of_value: false,
            scope_label: "Toward",
            supports_extra_action: false,
            eligibility: { kind: "available" },
            has_value: false,
            automation_level: "automated",
            applies_to: [],
            categories: [],
          },
        ]}
        participants={[]}
        participantKind={participantKind}
        disabled={false}
        pending={false}
        onSave={onSave}
      />,
    );
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Add condition" }));
    fireEvent.click(await screen.findByText("Attitude"));
    const reminder =
      participantKind === "pc"
        ? await screen.findByText(
            "Only supernatural effects can impose an attitude on a PC.",
          )
        : screen.queryByText(
            "Only supernatural effects can impose an attitude on a PC.",
          );
    if (participantKind === "pc") expect(reminder).toBeVisible();
    else expect(reminder).toBeNull();
    expect(screen.queryByRole("checkbox", { name: "Supernatural effect" })).toBeNull();
    await apply();
    await waitFor(() =>
      expect(onSave).toHaveBeenCalledWith(
        expect.objectContaining({ parameters: { kind: "attitude" } }),
      ),
    );
  },
);

it("keeps the fascinating subject separate from the effect's causal source", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 20n,
        condition_key: "conditions:fascinated",
        name: "Fascinated",
        parameters: {
          kind: "fascinated",
          subject: { kind: "other", description: "Dancing flame" },
        },
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        immunity_override: false,
        source_participant_key: "caster",
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[]}
      participants={
        [
          { participant_key: "caster", display_name: "Caster" },
        ] as import("../../generated/atlas").EncounterParticipantView[]
      }
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  expect(screen.getByRole("combobox", { name: "Subject" })).toBeVisible();
  expect(screen.getByRole("textbox", { name: "Subject description" })).toHaveValue(
    "Dancing flame",
  );
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        parameters: {
          kind: "fascinated",
          subject: { kind: "other", description: "Dancing flame" },
        },
        source_participant_key: "caster",
      }),
    ),
  );
});

it("edits a Fleeing subject independently of its causal source", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 21n,
        condition_key: "conditions:fleeing",
        name: "Fleeing",
        parameters: {
          kind: "fleeing",
          subject: { kind: "participant", participant_key: "threat" },
        },
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        immunity_override: false,
        source_participant_key: "caster",
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[]}
      participants={
        [
          { participant_key: "caster", display_name: "Caster" },
          { participant_key: "threat", display_name: "Threat" },
        ] as import("../../generated/atlas").EncounterParticipantView[]
      }
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  expect(screen.getByRole("combobox", { name: "Fleeing from" })).toBeInTheDocument();
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        parameters: {
          kind: "fleeing",
          subject: { kind: "participant", participant_key: "threat" },
        },
        source_participant_key: "caster",
      }),
    ),
  );
});

function persistentEditor(unknown = false) {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={
        unknown
          ? {
              condition_id: 1n,
              condition_key: "persistent",
              name: "Persistent Damage",
              parameters: {
                kind: "persistent_damage",
                formula: "1d6",
                damage_type: "fire",
                multiplier: "normal",
                recovery: { kind: "source_dependent" },
              },
              scope: { kind: "everyone" },
              extra_action: { kind: "unspecified" },
              immunity_override: false,
              note: "Existing note",
              duration: { amount: 2n, unit: "minutes", timing: null },
              created_at: "now",
              updated_at: "now",
            }
          : undefined
      }
      definitions={[
        {
          condition_ref: "persistent",
          name: "Persistent Damage",
          editor_kind: "persistent_damage",
          scope_label: null,
          supports_extra_action: false,
          supports_duration_instead_of_value: false,
          eligibility: { kind: "available" },
          has_value: false,
          automation_level: "automated",
          applies_to: [],
          categories: [],
        },
      ]}
      damageTypes={[{ value: "fire", label: "Fire" }]}
      standardRecoveryDc={15n}
      participants={[]}
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  return onSave;
}
it("offers one numeric default flat-check DC and one-row damage fields without recovery modes", async () => {
  const onSave = persistentEditor();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Add condition" }));
  fireEvent.click(await screen.findByText("Persistent Damage"));
  expect(await screen.findByRole("spinbutton", { name: "Flat check DC" })).toHaveValue(
    "15",
  );
  expect(screen.getByPlaceholderText("Formula, e.g. 1d6")).toHaveAccessibleName(
    "Damage formula",
  );
  expect(screen.getByRole("combobox", { name: "Damage type" })).toBeVisible();
  expect(screen.queryByRole("combobox", { name: "Persistent recovery" })).toBeNull();
  fireEvent.change(screen.getByRole("textbox", { name: "Damage formula" }), {
    target: { value: "2d6" },
  });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Damage type" }));
  fireEvent.click(await screen.findByText("Fire"));
  fireEvent.change(screen.getByRole("spinbutton", { name: "Flat check DC" }), {
    target: { value: "" },
  });
  await screen.findByText("Enter a whole number.");
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(onSave).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Flat check DC" }), {
    target: { value: "20" },
  });
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        parameters: expect.objectContaining({
          recovery: { kind: "specified", dc: 20n },
        }),
      }),
    ),
  );
});
it("keeps an existing unknown recovery DC blank while preserving unrelated source fields", async () => {
  const onSave = persistentEditor(true);
  expect(screen.getByRole("spinbutton", { name: "Flat check DC" })).toHaveValue("");
  fireEvent.change(screen.getByRole("textbox", { name: "Damage formula" }), {
    target: { value: "3d6" },
  });
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        parameters: expect.objectContaining({
          formula: "3d6",
          recovery: { kind: "source_dependent" },
        }),
        note: "Existing note",
        duration: expect.objectContaining({ amount: 2n }),
      }),
    ),
  );
});

it("accepts incomplete persistent damage without eager required-field errors", async () => {
  const onSave = persistentEditor();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Add condition" }));
  fireEvent.click(await screen.findByText("Persistent Damage"));
  expect(screen.queryByText("Enter a damage formula.")).toBeNull();
  expect(screen.queryByText("Choose a damage type.")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Add details" }));
  fireEvent.change(screen.getByRole("textbox", { name: "Condition note" }), {
    target: { value: "Burning until the blessing ends" },
  });
  await apply();
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({ note: "Burning until the blessing ends" }),
    ),
  );
});

it("switches duration Stunned to an exclusive action value", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 91n,
        condition_key: "conditionitems:stunned",
        name: "Stunned",
        immunity_override: false,
        parameters: { kind: "none" },
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        duration: { amount: 1n, unit: "minutes", timing: null },
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[
        {
          condition_ref: "conditionitems:stunned",
          name: "Stunned",
          editor_kind: "stunned",
          supports_duration_instead_of_value: true,
          scope_label: null,
          supports_extra_action: false,
          eligibility: { kind: "available" },
          has_value: true,
          default_value: 1n,
          automation_level: "automated",
          applies_to: [],
          categories: [],
        },
      ]}
      participants={[]}
      disabled={false}
      pending={false}
      onSave={onSave}
    />,
  );
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Stunned mode" }));
  fireEvent.click(await screen.findByText("Lose actions"));
  expect(
    await screen.findByRole("spinbutton", { name: "Condition value" }),
  ).toHaveValue("1");
  expect(screen.queryByRole("button", { name: "Add duration" })).toBeNull();
  expect(screen.queryByRole("spinbutton", { name: "Duration amount" })).toBeNull();
  await apply({ value: 1n, duration: undefined });
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({ value: 1n, duration: undefined }),
    ),
  );
});
