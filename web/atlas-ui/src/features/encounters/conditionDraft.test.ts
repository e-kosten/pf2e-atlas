import type { EncounterParticipantConditionView } from "../../generated/atlas";
import {
  conditionDraft,
  editableConditionParameters,
  type ConditionFormValues,
} from "./conditionDraft";

const condition: EncounterParticipantConditionView = {
  condition_id: 7n,
  condition_key: "conditionitems:persistent",
  name: "Persistent Damage",
  scope: { kind: "everyone" },
  extra_action: { kind: "unspecified" },
  immunity_override: false,
  parameters: {
    kind: "persistent_damage",
    formula: "1d6",
    damage_type: "fire",
    multiplier: "normal",
    recovery: { kind: "standard" },
    episode_id: "episode",
  },
  duration: { amount: 2n, unit: "rounds", timing: null },
  created_at: "now",
  updated_at: "now",
};
const fields: ConditionFormValues = {
  durationUnit: "none",
  persistentFormula: "1d6",
  persistentDamageType: "fire",
  persistentRecoveryDc: 15,
  note: "updated",
};
const draft = (values = fields, source = condition, durationEdited = false) =>
  conditionDraft(values, durationEdited, 4n, "actor", undefined, source);

it.each(["standard", "source_dependent"] as const)(
  "preserves %s recovery and unresolved duration during an unrelated note edit",
  (kind) => {
    const source = {
      ...condition,
      parameters: {
        ...condition.parameters,
        kind: "persistent_damage" as const,
        formula: "1d6",
        damage_type: "fire",
        multiplier: "normal" as const,
        recovery: { kind },
      },
    };
    expect(draft(fields, source)).toMatchObject({
      note: "updated",
      duration: condition.duration,
      parameters: { recovery: { kind } },
    });
    expect(draft(fields, source).parameters).not.toHaveProperty("episode_id");
  },
);
it("distinguishes an explicitly cleared DC or duration from an untouched field", () => {
  expect(
    draft(
      { ...fields, persistentRecoveryDcEdited: true, persistentRecoveryDc: null },
      condition,
      true,
    ),
  ).toMatchObject({
    duration: undefined,
    parameters: { recovery: { kind: "source_dependent" } },
  });
  expect(
    draft({ ...fields, persistentRecoveryDcEdited: true, persistentRecoveryDc: 12 })
      .parameters,
  ).toMatchObject({ recovery: { kind: "specified", dc: 12n } });
});
it("sends incomplete and complete damage through the same edit input, without resolved identity", () => {
  expect(draft({ ...fields, persistentFormula: "" }).parameters).toEqual({
    kind: "persistent_damage",
    formula: null,
    damage_type: "fire",
    multiplier: "normal",
    recovery: { kind: "standard" },
  });
  expect(editableConditionParameters(condition.parameters)).toEqual({
    kind: "persistent_damage",
    formula: "1d6",
    damage_type: "fire",
    multiplier: "normal",
    recovery: { kind: "standard" },
  });
});

it.each([
  {
    keys: ["stride", "strike", "custom"] as const,
    guidance: "  Granted weapon only  ",
    expected: {
      kind: "restricted",
      actions: ["stride", "strike"],
      guidance: "Granted weapon only",
    },
  },
  {
    keys: ["stride"] as const,
    guidance: "Old hidden guidance",
    expected: { kind: "restricted", actions: ["stride"], guidance: null },
  },
  {
    keys: ["unrestricted"] as const,
    guidance: "Old hidden guidance",
    expected: { kind: "unrestricted" },
  },
  {
    keys: ["unspecified"] as const,
    guidance: "Old hidden guidance",
    expected: { kind: "unspecified" },
  },
])("builds extra actions from selected keys $keys", ({ keys, guidance, expected }) => {
  expect(
    draft({ ...fields, extraActionKeys: [...keys], extraActionGuidance: guidance })
      .extra_action,
  ).toEqual(expected);
});
