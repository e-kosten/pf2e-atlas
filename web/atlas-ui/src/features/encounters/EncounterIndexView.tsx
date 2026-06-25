import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Checkbox, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import { deleteEncounter, getEncounters } from "../../api/atlasApi";
import type { EncounterSummaryView } from "../../generated/atlas";
import { encounterPath, navigateToAtlasRoute, type AtlasRoute } from "../../app/routes";
import { confirmDangerAction } from "../../shared/confirm/confirmAction";
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
    <main className="encounter-index-view">
      <section className="list-index-view__toolbar">
        <div>
          <h2>Encounters</h2>
          <p>{visible.length} active encounters</p>
        </div>
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
      </section>
      <section className="list-index-view__table">
        <Table
          columns={encounterColumns(
            (encounter) =>
              navigateToAtlasRoute({ kind: "encounterEdit", slug: encounter.slug }),
            (encounter) =>
              confirmDangerAction({
                title: `Delete ${encounter.name}?`,
                content: "This permanently deletes the encounter.",
                okText: "Delete",
                onConfirm: () => {
                  deleteMutation.mutate(encounter.slug);
                },
              }),
          )}
          dataSource={visible}
          loading={encounters.isLoading || encounters.isFetching}
          locale={{ emptyText: "No encounters" }}
          pagination={false}
          onRow={(encounter) => ({
            className: "index-row",
            tabIndex: 0,
            onClick: () =>
              navigateToAtlasRoute({ kind: "encounter", slug: encounter.slug }),
            onKeyDown: (event) => {
              if (event.key === "Enter") {
                navigateToAtlasRoute({ kind: "encounter", slug: encounter.slug });
              }
            },
          })}
          rowKey={(encounter) => encounter.encounter_key}
          size="middle"
        />
      </section>
      <CreateEncounterModal
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onCreated={(slug) => {
          setCreateOpen(false);
          navigateToAtlasRoute({ kind: "encounter", slug });
        }}
      />
    </main>
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
              event.stopPropagation();
              onEdit(encounter);
            }}
          />
          <Button
            danger
            aria-label={`Delete ${encounter.name}`}
            icon={<Trash2 size={14} />}
            size="small"
            onClick={(event) => {
              event.stopPropagation();
              onDelete(encounter);
            }}
          />
        </span>
      ),
    },
  ];
}
