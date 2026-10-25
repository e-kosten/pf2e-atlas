import {
  navigateDestination,
  destinationHref,
  isPrimaryActivation,
} from "../../app/navigation";
import { encounterDestination, encounterEditDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Tabs } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import { deleteEncounter, getEncounters } from "../../api/atlasApi";
import type { EncounterSummaryView } from "../../generated/atlas";
import { formatDate } from "../../shared/ui/formatDate";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { IndexTable, stopIndexRowAction } from "../../shared/ui/tables/IndexTable";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { PartyManager } from "../parties/PartyManager";
import { CreateEncounterModal } from "./EncounterModals";
type EncounterIndexViewProps = {
  route: Record<string, never>;
};
export function EncounterIndexView(_props: EncounterIndexViewProps) {
  usePageTitle("Encounters");
  const queryClient = useQueryClient();
  const [partiesOpen, setPartiesOpen] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [view, setView] = useState("active");
  const encounters = useQuery({ queryKey: ["encounters"], queryFn: getEncounters });
  const visible = (encounters.data?.encounters ?? []).filter(
    (encounter) => (view === "archived") === (encounter.status === "archived"),
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
          <Button
            onClick={(event) => {
              event.currentTarget.focus();
              setPartiesOpen(true);
            }}
          >
            Parties
          </Button>
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
        <>
          {partiesOpen && <PartyManager onClose={() => setPartiesOpen(false)} />}
          <CreateEncounterModal
            open={createOpen}
            onCancel={() => setCreateOpen(false)}
            onCreated={(slug) => {
              setCreateOpen(false);
              navigateDestination(
                encounterDestination({
                  slug,
                }),
              );
            }}
          />
        </>
      }
      summary={
        encounters.data
          ? `${visible.length} ${view === "archived" ? "archived encounters" : "active encounters"}`
          : undefined
      }
      toolbar={
        <Tabs
          aria-label="Encounter views"
          activeKey={view}
          onChange={setView}
          items={[
            { key: "active", label: "Active" },
            { key: "archived", label: "Archived" },
          ]}
        />
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
            navigateDestination(
              encounterEditDestination({
                slug: encounter.slug,
              }),
            ),
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
              : view === "archived"
                ? "No archived encounters."
                : "No active encounters. Create an encounter to get started.",
        }}
        onActivateRow={(encounter) =>
          navigateDestination(
            encounterDestination({
              slug: encounter.slug,
            }),
          )
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
        <a href={destinationHref(encounterDestination({ slug: encounter.slug }))}>
          {encounter.name}
        </a>
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
            href={destinationHref(
              encounterEditDestination({
                slug: encounter.slug,
              }),
            )}
            label={`Edit ${encounter.name}`}
            icon={<Edit2 size={14} />}
            onClick={(event) => {
              stopIndexRowAction(event);
              if (!isPrimaryActivation(event)) return;
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
