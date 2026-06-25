import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "antd";
import { deleteEncounter, getEncounter, updateEncounter } from "../../api/atlasApi";
import { navigateToAtlasRoute, type AtlasRoute } from "../../app/routes";
import { confirmDangerAction } from "../../shared/ui/actions/confirmDangerAction";
import { EditEncounterForm } from "./EncounterModals";

type EncounterEditViewProps = {
  route: Extract<AtlasRoute, { kind: "encounterEdit" }>;
};

export function EncounterEditView({ route }: EncounterEditViewProps) {
  const queryClient = useQueryClient();
  const encounter = useQuery({
    queryKey: ["encounter", route.slug],
    queryFn: () => getEncounter(route.slug),
  });
  const updateEncounterMutation = useMutation({
    mutationFn: updateEncounter,
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["encounter", route.slug] });
      await queryClient.invalidateQueries({ queryKey: ["encounters"] });
      navigateToAtlasRoute({ kind: "encounter", slug: view.encounter.slug });
    },
  });
  const deleteMutation = useMutation({
    mutationFn: deleteEncounter,
    onSuccess: () => navigateToAtlasRoute({ kind: "encounters" }),
  });

  return (
    <main className="encounter-edit-view">
      <section className="encounter-edit-view__header">
        <div>
          <h2>Edit Encounter</h2>
          <p>{encounter.data?.encounter.name ?? route.slug}</p>
        </div>
        <Button
          onClick={() => navigateToAtlasRoute({ kind: "encounter", slug: route.slug })}
        >
          Done
        </Button>
      </section>
      {encounter.isLoading || !encounter.data ? (
        <div className="detail-empty">Loading encounter...</div>
      ) : (
        <section className="encounter-edit-view__content">
          <EditEncounterForm
            encounter={encounter.data.encounter}
            encounterNote={encounter.data.note}
            onSave={(request) => updateEncounterMutation.mutate(request)}
          />
          <div className="encounter-edit-view__metadata">
            <span>Created {encounter.data.encounter.created_at}</span>
            <span>Updated {encounter.data.encounter.updated_at}</span>
          </div>
          <Button
            danger
            onClick={() =>
              confirmDangerAction({
                title: `Delete ${encounter.data.encounter.name}?`,
                content: "This permanently deletes the encounter.",
                okText: "Delete",
                onConfirm: () => {
                  deleteMutation.mutate(route.slug);
                },
              })
            }
          >
            Delete encounter
          </Button>
        </section>
      )}
    </main>
  );
}
