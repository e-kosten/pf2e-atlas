import {
  Button,
  ConfigProvider,
  Dropdown,
  Form,
  Input,
  List,
  Select,
  Space,
  Typography,
} from "antd";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useLayoutEffect, useRef, useState } from "react";
import {
  getEncounterPcReference,
  getPcReference,
  savePcAbilityLink,
  searchPage,
  setEncounterPcReference,
} from "../../api/atlasApi";
import { searchDestination } from "../../app/destinations";
import { destinationHref } from "../../app/navigation";
import type {
  EncounterPcAssociation,
  PcAbilitySelection,
  PcAbilityView,
  PcReferenceView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { DefinitionDetailPane } from "../../shared/records/DefinitionDetailPane";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { ReferenceHelpProvider } from "../../shared/records/ReferenceHelp";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { usePreviewReadingPosition } from "../../shared/records/usePreviewReadingPosition";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import { searchExecution } from "../../shared/search/searchExecution";
import { AtlasDrawer } from "../../shared/ui/overlays/AtlasDrawer";
import { useParties } from "./useParties";

type PartyLocation = { kind: "party"; partyKey: string; memberKey: string };
type EncounterLocation = {
  kind: "encounter";
  encounterKey: string;
  participantKey: string;
  revision: number;
};
type Location = PartyLocation | EncounterLocation;

const allPcReferences = ["pc-reference"] as const;
const getPopupContainer = (trigger?: HTMLElement) =>
  trigger?.closest<HTMLElement>(".pc-character-drawer-root") ?? document.body;
const abilityLabel = (ability: PcAbilityView) =>
  ability.choice ? `${ability.label} (${ability.choice})` : ability.label;

export function PcCharacterDrawer({
  location,
  name,
  opener,
  onClose,
  onRefresh,
}: {
  location: Location;
  name: string;
  opener: HTMLElement | null;
  onClose: () => void;
  onRefresh?: () => void;
}) {
  const client = useQueryClient();
  const titleId = useId();
  const paused = useSavedWritesPaused();
  const [closing, setClosing] = useState(false);
  const [filter, setFilter] = useState("");
  const [search, setSearch] = useState<{
    ability: PcAbilityView;
    query: string;
    draft: string;
  } | null>(null);
  const [editingAssociation, setEditingAssociation] = useState(false);
  const [memberKey, setMemberKey] = useState<string>();
  const searchInput = useRef<React.ComponentRef<typeof Input>>(null);
  const ruleHeading = useRef<HTMLDivElement>(null);
  const preview = useRecordPreview();
  const selectedRecord = preview.detail?.selected.record_key;
  const previewOpen = preview.destination !== null;
  const { ref: ruleContent, onLandingReady } = usePreviewReadingPosition(
    preview.definition ?? preview.detail,
    preview.readingHistory,
    previewOpen,
    preview.landing,
  );
  const profile = useSearchProfile();
  const execution = searchExecution(profile, "text_search");
  const query = search?.query ?? "";
  const result = useQuery({
    queryKey: [
      "pc-ability-search",
      query,
      execution.profile,
      execution.mode,
      profile.preferences.ranking,
      profile.preferences.searchContent,
    ],
    queryFn: ({ signal }) =>
      searchPage(
        {
          criteria: {
            kind: "text_search",
            mode: execution.mode,
            query,
            search_content: profile.preferences.searchContent,
          },
          page: { number: 1, size: 10 },
          search_profile: execution.profile,
          ranking: profile.preferences.ranking,
        },
        signal,
      ),
    enabled: Boolean(search && query && execution.ready),
    retry: false,
    meta: referenceQueryMeta,
  });
  const reference = useQuery({
    queryKey: resultReferenceKey(location),
    queryFn: async ({ signal }) =>
      location.kind === "party"
        ? getPcReference(location.partyKey, location.memberKey, signal)
        : ((
            await getEncounterPcReference(
              location.encounterKey,
              location.participantKey,
              signal,
            )
          ).reference ?? null),
    retry: false,
    meta: referenceQueryMeta,
  });
  const parties = useParties(location.kind === "encounter" && editingAssociation);
  const save = useMutation({
    retry: false,
    mutationFn: ({
      target,
      ability,
      selection,
    }: {
      target: PcReferenceView;
      ability: PcAbilityView;
      selection: PcAbilitySelection;
    }) =>
      savePcAbilityLink(target.party_key, target.member_key, ability.ability_key, {
        expected_party_revision: target.party_revision,
        selection,
      }),
    onSuccess: async (saved) => {
      client.setQueryData(resultReferenceKey(location), saved);
      await Promise.all([
        client.invalidateQueries({ queryKey: allPcReferences }),
        client.invalidateQueries({ queryKey: ["parties"] }),
      ]);
      if (search) {
        preview.closeForNavigation();
        setSearch(null);
      }
    },
    onError: () => {
      void reference.refetch();
    },
  });
  const attach = useMutation({
    retry: false,
    mutationFn: ({
      encounter,
      association,
    }: {
      encounter: EncounterLocation;
      association: EncounterPcAssociation;
    }) =>
      setEncounterPcReference(encounter.encounterKey, encounter.participantKey, {
        expected_encounter_revision: encounter.revision,
        association,
      }),
    onSuccess: async (_receipt, { encounter }) => {
      await Promise.all([
        client.invalidateQueries({ queryKey: allPcReferences }),
        client.invalidateQueries({ queryKey: ["encounter", encounter.encounterKey] }),
      ]);
      setEditingAssociation(false);
    },
    onError: (_error, { encounter }) => {
      void client.invalidateQueries({
        queryKey: ["encounter", encounter.encounterKey],
      });
    },
  });
  const doc = reference.data;
  const draftQuery = search?.draft;
  const searching = Boolean(search);
  useLayoutEffect(() => {
    if (previewOpen) ruleHeading.current?.focus({ preventScroll: true });
    else if (searching) searchInput.current?.focus({ preventScroll: true });
  }, [previewOpen, searching]);
  const searchAnchor = useRef<HTMLElement>(null);
  const resultAnchor = useRef<HTMLElement>(null);
  const restoreOnBack = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => {
    if (!previewOpen && restoreOnBack.current) {
      restoreOnBack.current.focus({ preventScroll: true });
      restoreOnBack.current = null;
    }
  }, [previewOpen, searching]);
  function startSearch(ability: PcAbilityView, anchor?: HTMLElement) {
    searchAnchor.current = anchor ?? (document.activeElement as HTMLElement | null);
    save.reset();
    setSearch({ ability, query: ability.label, draft: ability.label });
  }
  function back() {
    if (preview.readingHistory.canGoBack) {
      preview.readingHistory.onBack();
    } else if (previewOpen) {
      restoreOnBack.current = resultAnchor.current;
      preview.closeForNavigation();
    } else if (search) {
      restoreOnBack.current = searchAnchor.current;
      setSearch(null);
      save.reset();
    }
  }
  const visible = doc?.abilities.filter((ability) => {
    const term = filter.trim().toLocaleLowerCase();
    return [abilityLabel(ability), ability.record_title].some((label) =>
      label?.toLocaleLowerCase().includes(term),
    );
  });
  const associationOptions = parties.data?.parties.map((party) => ({
    label: party.name,
    options: party.members.map((member) => ({
      value: member.member_key,
      label: member.name,
    })),
  }));
  const busy = save.isPending || attach.isPending;
  return (
    <AtlasDrawer
      open={!closing}
      aria-labelledby={titleId}
      onClose={() => setClosing(true)}
      closable={!busy}
      keyboard={!busy}
      mask={{ closable: !busy }}
      afterOpenChange={(open) => {
        if (!open) {
          onClose();
          opener?.focus({ preventScroll: true });
        }
      }}
      size="min(760px, 100vw)"
      title={
        <Space wrap>
          {(search || previewOpen) && (
            <Button type="text" size="small" onClick={back}>
              Back
            </Button>
          )}
          <span id={titleId}>{doc?.name ?? name}</span>
        </Space>
      }
      focusable={{ focusTriggerAfterClose: false }}
      classNames={{ root: "pc-character-drawer-root" }}
      styles={{
        root: { overflow: "hidden", pointerEvents: "auto" },
        body: { display: "flex", flexDirection: "column", gap: 12 },
      }}
      drawerRender={(content) => (
        <ConfigProvider getPopupContainer={getPopupContainer}>{content}</ConfigProvider>
      )}
    >
      {reference.error && (
        <ErrorNotice
          error={reference.error}
          title="Could not load PC reference"
          action={<Button onClick={() => void reference.refetch()}>Retry</Button>}
        />
      )}
      {save.error && (
        <ErrorNotice error={save.error} title="Could not save rule link" />
      )}
      {attach.error && (
        <ErrorNotice error={attach.error} title="Could not update party member link" />
      )}
      <section
        hidden={Boolean(search || previewOpen)}
        style={{ minHeight: 0, overflow: "auto" }}
        aria-label="PC abilities"
      >
        <Space orientation="vertical" size="middle" style={{ display: "flex" }}>
          {reference.isPending && <p role="status">Loading PC reference…</p>}
          {location.kind === "encounter" && (doc || reference.data === null) && (
            <Space wrap>
              {doc && (
                <Typography.Text type="secondary">
                  {doc.name}’s party reference
                </Typography.Text>
              )}
              <Button
                disabled={paused || busy}
                onClick={() => {
                  setMemberKey(doc?.member_key);
                  setEditingAssociation(!editingAssociation);
                }}
              >
                {doc ? "Change party member" : "Attach to party member"}
              </Button>
              {doc && (
                <Button
                  disabled={paused || busy}
                  onClick={() =>
                    attach.mutate({
                      encounter: location,
                      association: { kind: "detach" },
                    })
                  }
                >
                  Detach
                </Button>
              )}
            </Space>
          )}
          {location.kind === "encounter" && editingAssociation && (
            <>
              {parties.error && (
                <ErrorNotice
                  error={parties.error}
                  title="Could not load party members"
                />
              )}
              <Space wrap>
                <Select
                  aria-label="Party member"
                  placeholder="Choose a party member"
                  options={associationOptions}
                  value={memberKey}
                  onChange={setMemberKey}
                  loading={parties.isPending}
                  style={{ minWidth: "14rem" }}
                />
                <Button
                  type="primary"
                  disabled={paused || !memberKey || busy}
                  onClick={() => {
                    if (memberKey)
                      attach.mutate({
                        encounter: location,
                        association: { kind: "attach", member_key: memberKey },
                      });
                  }}
                >
                  Attach
                </Button>
              </Space>
            </>
          )}
          {doc && (
            <>
              <Space wrap>
                {doc.level !== undefined && (
                  <Typography.Text type="secondary">Level {doc.level}</Typography.Text>
                )}
                {onRefresh && (
                  <Button disabled={paused} onClick={onRefresh}>
                    {doc.imported
                      ? "Refresh from Pathbuilder"
                      : "Import from Pathbuilder"}
                  </Button>
                )}
              </Space>
              {doc.imported ? (
                <>
                  <Input
                    aria-label="Filter abilities"
                    placeholder="Filter abilities"
                    value={filter}
                    onChange={(event) => setFilter(event.target.value)}
                    allowClear
                  />
                  <List
                    size="small"
                    dataSource={visible}
                    locale={{
                      emptyText: filter
                        ? "No matching abilities."
                        : "No abilities in this export.",
                    }}
                    renderItem={(ability) => {
                      const label = abilityLabel(ability);
                      return (
                        <List.Item key={ability.ability_key}>
                          <Space
                            wrap
                            style={{ width: "100%", justifyContent: "space-between" }}
                          >
                            {ability.record_key && ability.record_title ? (
                              <Button
                                type="link"
                                style={{
                                  paddingInline: 0,
                                  whiteSpace: "normal",
                                  height: "auto",
                                  textAlign: "start",
                                }}
                                onClick={(event) => {
                                  resultAnchor.current = event.currentTarget;
                                  preview.open(
                                    ability.record_key!,
                                    event.currentTarget,
                                  );
                                }}
                              >
                                {label}
                              </Button>
                            ) : (
                              <Typography.Text>{label}</Typography.Text>
                            )}
                            <Space wrap>
                              {ability.record_key && ability.record_title ? (
                                <Dropdown
                                  trigger={["click"]}
                                  menu={{
                                    items: [
                                      { key: "change", label: "Change link" },
                                      ...(ability.manual
                                        ? [
                                            {
                                              key: "automatic",
                                              label: "Use automatic match",
                                            },
                                          ]
                                        : []),
                                    ],
                                    onClick: ({ key }) => {
                                      if (key === "change")
                                        startSearch(
                                          ability,
                                          resultAnchor.current ?? undefined,
                                        );
                                      else
                                        save.mutate({
                                          target: doc,
                                          ability,
                                          selection: { kind: "automatic" },
                                        });
                                    },
                                  }}
                                >
                                  <Button
                                    size="small"
                                    aria-label={`Link options for ${label}`}
                                    disabled={paused || busy}
                                    onClick={(event) => {
                                      resultAnchor.current = event.currentTarget;
                                    }}
                                  >
                                    Link options
                                  </Button>
                                </Dropdown>
                              ) : (
                                <>
                                  <Button
                                    size="small"
                                    aria-label={`Search for ${label}`}
                                    onClick={(event) =>
                                      startSearch(ability, event.currentTarget)
                                    }
                                  >
                                    {ability.record_key
                                      ? "Find replacement"
                                      : "Search for this ability"}
                                  </Button>
                                  {ability.manual && (
                                    <Button
                                      size="small"
                                      disabled={paused || busy}
                                      onClick={() =>
                                        save.mutate({
                                          target: doc,
                                          ability,
                                          selection: { kind: "automatic" },
                                        })
                                      }
                                    >
                                      Use automatic match
                                    </Button>
                                  )}
                                </>
                              )}
                            </Space>
                          </Space>
                        </List.Item>
                      );
                    }}
                  />
                </>
              ) : (
                <Typography.Text type="secondary">
                  This PC has no Pathbuilder reference yet.
                </Typography.Text>
              )}
            </>
          )}
        </Space>
      </section>
      {search && (
        <section
          hidden={previewOpen}
          style={{ minHeight: 0, overflow: "auto" }}
          aria-label="Ability search"
        >
          <Form
            onFinish={() => {
              const next = draftQuery?.trim();
              if (next) setSearch({ ...search, query: next });
            }}
          >
            <Space.Compact style={{ width: "100%" }}>
              <Input
                aria-label="Search Atlas rules"
                ref={searchInput}
                value={draftQuery}
                onChange={(event) =>
                  setSearch({ ...search, draft: event.target.value })
                }
              />
              <Button type="primary" htmlType="submit" disabled={!draftQuery?.trim()}>
                Search
              </Button>
            </Space.Compact>
          </Form>
          {!execution.ready && <p>Search is getting ready.</p>}
          {result.error && (
            <ErrorNotice
              error={result.error}
              title="Could not search Atlas"
              action={<Button onClick={() => void result.refetch()}>Retry</Button>}
            />
          )}
          <List
            size="small"
            dataSource={result.data?.rows}
            loading={result.isPending && execution.ready}
            locale={{ emptyText: "No rules found." }}
            renderItem={({ record }) => (
              <List.Item key={record.record_key}>
                <Space orientation="vertical" size={0} style={{ maxWidth: "100%" }}>
                  <Button
                    type="link"
                    style={{
                      paddingInline: 0,
                      whiteSpace: "normal",
                      height: "auto",
                      textAlign: "start",
                    }}
                    onClick={(event) => {
                      resultAnchor.current = event.currentTarget;
                      preview.open(
                        record.record_key,
                        event.currentTarget,
                        rootNavigation(record.record_key),
                      );
                    }}
                  >
                    {record.title}
                  </Button>
                  <Typography.Text type="secondary">
                    {[
                      record.kind_label,
                      record.publication ?? record.pack,
                      record.edition,
                    ]
                      .filter(Boolean)
                      .join(" · ")}
                  </Typography.Text>
                </Space>
              </List.Item>
            )}
          />
          <a
            href={destinationHref(
              searchDestination({ query: `?${new URLSearchParams({ q: query })}` }),
            )}
          >
            Open full search
          </a>
        </section>
      )}
      {previewOpen && (
        <div style={{ minHeight: 0, overflow: "auto" }} data-reference-body>
          <section ref={ruleContent} data-reference-host aria-label="Rule preview">
            <div ref={ruleHeading} tabIndex={-1} aria-label="Rule preview" />
            {doc &&
              search &&
              selectedRecord &&
              preview.detail?.selected.embedded_path.length === 0 && (
                <Button
                  type="primary"
                  disabled={paused || busy}
                  loading={save.isPending}
                  onClick={() => {
                    if (!preview.detail) return;
                    save.mutate({
                      target: doc,
                      ability: search.ability,
                      selection: {
                        kind: "manual",
                        record_key: selectedRecord,
                        expected_artifact_generation:
                          preview.detail.artifact_generation,
                      },
                    });
                  }}
                >
                  Link this rule
                </Button>
              )}
            <ReferenceHelpProvider
              onHelp={preview.openHelp}
              onReference={preview.open}
              generation={
                preview.detail?.artifact_generation ??
                preview.definition?.artifact_generation
              }
            >
              {preview.destination?.kind === "definition" ? (
                <DefinitionDetailPane
                  detail={preview.definition}
                  loading={preview.loading}
                  error={preview.error}
                  onRetry={preview.retry}
                  onReference={preview.open}
                />
              ) : (
                <RecordDetailPane
                  detail={preview.detail}
                  loading={preview.loading}
                  error={preview.error}
                  onRetry={preview.retry}
                  onReload={preview.reload}
                  onReference={preview.open}
                  onRankChange={preview.changeRank}
                  landing={preview.landing}
                  onLandingReady={onLandingReady}
                  showRelated={false}
                />
              )}
            </ReferenceHelpProvider>
          </section>
        </div>
      )}
    </AtlasDrawer>
  );
}

function resultReferenceKey(location: Location) {
  return location.kind === "party"
    ? [...allPcReferences, "party", location.partyKey, location.memberKey]
    : [...allPcReferences, "encounter", location.encounterKey, location.participantKey];
}
