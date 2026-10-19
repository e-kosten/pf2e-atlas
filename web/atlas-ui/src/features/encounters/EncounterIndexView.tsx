import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Checkbox } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import { deleteEncounter, getEncounters } from "../../api/atlasApi";
import type { EncounterSummaryView } from "../../generated/atlas";
import {
  encounterPath,
  atlasRoutePath,
  shouldHandleAtlasRouteClick,
  navigateToAtlasRoute,
  type AtlasRoute,
} from "../../app/routes";
import { formatDate } from "../../shared/ui/formatDate";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
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
    retry: false,
    mutationFn: (encounter: EncounterSummaryView) =>
      deleteEncounter(encounter.encounter_key),
    onSuccess: async (receipt, encounter) => {
      for (const ref of [receipt.encounter_key, receipt.slug, encounter.slug]) {
        queryClient.removeQueries({ queryKey: ["encounter", ref], exact: true });
      }
      await queryClient.invalidateQueries({ queryKey: ["encounters"] });
    },
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
      summary={
        encounters.data
          ? `${visible.length} ${showArchived ? "encounters" : "active encounters"}`
          : undefined
      }
      title="Encounters"
    >
      {encounters.error && (
        <Alert
          type="error"
          title="Encounters could not load"
          description={encounters.error.message}
          action={
            <Button
              onClick={() => void encounters.refetch()}
              loading={encounters.isFetching}
            >
              Retry encounters
            </Button>
          }
        />
      )}
      {deleteMutation.error && (
        <Alert
          type="error"
          title="Encounter could not be deleted"
          description={deleteMutation.error.message}
        />
      )}
      <IndexTable
        columns={encounterColumns(
          (encounter) =>
            navigateToAtlasRoute({ kind: "encounterEdit", slug: encounter.slug }),
          (encounter) => deleteMutation.mutateAsync(encounter),
          deleteMutation.isPending,
        )}
        scroll={{ x: 800 }}
        dataSource={visible}
        loading={encounters.isLoading || encounters.isFetching}
        locale={{
          emptyText: encounters.isLoading
            ? "Loading encounters…"
            : encounters.error && !encounters.data
              ? "Encounters unavailable"
              : !showArchived && encounters.data?.encounters.length
                ? "No active encounters. Show archived encounters to see more."
                : "No encounters. Create an encounter to get started.",
        }}
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
  onDelete: (encounter: EncounterSummaryView) => Promise<unknown>,
  deleting: boolean,
): ColumnsType<EncounterSummaryView> {
  return [
    {
      title: "Name",
      minWidth: 240,
      width: 280,
      dataIndex: "name",
      render: (_value, encounter) => (
        <a href={encounterPath(encounter.slug)}>{encounter.name}</a>
      ),
    },
    { title: "Status", dataIndex: "status", width: 120 },
    {
      title: "Round",
      dataIndex: "round_number",
      width: 90,
      render: (round: bigint) => round.toLocaleString(),
    },
    { title: "Participants", dataIndex: "participant_count", width: 120 },
    { title: "Updated", dataIndex: "updated_at", width: 180, render: formatDate },
    {
      title: "",
      width: 112,
      render: (_value, encounter) => (
        <span className="encounter-row-actions">
          <PaneIconLink
            href={atlasRoutePath({ kind: "encounterEdit", slug: encounter.slug })}
            label={`Edit ${encounter.name}`}
            icon={<Edit2 size={14} />}
            onClick={(event) => {
              stopIndexRowAction(event);
              if (!shouldHandleAtlasRouteClick(event)) return;
              event.preventDefault();
              onEdit(encounter);
            }}
          />
          <DangerActionButton
            disabled={deleting}
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
