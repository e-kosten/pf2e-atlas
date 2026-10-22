import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useQuery } from "@tanstack/react-query";
import { Alert, Button } from "antd";
import { getEncounter } from "../../api/atlasApi";
import { navigateToAtlasRoute, type AtlasRoute } from "../../app/routes";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { EditEncounterForm } from "./EncounterModals";
import { useEncounterMutations } from "./useEncounterMutations";

type EncounterEditViewProps = { route: Extract<AtlasRoute, { kind: "encounterEdit" }> };
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
          <p>{encounter.data?.encounter.name ?? route.slug}</p>
        </div>
        <Button
          disabled={mutations.pending}
          onClick={() => navigateToAtlasRoute({ kind: "encounter", slug: route.slug })}
        >
          Done
        </Button>
      </section>
      {encounter.error && !mutations.refreshError && (
        <Alert
          type="error"
          title="Encounter could not load"
          description={encounter.error.message}
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
        <Alert
          type="error"
          title="Saved, but encounter could not refresh"
          description={mutations.refreshError.message}
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
        <div className="detail-empty">Loading encounter...</div>
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
                  navigateToAtlasRoute({ kind: "encounter", slug: request.slug });
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
                if (result.active) navigateToAtlasRoute({ kind: "encounters" });
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
