import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import type { ComponentProps } from "react";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";

it.each([true, false])(
  "retires the committed editor after controls re-enable without reopening on later writes (refresh=%s)",
  async (refreshed) => {
    let resolveSave!: (value: boolean) => void;
    const saved = new Promise<boolean>((resolve) => {
      resolveSave = resolve;
    });
    const props: ComponentProps<typeof EncounterConditionControls> = {
      disabled: false,
      pending: false,
      addDisabled: false,
      encounterRef: "encounter",
      encounterRevision: 1n,
      current: encounterParticipant({
        conditions: [
          {
            condition_id: 1n,
            condition_key: "conditionitems:frightened",
            name: "Frightened",
            value: 1n,
            scope: { kind: "everyone" },
            parameters: { kind: "none" },
            extra_action: { kind: "unspecified" },
            immunity_override: false,
            created_at: "now",
            updated_at: "now",
          },
        ],
      }),
      conditionDefinitions: [
        {
          condition_ref: "conditionitems:frightened",
          name: "Frightened",
          editor_kind: "ordinary",
          supports_duration_instead_of_value: false,
          scope_label: null,
          supports_extra_action: false,
          eligibility: { kind: "available" },
          has_value: true,
          default_value: 1n,
          automation_level: "automated",
          applies_to: [],
          categories: [],
        },
      ],
      onAddCondition: vi.fn(),
      onRemoveCondition: vi.fn(),
      onUpdateCondition: vi.fn().mockReturnValue(saved),
      onPreviewCondition: vi.fn().mockResolvedValue({
        encounter_key: "encounter",
        encounter_revision: 1n,
        consequences: [],
      }),
      onReference: vi.fn(),
      participants: [],
    };
    const { rerender } = renderEncounter(<EncounterConditionControls {...props} />);
    const edit = screen.getByRole("button", { name: "Edit Frightened" });
    edit.focus();
    fireEvent.click(edit);
    const dialog = await screen.findByRole("dialog", { name: "Edit Frightened" });
    const apply = within(dialog).getByRole("button", { name: "Apply" });
    await waitFor(() => expect(apply).toBeEnabled());
    fireEvent.click(apply);
    await waitFor(() => expect(props.onUpdateCondition).toHaveBeenCalledOnce());
    rerender(<EncounterConditionControls {...props} disabled pending />);
    await act(async () => resolveSave(refreshed));
    expect(dialog).toBeVisible();
    expect(edit).toBeDisabled();
    rerender(<EncounterConditionControls {...props} />);
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(edit).toBeEnabled();
    rerender(<EncounterConditionControls {...props} disabled pending />);
    expect(screen.queryByRole("dialog")).toBeNull();
    rerender(<EncounterConditionControls {...props} />);
    expect(props.onUpdateCondition).toHaveBeenCalledOnce();
    fireEvent.click(edit);
    expect(
      await screen.findByRole("dialog", { name: "Edit Frightened" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  },
);
