import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterConditionEditor } from "./EncounterConditionEditor";

it("preserves duration Stunned without manufacturing a numeric value", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterConditionEditor
      condition={{
        condition_id: 1n,
        condition_key: "conditionitems:stunned",
        name: "Stunned",
        duration: { amount: 1n, unit: "minutes", timing: null },
        created_at: "now",
        updated_at: "now",
      }}
      definitions={[
        {
          condition_ref: "conditionitems:stunned",
          name: "Stunned",
          supports_duration_instead_of_value: true,
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
      onCancel={() => {}}
    />,
  );
  expect(screen.queryByRole("spinbutton", { name: "Condition value" })).toBeNull();
  expect(screen.getByText("For a duration")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        value: undefined,
        durationAmount: 1,
        durationUnit: "minutes",
      }),
      false,
    ),
  );
});
