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
import { Ellipsis } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useLayoutEffect, useRef, useState } from "react";
import {
  getEncounterPcReference,
  getCharacter,
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
import { refreshCharacters, useCharacters } from "./characterQueries";

type CharacterLocation = { kind: "character"; characterKey: string };
type EncounterLocation = {
  kind: "encounter";
  encounterKey: string;
  participantKey: string;
  revision: number;
};
type Location = CharacterLocation | EncounterLocation;

const allPcReferences = ["pc-reference"] as const;
const getPopupContainer = (trigger?: HTMLElement) =>
  trigger?.closest<HTMLElement>(".pc-character-drawer-root") ?? document.body;
const abilityLabel = (ability: PcAbilityView) =>
  ability.choice ? `${ability.label} (${ability.choice})` : ability.label;

export function CharacterReference({
  location,
  name,
  opener,
  onClose,
  onRefresh,
  mode = "drawer",
}: {
  location: Location;
  name: string;
  opener?: HTMLElement | null;
  onClose?: () => void;
  mode?: "page" | "drawer";
  onRefresh?: () => void;
}) {
  const client = useQueryClient();
  const titleId = useId();
  const paused = useSavedWritesPaused();
  const [closing, setClosing] = useState(false);
  const [filter, setFilter] = useState("");
  const [selectedAbility, setSelectedAbility] = useState<string>();
  const [search, setSearch] = useState<{
    ability: PcAbilityView;
    query: string;
    draft: string;
  } | null>(null);
  const [editingAssociation, setEditingAssociation] = useState(false);
  const [memberKey, setMemberKey] = useState<string>();
  const searchInput = useRef<React.ComponentRef<typeof Input>>(null);
  const ruleHeading = useRef<HTMLHeadingElement>(null);
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
  const execution = searchExecution(
    profile.error ? { ...profile, keywordSearch: true } : profile,
    "text_search",
  );
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
      location.kind === "character"
        ? getCharacter(location.characterKey, signal)
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
  const characters = useCharacters(location.kind === "encounter" && editingAssociation);
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
      savePcAbilityLink(target.character_key, ability.ability_key, {
        expected_character_revision: target.character_revision,
        selection,
      }),
    onSuccess: async (saved) => {
      client.setQueryData(resultReferenceKey(location), saved);
      await refreshCharacters(client);
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
    setSelectedAbility(ability.ability_key);
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
  const associationOptions = characters.data?.characters.map((character) => ({
    value: character.character_key,
    label: character.name,
  }));
  const busy = save.isPending || attach.isPending;
  const content = (
    <>
      {mode === "page" && (search || previewOpen) && (
        <Button className="character-reference__back" onClick={back}>
          Back
        </Button>
      )}
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
        <ErrorNotice error={attach.error} title="Could not update character link" />
      )}
      <section
        hidden={Boolean(search)}
        className="character-reference__index"
        style={{ minHeight: 0, overflow: "auto" }}
        aria-label="PC abilities"
      >
        <Space orientation="vertical" size="middle" style={{ display: "flex" }}>
          {reference.isPending && <p role="status">Loading PC reference…</p>}
          {location.kind === "encounter" && (doc || reference.data === null) && (
            <Space wrap>
              {doc && (
                <Typography.Text type="secondary">
                  {doc.name}’s character reference
                </Typography.Text>
              )}
              <Button
                disabled={paused || busy}
                onClick={() => {
                  setMemberKey(doc?.character_key);
                  setEditingAssociation(!editingAssociation);
                }}
              >
                {doc ? "Change character" : "Attach to character"}
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
              {characters.error && (
                <ErrorNotice
                  error={characters.error}
                  title="Could not load characters"
                />
              )}
              <Space wrap>
                <Select
                  aria-label="Character"
                  placeholder="Choose a character"
                  options={associationOptions}
                  value={memberKey}
                  onChange={setMemberKey}
                  loading={characters.isPending}
                  style={{ minWidth: "14rem" }}
                />
                <Button
                  type="primary"
                  disabled={paused || !memberKey || busy}
                  onClick={() => {
                    if (memberKey)
                      attach.mutate({
                        encounter: location,
                        association: { kind: "attach", character_key: memberKey },
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
              {(mode === "drawer" || onRefresh) && (
                <Space wrap>
                  {mode === "drawer" && doc.level !== undefined && (
                    <Typography.Text type="secondary">
                      Level {doc.level}
                    </Typography.Text>
                  )}
                  {onRefresh && (
                    <Button disabled={paused} onClick={onRefresh}>
                      {doc.imported
                        ? "Refresh from Pathbuilder"
                        : "Import from Pathbuilder"}
                    </Button>
                  )}
                </Space>
              )}
              {doc.imported ? (
                <>
                  <Input
                    aria-label="Find an ability"
                    placeholder="Find an ability"
                    value={filter}
                    onChange={(event) => setFilter(event.target.value)}
                    allowClear
                  />
                  <nav
                    className="character-reference__sections"
                    aria-label="Ability sections"
                  >
                    {doc.sections
                      .filter((section) =>
                        section.abilities.some((ability) =>
                          visible?.some(
                            (item) => item.ability_key === ability.ability_key,
                          ),
                        ),
                      )
                      .map((section) => (
                        <a key={section.kind} href={`#${titleId}-${section.kind}`}>
                          {section.label}
                        </a>
                      ))}
                  </nav>
                  {doc.sections.map((section) => {
                    const sectionAbilities = section.abilities.filter((ability) =>
                      visible?.some((item) => item.ability_key === ability.ability_key),
                    );
                    if (!sectionAbilities.length) return null;
                    return (
                      <section
                        key={section.kind}
                        id={`${titleId}-${section.kind}`}
                        className="character-ability-section"
                      >
                        <h3>
                          {section.label} <span>{sectionAbilities.length}</span>
                        </h3>
                        <List
                          size="small"
                          dataSource={sectionAbilities}
                          locale={{
                            emptyText: filter
                              ? "No matching abilities."
                              : "No abilities in this export.",
                          }}
                          renderItem={(ability) => {
                            const label = abilityLabel(ability);
                            return (
                              <List.Item key={ability.ability_key}>
                                <div className="character-ability-row">
                                  <div>
                                    {ability.record_key && ability.record_title ? (
                                      <Button
                                        type="link"
                                        className="character-ability-link"
                                        aria-current={
                                          previewOpen &&
                                          selectedAbility === ability.ability_key
                                            ? "true"
                                            : undefined
                                        }
                                        style={{
                                          paddingInline: 0,
                                          whiteSpace: "normal",
                                          height: "auto",
                                          textAlign: "start",
                                        }}
                                        onClick={(event) => {
                                          resultAnchor.current = event.currentTarget;
                                          setSelectedAbility(ability.ability_key);
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
                                    {ability.record_title &&
                                      ability.record_title
                                        .trim()
                                        .toLocaleLowerCase() !==
                                        ability.label.trim().toLocaleLowerCase() && (
                                        <span className="character-secondary">
                                          {ability.record_title}
                                        </span>
                                      )}
                                  </div>
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
                                          type="text"
                                          icon={
                                            <Ellipsis size={16} aria-hidden="true" />
                                          }
                                          disabled={paused || busy}
                                          onClick={(event) => {
                                            resultAnchor.current = event.currentTarget;
                                          }}
                                        ></Button>
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
                                            : "Find rule"}
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
                                </div>
                              </List.Item>
                            );
                          }}
                        />
                      </section>
                    );
                  })}
                  {visible?.length === 0 && (
                    <p>
                      {filter
                        ? "No matching abilities."
                        : "No abilities in this export."}
                    </p>
                  )}
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
          className="character-reference__index"
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
        <div
          className="character-reference__rule"
          style={{ minHeight: 0, overflow: "auto" }}
          data-reference-body
        >
          <section ref={ruleContent} data-reference-host aria-label="Rule preview">
            <h2 ref={ruleHeading} tabIndex={-1} className="sr-only">
              {preview.detail?.record.title ??
                preview.definition?.label ??
                "Rule preview"}
            </h2>
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
    </>
  );
  return mode === "page" ? (
    <div
      className={`character-reference character-reference--page${previewOpen ? " character-reference--reading" : ""}`}
    >
      {content}
    </div>
  ) : (
    <AtlasDrawer
      open={!closing}
      aria-labelledby={titleId}
      onClose={() => setClosing(true)}
      closable={!busy}
      keyboard={!busy}
      mask={{ closable: !busy }}
      afterOpenChange={(open) => {
        if (!open) {
          onClose?.();
          opener?.focus({ preventScroll: true });
        }
      }}
      size="min(1080px, 100vw)"
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
        body: { display: "flex", flexDirection: "column", gap: 12, minHeight: 0 },
      }}
      drawerRender={(content) => (
        <ConfigProvider getPopupContainer={getPopupContainer}>{content}</ConfigProvider>
      )}
    >
      <div
        className={`character-reference${previewOpen ? " character-reference--reading" : ""}`}
      >
        {content}
      </div>
    </AtlasDrawer>
  );
}

function resultReferenceKey(location: Location) {
  return location.kind === "character"
    ? ["character", location.characterKey]
    : [...allPcReferences, "encounter", location.encounterKey, location.participantKey];
}

export function PcCharacterDrawer(
  props: Omit<React.ComponentProps<typeof CharacterReference>, "mode">,
) {
  return <CharacterReference {...props} mode="drawer" />;
}
