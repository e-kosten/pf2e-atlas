import {
  Button,
  Dropdown,
  Empty,
  Form,
  Input,
  List,
  Skeleton,
  Space,
  Typography,
} from "antd";
import { ArrowLeft, Ellipsis } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useId, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { searchPage } from "../../api/atlasApi";
import type {
  PcAbilitySelection,
  PcAbilityView,
  PcReferenceView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { DefinitionDetailPane } from "../../shared/records/DefinitionDetailPane";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { ReferenceHelpProvider } from "../../shared/records/ReferenceHelp";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { usePreviewReadingPosition } from "../../shared/records/usePreviewReadingPosition";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";
import { searchExecution } from "../../shared/search/searchExecution";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { RecordEditionBadge } from "../../shared/records/RecordEditionBadge";

type AbilityDocument = Pick<PcReferenceView, "abilities" | "sections" | "imported">;

type AbilityLinkMutation = {
  error: Error | null;
  isPending: boolean;
  reset: () => void;
  mutate: (
    input: {
      ability: PcAbilityView;
      selection: PcAbilitySelection;
      recordTitle?: string;
    },
    options: { onSuccess: () => void },
  ) => void;
};

const abilityLabel = (ability: PcAbilityView) =>
  ability.choice ? `${ability.label} (${ability.choice})` : ability.label;

export function PcAbilityBrowser({
  doc,
  toolbar,
  link,
  linkErrorAction,
  blocked = false,
  review = false,
}: {
  doc?: AbilityDocument | null;
  toolbar?: ReactNode;
  link: AbilityLinkMutation;
  linkErrorAction?: ReactNode;
  blocked?: boolean;
  review?: boolean;
}) {
  const titleId = useId();
  const [filter, setFilter] = useState("");
  const [selectedAbility, setSelectedAbility] = useState<string>();
  const abilityRows = useRef(new Map<string, HTMLElement>());
  const restoreAbility = useRef<string | null>(null);
  const [search, setSearch] = useState<{
    ability: PcAbilityView;
    query: string;
    draft: string;
  } | null>(null);
  const searchInput = useRef<React.ComponentRef<typeof Input>>(null);
  const abilityFilter = useRef<React.ComponentRef<typeof Input>>(null);
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
  const draftQuery = search?.draft;
  const searching = Boolean(search);
  useLayoutEffect(() => {
    if (previewOpen) {
      if (!resultAnchor.current?.getClientRects().length)
        ruleHeading.current?.focus({ preventScroll: true });
    } else if (searching) searchInput.current?.focus({ preventScroll: true });
  }, [previewOpen, searching]);
  const { address, canGoBack, restoringHistory } = preview.readingHistory;
  useLayoutEffect(() => {
    if (canGoBack && !restoringHistory && document.activeElement === document.body)
      ruleHeading.current?.focus({ preventScroll: true });
  }, [address, canGoBack, restoringHistory]);
  const searchAnchor = useRef<HTMLElement>(null);
  const resultAnchor = useRef<HTMLElement>(null);
  const restoreOnBack = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => {
    if (!previewOpen && !searching && restoreAbility.current) {
      const key = restoreAbility.current;
      const frame = requestAnimationFrame(() => {
        const target = abilityRows.current
          .get(key)
          ?.querySelector<HTMLElement>(".character-ability-link");
        if (target) target.focus({ preventScroll: true });
        else abilityFilter.current?.focus({ preventScroll: true });
        restoreAbility.current = null;
      });
      return () => cancelAnimationFrame(frame);
    } else if (!previewOpen && restoreOnBack.current) {
      if (restoreOnBack.current.isConnected)
        restoreOnBack.current.focus({ preventScroll: true });
      else if (searching) searchInput.current?.focus({ preventScroll: true });
      else abilityFilter.current?.focus({ preventScroll: true });
      restoreOnBack.current = null;
    }
  }, [previewOpen, searching]);
  function startSearch(ability: PcAbilityView, anchor?: HTMLElement | null) {
    searchAnchor.current = anchor ?? (document.activeElement as HTMLElement | null);
    link.reset();
    setSelectedAbility(ability.ability_key);
    preview.closeForNavigation();
    setSearch({ ability, query: ability.label, draft: ability.label });
  }
  function returnToList() {
    restoreOnBack.current = resultAnchor.current;
    preview.closeForNavigation();
  }
  const visible = doc?.abilities.filter((ability) => {
    const term = filter.trim().toLocaleLowerCase();
    return [abilityLabel(ability), ability.record_title].some((label) =>
      label?.toLocaleLowerCase().includes(term),
    );
  });

  const busy = blocked || link.isPending;
  function submitLink(
    ability: PcAbilityView,
    selection: PcAbilitySelection,
    recordTitle?: string,
  ) {
    link.mutate(
      { ability, selection, recordTitle },
      {
        onSuccess: () => {
          if (!search) return;
          restoreAbility.current = ability.ability_key;
          preview.closeForNavigation();
          setSearch(null);
        },
      },
    );
  }

  const split = Boolean(doc?.imported && !search) || previewOpen;
  return (
    <div className="pc-ability-browser">
      <div
        data-reference-host
        className={`character-reference${split ? " character-reference--split" : ""}${search ? " character-reference--searching" : ""}${previewOpen ? " character-reference--reading" : ""}`}
      >
        {link.error && (
          <ErrorNotice
            error={link.error}
            title={
              review
                ? "Review the import again to link this rule"
                : "Could not save rule link"
            }
            action={linkErrorAction}
          />
        )}
        <section
          hidden={Boolean(search)}
          className="character-reference__index"
          style={{ minHeight: 0, overflow: "auto" }}
          aria-label="PC abilities"
        >
          <Space orientation="vertical" size="middle" style={{ display: "flex" }}>
            {toolbar}
            {doc &&
              (doc.imported ? (
                <>
                  <Input
                    aria-label="Find an ability"
                    ref={abilityFilter}
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
                                <div
                                  className="character-ability-row"
                                  ref={(node) => {
                                    if (node)
                                      abilityRows.current.set(
                                        ability.ability_key,
                                        node,
                                      );
                                    else
                                      abilityRows.current.delete(ability.ability_key);
                                  }}
                                >
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
                                          preview.selectRecord(
                                            ability.record_key!,
                                            event.currentTarget,
                                          );
                                        }}
                                      >
                                        {label}
                                      </Button>
                                    ) : (
                                      <>
                                        <Typography.Text>{label}</Typography.Text>
                                        {review && (
                                          <span className="character-secondary">
                                            {ability.record_key
                                              ? "Reference unavailable"
                                              : ability.ambiguous
                                                ? "Multiple matches"
                                                : "No automatic match"}
                                          </span>
                                        )}
                                      </>
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
                                    {review ? (
                                      <Button
                                        size="small"
                                        disabled={busy}
                                        aria-label={`${ability.record_title ? "Change link" : "Link rule"} for ${label}`}
                                        onClick={(event) =>
                                          startSearch(ability, event.currentTarget)
                                        }
                                      >
                                        {ability.record_title
                                          ? "Change link"
                                          : "Link rule…"}
                                      </Button>
                                    ) : ability.record_key && ability.record_title ? (
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
                                                abilityRows.current
                                                  .get(ability.ability_key)
                                                  ?.querySelector<HTMLElement>(
                                                    ".character-link-options",
                                                  ),
                                              );
                                            else
                                              submitLink(ability, {
                                                kind: "automatic",
                                              });
                                          },
                                        }}
                                      >
                                        <Button
                                          size="small"
                                          className="character-link-options"
                                          aria-label={`Link options for ${label}`}
                                          type="text"
                                          icon={
                                            <Ellipsis size={16} aria-hidden="true" />
                                          }
                                          disabled={busy}
                                        ></Button>
                                      </Dropdown>
                                    ) : (
                                      <>
                                        <Button
                                          size="small"
                                          disabled={link.isPending}
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
                                            disabled={busy}
                                            onClick={() =>
                                              submitLink(ability, { kind: "automatic" })
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
              ))}
          </Space>
        </section>
        {search && (
          <section className="pc-rule-search" aria-label="Ability search">
            <header className="pc-rule-search__header">
              <Button
                icon={<ArrowLeft size={16} />}
                onClick={() => {
                  restoreOnBack.current = searchAnchor.current;
                  preview.closeForNavigation();
                  setSearch(null);
                  link.reset();
                }}
              >
                Back to abilities
              </Button>
              <h2>Find rule for {abilityLabel(search.ability)}</h2>
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
                  <Button
                    type="primary"
                    htmlType="submit"
                    disabled={!draftQuery?.trim()}
                  >
                    Search
                  </Button>
                </Space.Compact>
              </Form>
            </header>
            <div className="pc-rule-search__status">
              {!execution.ready ? (
                <LoadingStatus label="Preparing rule search…" />
              ) : result.isFetching || result.isPending ? (
                <LoadingStatus
                  label={result.data ? "Updating rules…" : "Searching Atlas…"}
                />
              ) : result.data ? (
                <span>
                  {result.data.rows.length}{" "}
                  {result.data.rows.length === 1 ? "result" : "results"}
                </span>
              ) : null}
            </div>
            <div
              className="pc-rule-search__results"
              aria-busy={!execution.ready || result.isFetching}
            >
              {result.error && (
                <ErrorNotice
                  error={result.error}
                  title="Could not search Atlas"
                  action={<Button onClick={() => void result.refetch()}>Retry</Button>}
                />
              )}
              {!execution.ready ||
              (result.isPending && !result.data) ||
              (result.isFetching && !result.data?.rows.length) ? (
                <div className="pc-rule-search__placeholder" aria-hidden="true">
                  {[0, 1, 2].map((key) => (
                    <Skeleton
                      key={key}
                      active
                      title={{ width: "60%" }}
                      paragraph={{ rows: 1, width: "85%" }}
                    />
                  ))}
                </div>
              ) : result.data?.rows.length ? (
                <ul className="pc-rule-search__list">
                  {result.data.rows.map(({ record }) => (
                    <li key={record.record_key}>
                      <Button
                        type="text"
                        block
                        className="pc-rule-result"
                        aria-label={record.title}
                        aria-describedby={`${titleId}-${record.record_key}-kind ${titleId}-${record.record_key}-source`}
                        aria-current={
                          selectedRecord === record.record_key ? "true" : undefined
                        }
                        styles={{
                          root: {
                            height: "auto",
                            whiteSpace: "normal",
                            textAlign: "start",
                            padding: "var(--space-group)",
                            borderRadius: 0,
                          },
                        }}
                        onClick={(event) => {
                          resultAnchor.current = event.currentTarget;
                          preview.selectRecord(
                            record.record_key,
                            event.currentTarget,
                            rootNavigation(record.record_key),
                          );
                        }}
                      >
                        <span className="pc-rule-result__content">
                          <span className="pc-rule-result__identity">
                            <strong>{record.title}</strong>
                            <span id={`${titleId}-${record.record_key}-kind`}>
                              {[record.kind_label, record.level_label]
                                .filter(Boolean)
                                .join(" ")}
                            </span>
                          </span>
                          <span
                            className="pc-rule-result__source"
                            id={`${titleId}-${record.record_key}-source`}
                          >
                            <span>{record.publication ?? record.pack}</span>
                            <RecordEditionBadge edition={record.edition} />
                          </span>
                        </span>
                      </Button>
                    </li>
                  ))}
                </ul>
              ) : result.isSuccess && !result.isFetching ? (
                <Empty
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  description="No rules found. Try a different name."
                />
              ) : null}
            </div>
          </section>
        )}
        {previewOpen && (
          <div
            className="character-reference__rule"
            style={{ minHeight: 0, overflow: "auto" }}
            data-reference-body
          >
            <section ref={ruleContent} aria-label="Rule preview">
              <div className="character-reference__rule-actions">
                <Button
                  className="character-reference__return"
                  icon={<ArrowLeft size={16} />}
                  onClick={returnToList}
                >
                  {search ? "Back to results" : "Abilities"}
                </Button>
                {preview.readingHistory.canGoBack && (
                  <Button
                    icon={<ArrowLeft size={16} />}
                    onClick={preview.readingHistory.onBack}
                  >
                    Back
                  </Button>
                )}
              </div>
              <h2 ref={ruleHeading} tabIndex={-1} className="sr-only">
                {preview.detail?.record.title ??
                  preview.definition?.label ??
                  "Rule preview"}
              </h2>
              {doc &&
                search &&
                selectedRecord &&
                preview.detail?.selected.embedded_path.length === 0 && (
                  <div className="pc-rule-link-action">
                    <Button
                      type="primary"
                      disabled={busy || preview.loading}
                      loading={link.isPending}
                      onClick={() => {
                        if (!preview.detail) return;
                        submitLink(
                          search.ability,
                          {
                            kind: "manual",
                            record_key: selectedRecord,
                            expected_artifact_generation:
                              preview.detail.artifact_generation,
                          },
                          preview.detail.record.title,
                        );
                      }}
                    >
                      Link this rule
                    </Button>
                  </div>
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

        {!search && !previewOpen && doc?.imported && (
          <div className="character-reference__rule character-reference__prompt">
            Select an ability to read its rules.
          </div>
        )}
      </div>
    </div>
  );
}
