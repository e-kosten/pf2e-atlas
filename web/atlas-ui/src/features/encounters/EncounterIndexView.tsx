import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Checkbox } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import { deleteEncounter, getEncounters } from "../../api/atlasApi";
import type { EncounterSummaryView } from "../../generated/atlas";
import { encounterPath, navigateToAtlasRoute, type AtlasRoute } from "../../app/routes";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { IndexTable, stopIndexRowAction } from "../../shared/ui/tables/IndexTable";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { CreateEncounterModal } from "./EncounterModals";

type EncounterIndexViewProps = {
  route: Extract<AtlasRoute, { kind: "encounters" }>;
};

export function EncounterIndexView(_props: EncounterIndexViewProps) {
  const queryClient = useQueryClient();
  const [createOpen, setCreateOpen] = useState(false);
  const [showArchived, setShowArchived] = useState(false);
  const encounters = useQuery({ queryKey: ["encounters"], queryFn: getEncounters });
  const visible = (encounters.data?.encounters ?? []).filter(
    (encounter) => showArchived || encounter.status !== "archived",
  );
  const deleteMutation = useMutation({
    mutationFn: deleteEncounter,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["encounters"] }),
  });

  return (
    <EntityIndexPage
      actions={
        <div className="encounter-actions">
          <Checkbox
            checked={showArchived}
            onChange={(event) => setShowArchived(event.target.checked)}
          >
            Archived
          </Checkbox>
          <Button
            icon={<Plus size={16} />}
            onClick={() => setCreateOpen(true)}
            type="primary"
          >
            New Encounter
          </Button>
        </div>
      }
      className="encounter-index-view"
      overlays={
        <CreateEncounterModal
          open={createOpen}
          onCancel={() => setCreateOpen(false)}
          onCreated={(slug) => {
            setCreateOpen(false);
            navigateToAtlasRoute({ kind: "encounter", slug });
          }}
        />
      }
      summary={`${visible.length} active encounters`}
      title="Encounters"
    >
      <IndexTable
        columns={encounterColumns(
          (encounter) =>
            navigateToAtlasRoute({ kind: "encounterEdit", slug: encounter.slug }),
          (encounter) => deleteMutation.mutate(encounter.slug),
        )}
        dataSource={visible}
        loading={encounters.isLoading || encounters.isFetching}
        locale={{ emptyText: "No encounters" }}
        onActivateRow={(encounter) =>
          navigateToAtlasRoute({ kind: "encounter", slug: encounter.slug })
        }
        rowKey={(encounter) => encounter.encounter_key}
      />
    </EntityIndexPage>
  );
}

function encounterColumns(
  onEdit: (encounter: EncounterSummaryView) => void,
  onDelete: (encounter: EncounterSummaryView) => void,
): ColumnsType<EncounterSummaryView> {
  return [
    {
      title: "Name",
      dataIndex: "name",
      render: (_value, encounter) => (
        <a href={encounterPath(encounter.slug)}>{encounter.name}</a>
      ),
    },
    { title: "Status", dataIndex: "status", width: 120 },
    { title: "Round", dataIndex: "round_number", width: 90 },
    { title: "Participants", dataIndex: "participant_count", width: 120 },
    { title: "Updated", dataIndex: "updated_at", width: 220 },
    {
      title: "",
      width: 112,
      render: (_value, encounter) => (
        <span className="encounter-row-actions">
          <Button
            aria-label={`Edit ${encounter.name}`}
            icon={<Edit2 size={14} />}
            size="small"
            onClick={(event) => {
              stopIndexRowAction(event);
              onEdit(encounter);
            }}
          />
          <DangerActionButton
            aria-label={`Delete ${encounter.name}`}
            confirmContent="This permanently deletes the encounter."
            confirmOkText="Delete"
            confirmTitle={`Delete ${encounter.name}?`}
            icon={<Trash2 size={14} />}
            onBeforeConfirm={(event) => {
              stopIndexRowAction(event);
            }}
            onConfirm={() => onDelete(encounter)}
            size="small"
          />
        </span>
      ),
    },
  ];
}
