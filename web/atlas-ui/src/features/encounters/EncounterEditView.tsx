import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { navigateDestination } from "../../app/navigation";
import { encounterDestination, encountersDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useQuery } from "@tanstack/react-query";
import { Button } from "antd";
import { getEncounter } from "../../api/atlasApi";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { EditEncounterForm } from "./EncounterModals";
import { useEncounterMutations } from "./useEncounterMutations";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
type EncounterEditViewProps = {
  route: { slug: string };
};
export function EncounterEditView({ route }: EncounterEditViewProps) {
  return <EncounterEditor key={route.slug} route={route} />;
}
function EncounterEditor({ route }: EncounterEditViewProps) {
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
  });
  usePageTitle(encounter.data?.encounter.name ?? "Encounter");
  const mutations = useEncounterMutations(
    route.slug,
    encounter.data?.encounter.encounter_key,
  );
  const disabled = mutations.pending || !!mutations.refreshError || !!encounter.error;
  return (
    <main className="encounter-edit-view">
      <section className="encounter-edit-view__header">
        <div>
          <h2>Edit Encounter</h2>
          {encounter.data && <p>{encounter.data.encounter.name}</p>}
        </div>
        <Button
          disabled={mutations.pending}
          onClick={() =>
            navigateDestination(
              encounterDestination({
                slug: route.slug,
              }),
            )
          }
        >
          Done
        </Button>
      </section>
      {encounter.error && !mutations.refreshError && (
        <ErrorNotice
          error={encounter.error}
          title="Encounter could not load"
          action={
            <Button
              onClick={() => void encounter.refetch()}
              loading={encounter.isFetching}
            >
              Retry encounter
            </Button>
          }
        />
      )}
      {mutations.refreshError && (
        <ErrorNotice
          error={mutations.refreshError}
          outcome={mutations.refreshOutcome}
          type="warning"
          title={mutations.refreshMessage}
          action={
            <Button
              onClick={() => void mutations.retryRefresh()}
              loading={mutations.pending}
            >
              Retry encounter refresh
            </Button>
          }
        />
      )}
      {encounter.isLoading ? (
        <LoadingStatus className="detail-empty" label="Loading encounter…" />
      ) : (
        encounter.data && (
          <section className="encounter-edit-view__content">
            <EditEncounterForm
              encounter={encounter.data.encounter}
              encounterNote={encounter.data.note}
              disabled={disabled}
              pending={mutations.pending}
              error={mutations.errorMessage}
              onSave={async (request) => {
                const result = await mutations.run({ kind: "metadata", request });
                if (result.active && result.refreshed)
                  navigateDestination(
                    encounterDestination({
                      slug: request.slug,
                    }),
                  );
              }}
            />
            <div className="encounter-edit-view__metadata">
              <span>Created {encounter.data.encounter.created_at}</span>
              <span>Updated {encounter.data.encounter.updated_at}</span>
            </div>
            <DangerActionButton
              disabled={disabled}
              confirmContent="This permanently deletes the encounter."
              confirmOkText="Delete"
              confirmTitle={`Delete ${encounter.data.encounter.name}?`}
              onConfirm={async () => {
                const result = await mutations.run({ kind: "delete" });
                if (result.active) navigateDestination(encountersDestination());
              }}
            >
              Delete encounter
            </DangerActionButton>
          </section>
        )
      )}
    </main>
  );
}
