import { AtlasLink } from "../../app/AtlasLink";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { navigateDestination } from "../../app/navigation";
import { encounterDestination, encountersDestination } from "../../app/destinations";
import { useRouterState } from "@tanstack/react-router";
import { atlasRouter } from "../../app/router";
import { encounterIndexSearch } from "../../app/routeSearch";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Tooltip, Button, Tabs } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import { deleteEncounter, getEncounters } from "../../api/atlasApi";
import type { EncounterSummaryView } from "../../generated/atlas";
import { formatDate } from "../../shared/ui/formatDate";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { IndexTable, stopIndexRowAction } from "../../shared/ui/tables/IndexTable";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { PartyManager } from "../parties/PartyManager";
import { encounterStatusLabels } from "./encounterStatus";
import { CreateEncounterModal } from "./EncounterModals";
import { EncounterSettingsEditor } from "./EncounterSettingsEditor";
import { useEncounterQuery } from "./useEncounterQuery";
import { useEncounterMutations } from "./useEncounterMutations";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
type EncounterIndexViewProps = {
  route: Record<string, never>;
};
export function EncounterIndexView(_props: EncounterIndexViewProps) {
  usePageTitle("Encounters");
  const queryClient = useQueryClient();
  const [partiesOpen, setPartiesOpen] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [editing, setEditing] = useState<{ key: string; open: boolean }>();
  const editOpener = useRef<HTMLElement>(null);
  const newEncounter = useRef<HTMLButtonElement>(null);
  const closeEditor = () =>
    setEditing((current) => current && { ...current, open: false });
  const search = useRouterState({
    router: atlasRouter,
    select: (state) => state.location.search,
  });
  const { view } = encounterIndexSearch(search);
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
        <>
          <Button
            onClick={(event) => {
              event.currentTarget.focus();
              setPartiesOpen(true);
            }}
          >
            Parties
          </Button>
          <Button
            ref={newEncounter}
            icon={<Plus size={16} />}
            onClick={() => setCreateOpen(true)}
            type="primary"
          >
            New Encounter
          </Button>
        </>
      }
      className="encounter-index-view"
      overlays={
        <>
          {partiesOpen && <PartyManager onClose={() => setPartiesOpen(false)} />}
          {editing && (
            <EncounterIndexEditor
              key={editing.key}
              encounterKey={editing.key}
              open={editing.open}
              onClose={closeEditor}
              afterClose={() => {
                setEditing(undefined);
                restoreOverlayFocus(
                  editOpener.current?.isConnected
                    ? editOpener.current
                    : newEncounter.current,
                  undefined,
                );
              }}
            />
          )}
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
          onChange={(view) =>
            navigateDestination(
              encountersDestination({ view: view as "active" | "archived" }),
            )
          }
          items={[
            { key: "active", label: "Active" },
            { key: "archived", label: "Archived" },
          ]}
        />
      }
      title="Encounters"
    >
      {encounters.error && (
        <ErrorNotice
          error={encounters.error}
          title="Encounters could not load"
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
        <ErrorNotice
          error={deleteMutation.error}
          title="Encounter could not be deleted"
        />
      )}
      <IndexTable
        columns={encounterColumns(
          (encounter, opener) => {
            opener.focus({ preventScroll: true });
            editOpener.current = opener;
            setEditing({ key: encounter.encounter_key, open: true });
          },
          (encounter) => deleteMutation.mutateAsync(encounter),
          deleteMutation.isPending,
        )}
        scroll={{ x: 800 }}
        dataSource={visible}
        loading={encounters.isLoading || encounters.isFetching}
        locale={{
          emptyText: encounters.isLoading
            ? null
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
function EncounterIndexEditor({
  encounterKey,
  open,
  onClose,
  afterClose,
}: {
  encounterKey: string;
  open: boolean;
  onClose: () => void;
  afterClose: () => void;
}) {
  const encounter = useEncounterQuery(encounterKey);
  const mutations = useEncounterMutations(encounterKey, encounterKey);
  return (
    <EncounterSettingsEditor
      encounter={encounter}
      mutations={mutations}
      open={open}
      onClose={onClose}
      onDeleted={onClose}
      afterClose={afterClose}
    />
  );
}

function encounterColumns(
  onEdit: (encounter: EncounterSummaryView, opener: HTMLElement) => void,
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
        <AtlasLink {...encounterDestination({ slug: encounter.slug })}>
          {encounter.name}
        </AtlasLink>
      ),
    },
    {
      title: "Status",
      dataIndex: "status",
      width: 120,
      render: (_, encounter) => encounterStatusLabels[encounter.status],
    },
    {
      title: "Round",
      dataIndex: "round_number",
      width: 90,
      render: (round: number) => round.toLocaleString(),
    },
    { title: "Participants", dataIndex: "participant_count", width: 120 },
    { title: "Updated", dataIndex: "updated_at", width: 180, render: formatDate },
    {
      title: "",
      width: 112,
      render: (_value, encounter) => (
        <span className="encounter-row-actions">
          <PaneIconButton
            label={`Edit ${encounter.name}`}
            size="small"
            icon={<Edit2 size={14} />}
            onClick={(event) => {
              stopIndexRowAction(event);
              onEdit(encounter, event.currentTarget);
            }}
          />
          <Tooltip title={`Delete ${encounter.name}`} trigger={["hover", "focus"]}>
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
              type="text"
              size="small"
            />
          </Tooltip>
        </span>
      ),
    },
  ];
}
