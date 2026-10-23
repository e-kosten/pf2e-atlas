import { fireEvent, screen } from "@testing-library/react";
import type { ComponentProps, ReactNode } from "react";
import { EncounterInspectorPane } from "./EncounterInspectorPane";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";
import { detailFixture, spellFixture } from "../../test/fixtures";

vi.mock("../../api/atlasApi", () => ({
  getEncounterConditionLifecycleActions: vi
    .fn()
    .mockResolvedValue({ encounter_revision: 1n, actions: [] }),
  getEncounterHpLifecycle: vi.fn().mockResolvedValue([]),
}));
vi.mock("./EncounterParticipantSettings", () => ({
  EncounterParticipantSettings: () => null,
}));
vi.mock("./EncounterConditionControls", () => ({
  EncounterConditionControls: () => null,
}));
vi.mock("../../shared/records/RecordReferenceDrawer", () => ({
  RecordReferenceDrawer: ({ actions }: { actions: ReactNode }) => <>{actions}</>,
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
  const spell = detailFixture("actors:mage");
  spell.presentation.body = { kind: "spell", value: spellFixture() };
  const props = {
    disabled: false,
    previewLoading: false,
    encounterKey: "encounter",
    encounterRevision: 1n,
    participants: [],
    participant: encounterParticipant({
      participant_key: "one",
      participant_kind: "creature",
    }),
    previewRecordKey: "actors:mage",
    previewDetail: spell,
    onSpellAction,
  } as unknown as ComponentProps<typeof EncounterInspectorPane>;
  const { rerender } = renderEncounter(
    <EncounterInspectorPane {...props} previewError={new Error("Refresh failed")} />,
  );
  expect(screen.getByRole("button", { name: "Cast" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  expect(onSpellAction).not.toHaveBeenCalled();
  rerender(<EncounterInspectorPane {...props} previewLoading />);
  expect(screen.getByRole("button", { name: "Cast" })).toBeDisabled();
  rerender(<EncounterInspectorPane {...props} />);
  expect(screen.getByRole("button", { name: "Cast" })).toBeEnabled();
  rerender(
    <EncounterInspectorPane {...props} previewDetail={detailFixture("actors:mage")} />,
  );
  expect(screen.queryByRole("button", { name: "Cast" })).toBeNull();
});
