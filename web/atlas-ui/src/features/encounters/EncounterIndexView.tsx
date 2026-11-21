import { useAtlasRouter } from "../../app/router";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasLink } from "../../app/AtlasLink";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { navigateDestination } from "../../app/navigation";
import { encounterDestination, encountersDestination } from "../../app/destinations";
import { useRouterState } from "@tanstack/react-router";
import { encounterIndexSearch } from "../../app/routeSearch";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Tooltip, Button, Tabs } from "antd";
import type { ColumnType, ColumnsType } from "antd/es/table";
import { Edit2, Plus, Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import { deleteEncounter, getEncounters } from "../../api/atlasApi";
import type { EncounterDetailView, EncounterSummaryView } from "../../generated/atlas";
import { formatDate } from "../../shared/ui/formatDate";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { IndexTable, stopIndexRowAction } from "../../shared/ui/tables/IndexTable";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";

import { encounterStatusLabels } from "./encounterStatus";
import { CreateEncounterModal } from "./EncounterModals";
import { EncounterSettingsEditor } from "./EncounterSettingsEditor";
import { useCanonicalEncounter } from "./useEncounterQuery";
import { useEncounterSettingsOpening } from "./useEncounterSettingsOpening";
import { SettingsOpeningError } from "../../shared/ui/overlays/SettingsOpeningError";
import { useEncounterMutations } from "./useEncounterMutations";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
type EncounterIndexViewProps = {
  route: Record<string, never>;
};
export function EncounterIndexView(_props: EncounterIndexViewProps) {
  const atlasRouter = useAtlasRouter();
  usePageTitle("Encounters");
  const paused = useSavedWritesPaused();
  const queryClient = useQueryClient();
  const [createOpen, setCreateOpen] = useState(false);
  const editing = useEncounterSettingsOpening();
  const newEncounter = useRef<HTMLButtonElement>(null);
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
        <Button
          disabled={paused}
          ref={newEncounter}
          icon={<Plus size={16} />}
          onClick={() => setCreateOpen(true)}
          type="primary"
        >
          New encounter
        </Button>
      }
      className="encounter-index-view"
      overlays={
        <>
          {editing.data && (
            <EncounterIndexEditor
              key={editing.data.encounter.encounter_key}
              initialDetail={editing.data}
              open={editing.open}
              onClose={editing.close}
              afterClose={() => {
                editing.clear();
                restoreOverlayFocus(
                  editing.opener?.isConnected ? editing.opener : newEncounter.current,
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
                undefined,
                atlasRouter,
              );
            }}
          />
        </>
      }
      summary={
        encounters.data
          ? `${visible.length.toLocaleString()} ${view} ${visible.length === 1 ? "encounter" : "encounters"}`
          : undefined
      }
      toolbar={
        <Tabs
          aria-label="Encounter views"
          activeKey={view}
          onChange={(view) =>
            navigateDestination(
              encountersDestination({ view: view as "active" | "archived" }),
              undefined,
              atlasRouter,
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
      <SettingsOpeningError
        error={editing.error}
        title="Encounter settings could not load"
        onRetry={() => {
          editing.opener?.focus({ preventScroll: true });
          editing.retry();
        }}
      />
      <IndexTable
        {...encounterColumns(
          (encounter, opener) => {
            opener.focus({ preventScroll: true });

            editing.begin(encounter.encounter_key, opener);
          },
          (encounter) => {
            if (editing.ref === encounter.encounter_key) editing.abandon();
            return deleteMutation.mutateAsync(encounter);
          },
          paused || deleteMutation.isPending,
          editing.preparing ? editing.ref : undefined,
        )}
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
            undefined,
            atlasRouter,
          )
        }
        rowKey={(encounter) => encounter.encounter_key}
      />
    </EntityIndexPage>
  );
}
function EncounterIndexEditor({
  initialDetail,
  open,
  onClose,
  afterClose,
}: {
  initialDetail: EncounterDetailView;
  open: boolean;
  onClose: () => void;
  afterClose: () => void;
}) {
  const encounterKey = initialDetail.encounter.encounter_key;
  const encounter = useCanonicalEncounter(encounterKey);
  const mutations = useEncounterMutations(encounterKey, encounterKey);
  return (
    <EncounterSettingsEditor
      encounter={encounter.data ?? initialDetail}
      readError={encounter.error}
      retryRead={() => void encounter.refetch()}
      refreshing={encounter.isFetching}
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
  openingKey?: string,
) {
  const nameLink = (encounter: EncounterSummaryView) => (
    <AtlasLink {...encounterDestination({ slug: encounter.slug })}>
      {encounter.name}
    </AtlasLink>
  );
  const columns: ColumnsType<EncounterSummaryView> = [
    {
      title: "Name",
      minWidth: 240,
      width: 280,
      dataIndex: "name",
      render: (_, encounter) => nameLink(encounter),
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
  ];
  const actions: ColumnType<EncounterSummaryView> = {
    title: <span className="sr-only">Actions</span>,
    key: "actions",
    width: 112,
    render: (_value, encounter) => (
      <span className="encounter-row-actions">
        <PaneIconButton
          label={`Edit ${encounter.name}`}
          aria-busy={openingKey === encounter.encounter_key}
          disabled={deleting}
          loading={openingKey === encounter.encounter_key ? { delay: 200 } : false}
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
  };
  return {
    columns: [...columns, actions],
    compact: {
      below: 900,
      columns: [
        {
          title: "Name",
          dataIndex: "name",
          render: (_, encounter) => (
            <div className="index-row-summary">
              {nameLink(encounter)}
              <div className="index-row-summary__facts">
                <span>{encounterStatusLabels[encounter.status]}</span>
                <span>Round {encounter.round_number.toLocaleString()}</span>
                <span>
                  {encounter.participant_count.toLocaleString()}{" "}
                  {encounter.participant_count === 1 ? "participant" : "participants"}
                </span>
              </div>
            </div>
          ),
        },
        { ...actions, width: 96 },
      ] satisfies ColumnsType<EncounterSummaryView>,
    },
  };
}
