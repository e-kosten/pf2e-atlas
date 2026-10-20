import { fireEvent, render, screen } from "@testing-library/react";
import type { ComponentProps, ReactNode } from "react";
import { EncounterInspectorPane } from "./EncounterInspectorPane";
import { detailFixture } from "../../test/fixtures";

vi.mock("./EncounterParticipantSettings", () => ({
  EncounterParticipantSettings: () => null,
}));
vi.mock("./EncounterConditionControls", () => ({
  EncounterConditionControls: () => null,
}));
vi.mock("../../shared/records/RecordPreviewPopover", () => ({
  RecordPreviewPopover: ({ actions }: { actions: ReactNode }) => <>{actions}</>,
}));
vi.mock("./EncounterSpellControls", () => ({
  encounterCastingAnnotations: () => undefined,
  EncounterSpellControls: ({
    disabled,
    onAction,
  }: {
    disabled: boolean;
    onAction: () => void;
  }) => (
    <button disabled={disabled} onClick={onAction}>
      Cast
    </button>
  ),
}));

it("keeps retained spell content read-only after a contextual refresh fails and enables it only after a successful retry", () => {
  const onSpellAction = vi.fn();
  const props = {
    disabled: false,
    previewLoading: false,
    participant: {
      participant_key: "one",
      participant_kind: "creature",
      participant_variant: "normal",
    },
    previewRecordKey: "actors:mage",
    previewDetail: detailFixture("actors:mage"),
    onSpellAction,
  } as unknown as ComponentProps<typeof EncounterInspectorPane>;
  const { rerender } = render(
    <EncounterInspectorPane {...props} previewError={new Error("Refresh failed")} />,
  );
  expect(screen.getByRole("button", { name: "Cast" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  expect(onSpellAction).not.toHaveBeenCalled();
  rerender(<EncounterInspectorPane {...props} previewLoading />);
  expect(screen.getByRole("button", { name: "Cast" })).toBeDisabled();
  rerender(<EncounterInspectorPane {...props} />);
  expect(screen.getByRole("button", { name: "Cast" })).toBeEnabled();
});
