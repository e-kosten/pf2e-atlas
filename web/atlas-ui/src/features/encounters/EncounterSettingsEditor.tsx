import { Button } from "antd";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { EditEncounterModal } from "./EncounterModals";
import type { useEncounterMutations } from "./useEncounterMutations";
import type { EncounterDetailView } from "../../generated/atlas";

export function EncounterSettingsEditor({
  encounter,
  readError,
  retryRead,
  refreshing,
  mutations,
  open,
  disabled = false,
  onClose,
  onDeleted,
  afterClose,
}: {
  encounter: EncounterDetailView;
  readError: Error | null;
  retryRead: () => void;
  refreshing: boolean;
  mutations: ReturnType<typeof useEncounterMutations>;
  open: boolean;
  disabled?: boolean;
  onClose: (slug?: string) => void;
  onDeleted: () => void;
  afterClose?: () => void;
}) {
  const refreshFailed =
    mutations.errorKind === "metadata" && !mutations.metadataRefreshFailed
      ? mutations.refreshError
      : null;
  const feedback = refreshFailed ? (
    <ErrorNotice
      error={refreshFailed}
      outcome={mutations.refreshOutcome}
      type="warning"
      title={mutations.refreshMessage}
      action={
        <Button
          loading={mutations.pending}
          onClick={() =>
            void mutations.retryRefresh().then((view) => {
              if (view && mutations.refreshOutcome === "saved")
                onClose(view.encounter.slug);
            })
          }
        >
          Retry encounter refresh
        </Button>
      }
    />
  ) : readError && !mutations.metadataRefreshFailed ? (
    <ErrorNotice
      error={readError}
      title="Encounter could not load"
      action={
        <Button loading={refreshing} onClick={retryRead}>
          Retry encounter
        </Button>
      }
    />
  ) : null;

  return (
    <EditEncounterModal
      encounter={encounter.encounter}
      encounterNote={encounter.note}
      disabled={
        disabled || mutations.pending || !!mutations.refreshError || !!readError
      }
      pending={mutations.pending}
      error={mutations.errorKind === "metadata" ? mutations.errorMessage : undefined}
      feedback={feedback}
      open={open}
      onCancel={() => onClose()}
      afterClose={afterClose}
      onRefreshConflict={mutations.refreshMetadataConflict}
      onDelete={async () => {
        const result = await mutations.run({ kind: "delete" });
        if (result.active) onDeleted();
      }}
      onSave={async (request) => {
        const result = await mutations.run({ kind: "metadata", request });
        if (result.active && result.refreshed && result.view)
          onClose(result.view.encounter.slug);
      }}
    />
  );
}
