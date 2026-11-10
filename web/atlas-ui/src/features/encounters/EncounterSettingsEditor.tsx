import { Button } from "antd";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { EditEncounterModal } from "./EncounterModals";
import type { useEncounterMutations } from "./useEncounterMutations";
import type { useEncounterQuery } from "./useEncounterQuery";

export function EncounterSettingsEditor({
  encounter,
  mutations,
  open,
  disabled = false,
  onClose,
  onDeleted,
  afterClose,
}: {
  encounter: ReturnType<typeof useEncounterQuery>;
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
  ) : encounter.error && !mutations.metadataRefreshFailed ? (
    <ErrorNotice
      error={encounter.error}
      title="Encounter could not load"
      action={
        <Button loading={encounter.isFetching} onClick={() => void encounter.refetch()}>
          Retry encounter
        </Button>
      }
    />
  ) : !encounter.data && encounter.isLoading ? (
    <LoadingStatus label="Loading encounter…" />
  ) : null;

  return (
    <EditEncounterModal
      encounter={encounter.data?.encounter}
      encounterNote={encounter.data?.note}
      disabled={
        disabled || mutations.pending || !!mutations.refreshError || !!encounter.error
      }
      pending={mutations.pending}
      error={mutations.errorKind === "metadata" ? mutations.errorMessage : undefined}
      feedback={feedback}
      open={open}
      onCancel={() => onClose()}
      afterClose={afterClose}
      onRefreshConflict={mutations.refreshMetadataConflict}
      onDelete={
        encounter.data
          ? async () => {
              const result = await mutations.run({ kind: "delete" });
              if (result.active) onDeleted();
            }
          : undefined
      }
      onSave={async (request) => {
        const result = await mutations.run({ kind: "metadata", request });
        if (result.active && result.refreshed && result.view)
          onClose(result.view.encounter.slug);
      }}
    />
  );
}
