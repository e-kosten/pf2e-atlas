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
import { Button, Tabs } from "antd";
import type { ColumnType, ColumnsType } from "antd/es/table";
import { Edit2, Plus } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { deleteEncounter, getEncounters } from "../../api/atlasApi";
import type { EncounterDetailView, EncounterSummaryView } from "../../generated/atlas";
import { formatDate } from "../../shared/ui/formatDate";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { EntityActionsMenu } from "../../shared/ui/actions/EntityActionsMenu";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import { isUnconfirmedMutation } from "../../api/apiError";
import { canFocusTabElement, useTabCloseGuard } from "../../app/DesktopTabContext";
import { IndexTable, stopIndexRowAction } from "../../shared/ui/tables/IndexTable";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";

import {
  archiveNeedsRefresh,
  encounterTurnText,
  setEncounterArchived,
  useArchiveError,
} from "./encounterSummary";
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
  const archiveOpener = useRef<HTMLButtonElement | null>(null);
  const tabs = useRef<HTMLDivElement>(null);
  const visible = (encounters.data?.encounters ?? []).filter(
    (encounter) => (view === "archived") === encounter.archived,
  );
  useLayoutEffect(() => {
    const opener = archiveOpener.current;
    if (!opener || opener.isConnected) return;
    archiveOpener.current = null;
    const tab = tabs.current?.querySelector<HTMLElement>(
      '[role="tab"][aria-selected="true"]',
    );
    if (canFocusTabElement(tab)) restoreOverlayFocus(tab, undefined);
  }, [view, encounters.data]);
  const [archiveRefreshFailure, setArchiveRefreshFailure] = useState<{
    error: unknown;
    outcome: "saved" | "changed" | "uncertain";
  }>();
  const archiveMutation = useMutation({
    onMutate: ({ opener }) => {
      archiveOpener.current = opener;
      setArchiveRefreshFailure(undefined);
    },
    retry: false,
    mutationFn: ({
      encounter,
    }: {
      encounter: EncounterSummaryView;
      opener: HTMLButtonElement;
    }) => setEncounterArchived(encounter.encounter_key, !encounter.archived),
    onSettled: async (_receipt, error, { encounter }) => {
      if (error && !archiveNeedsRefresh(error)) return;
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["encounters"] }),
        queryClient.invalidateQueries({
          queryKey: ["encounter", encounter.encounter_key],
          exact: true,
        }),
      ]);
      const readError = queryClient.getQueryState(["encounters"])?.error;
      if (readError) {
        setArchiveRefreshFailure({
          error: readError,
          outcome: !error
            ? "saved"
            : isUnconfirmedMutation(error)
              ? "uncertain"
              : "changed",
        });
      }
    },
  });
  const archiveOutcome =
    encounters.error && encounters.error === archiveRefreshFailure?.error
      ? archiveRefreshFailure.outcome
      : undefined;
  const archiveTarget = archiveMutation.variables?.encounter;
  const observedMembership = encounters.data?.encounters.find(
    (row) => row.encounter_key === archiveTarget?.encounter_key,
  )?.archived;
  const archiveError = useArchiveError(
    archiveMutation.error,
    archiveTarget && !archiveTarget.archived,
    encounters.error ? undefined : observedMembership,
    archiveMutation.reset,
  );
  useTabCloseGuard(() => (archiveMutation.isPending ? "pending" : undefined));
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
        <div ref={tabs}>
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
        </div>
      }
      title="Encounters"
    >
      {encounters.error && (
        <ErrorNotice
          error={encounters.error}
          title={
            archiveOutcome === "saved"
              ? "Saved, but encounters could not refresh"
              : archiveOutcome === "uncertain"
                ? "Archive change could not be confirmed"
                : archiveOutcome === "changed"
                  ? "Encounter changed and could not refresh"
                  : "Encounters could not load"
          }
          outcome={archiveOutcome}
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
      {archiveError && !archiveOutcome && (
        <ErrorNotice
          error={archiveError}
          title="Archive change could not be confirmed"
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
          (encounter, opener) => archiveMutation.mutate({ encounter, opener }),
          paused ||
            deleteMutation.isPending ||
            archiveMutation.isPending ||
            !!encounters.error,
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
  onArchive: (encounter: EncounterSummaryView, opener: HTMLButtonElement) => void,
  disabled: boolean,
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
      dataIndex: "name",
      render: (_, encounter) => nameLink(encounter),
    },
    {
      title: "Turn",
      key: "turn",
      width: 260,
      render: (_, encounter) => encounterTurnText(encounter),
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
          disabled={disabled}
          loading={openingKey === encounter.encounter_key ? { delay: 200 } : false}
          size="small"
          icon={<Edit2 size={14} />}
          onClick={(event) => {
            stopIndexRowAction(event);
            onEdit(encounter, event.currentTarget);
          }}
        />
        <EncounterRowMenu
          encounter={encounter}
          disabled={disabled}
          onArchive={onArchive}
          onDelete={onDelete}
        />
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
                {encounterTurnText(encounter) && (
                  <span>{encounterTurnText(encounter)}</span>
                )}
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

function EncounterRowMenu({
  encounter,
  disabled,
  onArchive,
  onDelete,
}: {
  encounter: EncounterSummaryView;
  disabled: boolean;
  onArchive: (encounter: EncounterSummaryView, opener: HTMLButtonElement) => void;
  onDelete: (encounter: EncounterSummaryView) => Promise<unknown>;
}) {
  const trigger = useRef<HTMLButtonElement>(null);
  const { confirm, dialog } = useConfirmDangerAction();
  return (
    <>
      <EntityActionsMenu
        label={`More actions for ${encounter.name}`}
        size="small"
        trigger={trigger}
        disabled={disabled}
        items={[
          {
            key: "archive",
            label: encounter.archived ? "Restore encounter" : "Archive encounter",
          },
          { key: "delete", label: "Delete encounter", danger: true },
        ]}
        onAction={(key, opener) => {
          if (key === "archive") onArchive(encounter, opener);
          else
            confirm({
              title: `Delete ${encounter.name}?`,
              content: "This permanently deletes the encounter.",
              okText: "Delete",
              onConfirm: () => onDelete(encounter),
            });
        }}
      />
      {dialog}
    </>
  );
}
