import { FilterDisclosure } from "./FilterDisclosure";
import { FilterFieldSelect } from "./FilterFieldSelect";
import { restoreOverlayFocus } from "../ui/overlays/restoreOverlayFocus";
import { createPortal } from "react-dom";
import { AtlasDrawer as Drawer } from "../ui/overlays/AtlasDrawer";
import { ErrorNotice } from "../feedback/ErrorNotice";
import {
  Button,
  ConfigProvider,
  Form,
  Input,
  Popover,
  Select,
  Skeleton,
  Space,
  Tooltip,
  Typography,
  theme,
  type InputRef,
} from "antd";
import {
  ClearOutlined,
  DownOutlined,
  CloseOutlined,
  FilterOutlined,
} from "@ant-design/icons";
import {
  createContext,
  useContext,
  useId,
  type CSSProperties,
  type ReactNode,
  type RefObject,
  useMemo,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import {
  QueryBuilder,
  RuleComponents,
  RuleGroupBodyComponents,
  type UseRuleGroup,
  useRule,
  add,
  update,
  type RuleType,
  type RuleProps,
  type RuleGroupType,
  type ValueEditorProps,
  type FieldSelectorProps,
  type OperatorSelectorProps,
  type CombinatorSelectorProps,
  type ActionProps,
} from "react-querybuilder";
import { QueryBuilderAntD } from "@react-querybuilder/antd";
import type {
  FilterEditorFieldView,
  FilterEditorView,
  QueryPredicate,
  QueryRelationship,
} from "../../generated/atlas";
import type { SearchWorkspaceState } from "../../features/search/useSearchWorkspace";
import {
  builderPredicate,
  commonDraftCondition,
  commonDraftRange,
  BOOLEAN_CONDITION,
  LINK_CONDITION,
  RELATED_CONDITION,
  emptyGroup,
  isClearedOperand,
  numericLiteral,
  replaceCommonDraft,
  replaceCommonRangeDraft,
  rangeRule,
  type PredicateRule,
} from "./queryBuilder";
import {
  chipLinkClauses,
  clearAllFilters,
  countAppliedFilterConditions,
  hasAdvancedFilterConditions,
  type CommonCondition,
  type FilterPanelState,
} from "./filterControls";
import { CompactSelect } from "../ui/pickers/CompactSelect";
import { FilterValueInput } from "./FilterValueInput";
import { SearchEntry, type SimilarSearchCriterion } from "./SearchEntry";
import { CommonChoicePicker } from "./FilterChoicePicker";
import { PrimaryToolbar } from "../ui/PrimaryToolbar";
import { useFilterEditor } from "./useFilterEditor";
import { existingClauseId } from "./useFilterDiscovery";
import { isBuilderGroup, normalizeBuilderEdit } from "./builderEdits";
import {
  LinkPicker,
  LinkPickerContext,
  type LinkPickerProps,
} from "./LinkPickerContext";
import { LinkFilterControls } from "./LinkFilterControls";
import { countLinkConditions, type LinkLeaf } from "./linkPredicates";
import { sameRelationship } from "./linkPredicates";
import {
  RelatedEntryEditor,
  RelatedEntrySummary,
  isEmbeddedEntryField,
} from "./RelatedEntryEditor";
import { CollectionValueInput, isCompactCollection } from "./CollectionValueInput";
import { transitionOperand } from "./operatorTransitions";
import { ActorMetricPicker } from "./ActorMetricPicker";
import {
  ACTOR_METRIC_CHOOSERS,
  actorMetricKind,
  isActorMetricChooser,
  type ActorMetricKind,
} from "./actorMetricFields";

const getAdvancedPopupContainer = (trigger?: HTMLElement) =>
  trigger?.closest<HTMLElement>(".advanced-filter-drawer-root") ?? document.body;

const LinkEditingContext = createContext<{
  open: (
    rule: PredicateRule,
    replace: (next: PredicateRule) => void,
    opener?: HTMLElement,
  ) => void;
  title: (leaf: LinkLeaf) => string;
} | null>(null);

export function FilterPanel({
  workspace,
  children,
}: {
  workspace: SearchWorkspaceState;
  children?: ReactNode;
}) {
  return <FilterControls filterState={workspace}>{children}</FilterControls>;
}
export function FilterControls({
  filterState: workspace,
  includeSearch = true,
  includeLinks = true,
  layout = "panel",
  searchRowEnd,
  searchInputRef,
  similarSearch,
  additionalFilters,
  children,
  pane,
}: {
  filterState: FilterPanelState;
  includeSearch?: boolean;
  includeLinks?: boolean;
  layout?: "panel" | "toolbar";
  pane?: { narrow: boolean; summary: ReactNode };
  searchRowEnd?: ReactNode;
  searchInputRef?: RefObject<InputRef | null>;
  similarSearch?: SimilarSearchCriterion;
  additionalFilters?: ReactNode;
  children?: ReactNode;
}) {
  const { search, setSearch } = workspace;
  const [advanced, setAdvanced] = useState(false);
  const [selectionContainer, setSelectionContainer] = useState<HTMLDivElement | null>(
    null,
  );
  const advancedTrigger = useRef<HTMLButtonElement>(null);
  const linksTrigger = useRef<HTMLButtonElement>(null);
  const [draftRevision, setDraftRevision] = useState(0);
  const editor = workspace.filterEditor;
  const fields = useMemo(
    () => editor?.groups.flatMap((group) => group.fields) ?? [],
    [editor],
  );
  const prepared = useFilterEditor(search, fields, setSearch);
  const Picker = useContext(LinkPickerContext);
  const [picker, setPicker] = useState<
    (LinkPickerProps & { key: string; advanced: boolean }) | null
  >(null);
  const [draftLabels, setDraftLabels] = useState<
    Record<
      string,
      { relationship: QueryRelationship; title: string; generation: string }
    >
  >({});
  const latestEditor = useRef(prepared);
  useLayoutEffect(() => {
    latestEditor.current = prepared;
  }, [prepared]);
  const generation =
    workspace.filterDiscoveryContext?.expected_artifact_generation ??
    editor?.artifact_generation;
  const latestContext = useRef({ generation, filter: search.filter });
  useLayoutEffect(() => {
    latestContext.current = { generation, filter: search.filter };
  }, [generation, search.filter]);
  function linkTitle(leaf: LinkLeaf): string {
    const draft = draftLabels[leaf.clause_id ?? ""];
    if (
      draft &&
      (!generation || draft.generation === generation) &&
      sameRelationship(draft.relationship, leaf.relationship)
    )
      return draft.title;
    const presentation = editor?.relationship_clauses.find(
      (entry) =>
        entry.clause_id === leaf.clause_id &&
        sameRelationship(entry.relationship, leaf.relationship),
    );
    if (
      presentation &&
      (!workspace.filterDiscoveryContext?.expected_artifact_generation ||
        editor?.artifact_generation ===
          workspace.filterDiscoveryContext.expected_artifact_generation)
    )
      return presentation.status === "available"
        ? presentation.title +
            (leaf.relationship.selection.embedded_path.length
              ? ` · ${presentation.record?.title ?? presentation.title}`
              : "")
        : "Record unavailable";
    return workspace.filterDiscoveryError ? "Record unavailable" : "Loading record…";
  }
  function openPicker(
    initial: QueryRelationship | undefined,
    id: string,
    confirm: (relationship: QueryRelationship, excluded: boolean) => string | null,
    {
      opener,
      advanced = false,
      initialExcluded = false,
    }: {
      opener?: HTMLElement;
      advanced?: boolean;
      initialExcluded?: boolean;
    } = {},
  ) {
    if (!Picker) throw new Error("Link condition picker provider is required");
    const openingQuery = prepared.query;
    let restoreToolbarFocus = false;
    const parentContext = prepared.linkDiscoveryContext(id, advanced);
    setPicker({
      key: crypto.randomUUID(),
      advanced,
      open: true,
      initial,
      initialExcluded,
      parent: {
        ...parentContext,
        filter: workspace.fixedFilter
          ? {
              kind: "all_of",
              children: [
                workspace.fixedFilter,
                ...[parentContext.filter].flatMap((filter) => (filter ? [filter] : [])),
              ],
            }
          : parentContext.filter,
        expected_artifact_generation: generation ?? null,
        list_ref:
          workspace.filterDiscoveryContext?.kind === "saved_list"
            ? workspace.filterDiscoveryContext.list_ref
            : null,
        trait_key:
          workspace.resultRequest?.criteria.kind === "trait_references"
            ? workspace.resultRequest.criteria.trait_key
            : null,
        connection_scope: workspace.resultRequest?.connection_scope ?? null,
      },
      afterClose: () => {
        const target = restoreToolbarFocus
          ? (linksTrigger.current ?? advancedTrigger.current)
          : opener;
        if (target?.isConnected) target.focus({ preventScroll: true });
      },
      onCancel: () =>
        setPicker((current) => (current ? { ...current, open: false } : null)),
      onConfirm: (relationship, selection, excluded) => {
        const currentGeneration = latestContext.current.generation;
        if (
          selection &&
          currentGeneration &&
          selection.artifact_generation !== currentGeneration
        )
          return "The reference collection changed. Refresh results before changing this condition.";
        if (
          advanced &&
          JSON.stringify(latestEditor.current.query) !== JSON.stringify(openingQuery)
        )
          return "Filters changed. Reopen this condition before applying changes.";
        if (
          !advanced &&
          initial &&
          !chipLinkClauses(latestContext.current.filter).some(
            (leaf) =>
              leaf.clause_id === id && sameRelationship(leaf.relationship, initial),
          )
        )
          return "This condition changed. Reopen it before applying changes.";
        const error = confirm(relationship, excluded);
        if (error) return error;
        restoreToolbarFocus = !advanced && !!initial && excluded;
        if (selection)
          setDraftLabels((current) => ({
            ...current,
            [id]: {
              relationship,
              title: selection.record.title,
              generation: selection.artifact_generation,
            },
          }));
        setPicker((current) => (current ? { ...current, open: false } : null));
        return null;
      },
    });
  }
  const linkEditing = {
    title: linkTitle,
    open: (
      rule: PredicateRule,
      replace: (next: PredicateRule) => void,
      opener?: HTMLElement,
    ) => {
      const initial = rule.value?.selection
        ? ({ ...rule.value, direction: rule.operator } as QueryRelationship)
        : undefined;
      const id = rule.id ?? crypto.randomUUID();
      openPicker(
        initial,
        id,
        (relationship, excluded) => {
          replace({
            ...rule,
            id,
            operator: relationship.direction,
            value: relationship,
            meta: { ...rule.meta, atlasExcluded: excluded },
          });
          return null;
        },
        {
          advanced: true,
          initialExcluded: !!rule.meta?.atlasExcluded,
          opener,
        },
      );
    },
  };
  const countWorkspace = {
    ...workspace,
    filterCountsEnabled:
      workspace.filterCountsEnabled !== false && search.mode !== "similar_records",
  };
  const advancedEditor =
    editor && prepared.query ? (
      <StructuredBuilder
        editor={editor}
        query={prepared.query}
        useApplicability={
          workspace.filterApplicabilityCurrent === true && !prepared.error
        }
        onChange={prepared.change}
        workspace={countWorkspace}
      />
    ) : null;
  const commonLayout = layout === "panel" ? "panel" : "toolbar";
  const inlineError =
    prepared.error && (search.filterError || !prepared.query) ? (
      <Typography.Text type="danger" role="status">
        {prepared.error}
      </Typography.Text>
    ) : null;
  const searchEntry = includeSearch && (
    <SearchEntry
      label={pane ? "Search this list" : undefined}
      placeholder={pane ? "Search this list…" : undefined}
      inputRef={searchInputRef}
      query={search.query}
      similar={similarSearch}
      onSearch={(query) =>
        setSearch(
          {
            ...search,
            query,
            mode: query.trim() ? "text_search" : "browse",
          },
          "submit",
        )
      }
    />
  );
  const commonFields = fields
    .filter((field) => editor?.common_field_ids.includes(field.definition.id))
    .map((field) => field.definition);
  const advancedApplied =
    editor && hasAdvancedFilterConditions(search.filter, commonFields);
  const canClear = Boolean(
    search.filter ||
    search.filterError ||
    prepared.error ||
    prepared.query?.rules.length,
  );
  function clear() {
    prepared.clear();
    setDraftRevision((revision) => revision + 1);
    setSearch(clearAllFilters(search));
    advancedTrigger.current?.focus();
  }
  const feedback = (
    <>
      {!advanced && prepared.error && inlineError}
      {workspace.filterDiscoveryError && (
        <ErrorNotice
          error={workspace.filterDiscoveryError}
          title="Filter information could not be loaded"
          description={workspace.filterDiscoveryError}
          action={
            pane &&
            workspace.refreshFilterInformation && (
              <Button onClick={() => void workspace.refreshFilterInformation?.()}>
                Retry filters
              </Button>
            )
          }
        />
      )}
    </>
  );
  const criteria = (
    <>
      <div className="filter-controls__tools">
        <div className="common-filters">
          {!editor && workspace.filterDiscoveryLoading && (
            <div
              className="filter-controls__pending"
              role="status"
              aria-label="Loading filters"
            >
              {[112, 144, 112, 112].map((width, index) => (
                <Skeleton.Button key={index} style={{ width }} aria-hidden="true" />
              ))}
            </div>
          )}
          {(editor?.common_field_ids ?? []).map((id) => {
            const field = fields.find((candidate) => candidate.definition.id === id);
            return field ? (
              <CommonFilter
                key={id + ":" + draftRevision}
                field={field}
                layout={commonLayout}
                compactPane={!!pane}
                selectionContainer={
                  commonLayout === "toolbar" ? selectionContainer : undefined
                }
                condition={
                  prepared.query
                    ? commonDraftCondition(prepared.query, field.definition)
                    : { status: "empty", clause: null }
                }
                range={
                  prepared.query
                    ? commonDraftRange(prepared.query, field.definition)
                    : null
                }
                onChange={(replacement) => {
                  if (prepared.query)
                    prepared.changeCommon(
                      replaceCommonDraft(prepared.query, field.definition, replacement),
                      field.definition,
                    );
                }}
                onRangeChange={(range) => {
                  if (prepared.query)
                    prepared.changeCommon(
                      replaceCommonRangeDraft(prepared.query, field.definition, range),
                      field.definition,
                    );
                }}
                workspace={countWorkspace}
                onAdvanced={() => setAdvanced(true)}
              />
            ) : null;
          })}
        </div>
        {includeLinks && (
          <Button
            ref={linksTrigger}
            onClick={() => {
              const limit = editor?.limits.relationship_leaves;
              if (limit !== undefined && countLinkConditions(search.filter) >= limit) {
                setAdvanced(true);
                return;
              }
              const id = crypto.randomUUID();
              openPicker(
                undefined,
                id,
                (relationship, excluded) =>
                  latestEditor.current.addLink(relationship, id, excluded),
                { opener: linksTrigger.current ?? undefined },
              );
            }}
          >
            Links <DownOutlined aria-hidden="true" />
          </Button>
        )}
        {additionalFilters}
        <div className="filter-panel-actions">
          <Tooltip
            title={
              advancedApplied && !advanced ? "Additional filters applied" : undefined
            }
            trigger={["hover", "focus"]}
          >
            <Button
              ref={advancedTrigger}
              className="filter-controls__advanced"
              data-applied={advancedApplied}
              color="default"
              variant="outlined"
              aria-description={
                advancedApplied ? "Additional filters applied" : undefined
              }
              icon={<FilterOutlined aria-hidden="true" />}
              onClick={() => setAdvanced(!advanced)}
              aria-expanded={advanced}
              aria-label="Advanced filters"
            >
              Advanced
            </Button>
          </Tooltip>
          {!pane && canClear && (
            <Tooltip title="Clear filters" trigger={["hover", "focus"]}>
              <Button
                icon={<ClearOutlined aria-hidden="true" />}
                aria-label="Clear filters"
                onClick={clear}
              />
            </Tooltip>
          )}
        </div>
      </div>
      <div
        className="filter-controls__selections"
        role="group"
        aria-label="Applied filters"
      >
        <div
          className="filter-controls__common-selections"
          ref={setSelectionContainer}
        />
        <LinkFilterControls
          filter={search.filter}
          title={linkTitle}
          limit={editor?.limits.relationship_leaves}
          onEdit={(leaf, opener) =>
            openPicker(
              leaf.relationship,
              leaf.clause_id!,
              (next, excluded) => latestEditor.current.editLink(leaf, next, excluded),
              { opener },
            )
          }
          onRemove={(leaf) => {
            const trigger = linksTrigger.current ?? advancedTrigger.current;
            restoreOverlayFocus(
              trigger,
              trigger?.ownerDocument.activeElement as HTMLElement | null,
            );
            latestEditor.current.editLink(leaf, null);
          }}
        />
      </div>
    </>
  );
  return (
    <LinkEditingContext value={linkEditing}>
      <aside
        className={
          pane
            ? "pane-filter-controls"
            : layout === "panel"
              ? "filter-panel"
              : "filter-toolbar"
        }
      >
        {children}
        {pane ? (
          <FilterDisclosure
            narrow={pane.narrow}
            searchEntry={searchEntry}
            summary={pane.summary}
            count={countAppliedFilterConditions(search.filter, commonFields)}
            childOverlayOpen={advanced || Boolean(picker?.open)}
            clear={clear}
            canClear={canClear}
          >
            {criteria}
            {feedback}
          </FilterDisclosure>
        ) : (
          <div className="filter-controls__chrome">
            {includeSearch && (
              <PrimaryToolbar className="filter-controls__search-row">
                {searchEntry}
                {searchRowEnd}
              </PrimaryToolbar>
            )}
            <div className="filter-controls__criteria">{criteria}</div>
          </div>
        )}
        <Drawer
          title="Advanced filters"
          rootClassName="advanced-filter-drawer-root"
          styles={{
            root: { overflow: "hidden", pointerEvents: advanced ? "auto" : undefined },
          }}
          drawerRender={(panel) => (
            <ConfigProvider getPopupContainer={getAdvancedPopupContainer}>
              {panel}
            </ConfigProvider>
          )}
          placement="left"
          push={false}
          size="min(100vw, 640px)"
          open={advanced}
          onClose={() => setAdvanced(false)}
        >
          <AdvancedFilterSurface label="Advanced filters">
            {prepared.error &&
              !search.filterError &&
              search.filter &&
              prepared.query &&
              hasConfiguredCondition(prepared.query) && (
                <Typography.Text type="secondary" role="status">
                  Changes pending
                </Typography.Text>
              )}
            {advancedEditor}
            {Picker && picker?.advanced && <LinkPicker {...picker} key={picker.key} />}
            {inlineError}
          </AdvancedFilterSurface>
        </Drawer>
        {!pane && feedback}
      </aside>
      {Picker && picker && !picker.advanced && (
        <LinkPicker {...picker} key={picker.key} />
      )}
    </LinkEditingContext>
  );
}

function CommonFilter({
  field,
  range,
  onRangeChange,
  condition,
  onChange,
  workspace,
  onAdvanced,
  layout,
  compactPane,
  selectionContainer,
}: {
  field: FilterEditorFieldView;
  selectionContainer?: HTMLElement | null;
  layout: "panel" | "toolbar";
  compactPane: boolean;
  condition: CommonCondition;
  range: [string, string] | null;
  onRangeChange: (range: [string, string]) => void;
  onChange: (replacement: QueryPredicate | null) => void;
  workspace: FilterPanelState;
  onAdvanced: () => void;
}) {
  const update = onChange;
  const rangeTrigger = useRef<HTMLButtonElement>(null);
  return (
    <section
      className={
        layout === "toolbar" ? "common-filter common-filter--toolbar" : "common-filter"
      }
      aria-label={field.label + " filter"}
    >
      {layout === "panel" && (
        <div className="common-filter-heading">
          <Typography.Text strong>{field.label}</Typography.Text>
          {!compactPane && condition.status === "editable" && (
            <Button
              type="text"
              size="small"
              aria-label={"Clear " + field.label}
              onClick={() => update(null)}
            >
              Clear
            </Button>
          )}
        </div>
      )}
      {condition.status === "advanced" && !range ? (
        <Button
          onClick={onAdvanced}
          aria-label={`Edit ${field.label} in advanced filters`}
        >
          {field.label}
        </Button>
      ) : field.definition.field_type === "number" ? (
        layout === "panel" ? (
          <CommonRange
            field={field}
            range={range ?? ["", ""]}
            onChange={onRangeChange}
            compact={compactPane}
          />
        ) : (
          <Popover
            trigger="click"
            placement="bottom"
            styles={{ root: { width: "min(360px, calc(100vw - 32px))" } }}
            title={field.label}
            content={
              <Space orientation="vertical" style={{ width: "100%" }}>
                <CommonRange
                  field={field}
                  range={range ?? ["", ""]}
                  onChange={onRangeChange}
                />
                <Button
                  type="text"
                  onClick={() => {
                    onRangeChange(["", ""]);
                  }}
                >
                  Clear range
                </Button>
              </Space>
            }
          >
            <Button
              ref={rangeTrigger}
              aria-label={
                "Edit " + field.label + " range: " + rangeSummary(range, "words")
              }
            >
              {field.label}
            </Button>
          </Popover>
        )
      ) : (
        <CommonChoicePicker
          selectionContainer={selectionContainer}
          field={field}
          buttonText={compactPane ? (condition.clause ? "Change…" : "Any") : undefined}
          clause={condition.clause}
          scope={workspace.vocabularyScope ?? { kind: "artifact" }}
          countContext={workspace.filterDiscoveryContext}
          countsEnabled={workspace.filterCountsEnabled !== false}
          countQueryKey={workspace.filterCountQueryKey}
          countLimit={workspace.filterEditor?.limits.literal_list ?? 0}
          onChange={update}
        />
      )}
      {layout === "toolbar" &&
        range?.some((bound) => bound.trim()) &&
        selectionContainer &&
        createPortal(
          <Button
            className="applied-filter-chip"
            size="small"
            color="default"
            variant="filled"
            aria-label={`Clear ${field.label} range`}
            onClick={(event) => {
              restoreOverlayFocus(rangeTrigger.current, event.currentTarget);
              onRangeChange(["", ""]);
            }}
          >
            {field.label}: {rangeSummary(range)}
            <CloseOutlined aria-hidden="true" />
          </Button>,
          selectionContainer,
        )}
    </section>
  );
}

function rangeSummary(
  range: [string, string] | null,
  format: "symbols" | "words" = "symbols",
): string {
  const [min, max] = range?.map((value) => value.trim()) ?? ["", ""];
  if (!min && !max) return "Any";
  if (!min) return format === "symbols" ? `≤ ${max}` : `${max} or lower`;
  if (!max) return format === "symbols" ? `≥ ${min}` : `${min} or higher`;
  try {
    if (numericLiteral(min) === numericLiteral(max))
      return format === "symbols" ? min : `Exactly ${min}`;
  } catch {
    // Invalid drafts remain visible while the range editor reports the error.
  }
  return format === "symbols" ? `${min}–${max}` : `${min} to ${max}`;
}

function CommonRange({
  field,
  range,
  onChange,
  compact = false,
}: {
  field: FilterEditorFieldView;
  range: [string, string];
  compact?: boolean;
  onChange: (range: [string, string]) => void;
}) {
  const id = useId();
  const [editingRange, setEditingRange] = useState(false);
  let error: string | null = null;
  try {
    const min = range[0].trim() ? numericLiteral(range[0]) : null;
    const max = range[1].trim() ? numericLiteral(range[1]) : null;
    if (min !== null && max !== null && min > max)
      throw new Error("Minimum exceeds maximum");
  } catch (failure) {
    error = failure instanceof Error ? failure.message : String(failure);
  }
  const message = editingRange ? null : error;
  return (
    <div
      onFocus={() => setEditingRange(true)}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) setEditingRange(false);
      }}
    >
      <div className="filter-range">
        <label className="filter-range__bound" htmlFor={id + "-minimum"}>
          {!compact && <span>Minimum</span>}
          <Input
            id={id + "-minimum"}
            prefix={
              compact ? (
                <Typography.Text type="secondary">Min</Typography.Text>
              ) : undefined
            }
            aria-label={field.label + " minimum"}
            aria-invalid={message ? true : undefined}
            aria-describedby={message ? id + "-error" : undefined}
            name={field.definition.id + "-minimum"}
            inputMode="decimal"
            autoComplete="off"
            placeholder="Any"
            status={message ? "error" : undefined}
            value={range[0]}
            onChange={(event) => onChange([event.target.value, range[1]])}
          />
        </label>
        <label className="filter-range__bound" htmlFor={id + "-maximum"}>
          {!compact && <span>Maximum</span>}
          <Input
            id={id + "-maximum"}
            prefix={
              compact ? (
                <Typography.Text type="secondary">Max</Typography.Text>
              ) : undefined
            }
            aria-label={field.label + " maximum"}
            aria-invalid={message ? true : undefined}
            aria-describedby={message ? id + "-error" : undefined}
            name={field.definition.id + "-maximum"}
            inputMode="decimal"
            autoComplete="off"
            placeholder="Any"
            status={message ? "error" : undefined}
            value={range[1]}
            onChange={(event) => onChange([range[0], event.target.value])}
          />
        </label>
      </div>
      {error && (
        <Typography.Text
          type="danger"
          role={message ? "status" : undefined}
          aria-hidden={message ? undefined : true}
          id={id + "-error"}
          className={message ? undefined : "filter-validation-deferred"}
        >
          {error}
        </Typography.Text>
      )}
    </div>
  );
}

function AdvancedFilterSurface({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  const { token } = theme.useToken();
  return (
    <section
      className="advanced-filters"
      aria-label={label}
      style={
        {
          "--filter-action-hover": token.colorFillTertiary,
          "--filter-action-motion": `${token.motionDurationMid} ${token.motionEaseInOut}`,
        } as CSSProperties
      }
    >
      {children}
    </section>
  );
}

type BuilderContext = {
  editor: FilterEditorView;
  query: RuleGroupType;
  scope: string | null;
  narrowFields: boolean;
  relatedTarget: boolean;
  workspace: FilterPanelState;
  addCondition: (path: number[], fieldId: string) => void;
  pendingConditionRef: RefObject<string | null>;
  replaceGroup: (path: number[], group: RuleGroupType) => void;
  replaceRule: (path: number[], rule: RuleType, normalize?: boolean) => void;
};
function StructuredBuilder({
  editor,
  query,
  onChange,
  scope = null,
  useApplicability = false,
  relatedTarget = false,
  workspace,
}: {
  editor: FilterEditorView;
  query: RuleGroupType;
  onChange: (query: RuleGroupType) => void;
  scope?: string | null;
  useApplicability?: boolean;
  relatedTarget?: boolean;
  workspace: FilterPanelState;
}) {
  const pendingConditionRef = useRef<string | null>(null);
  const visible = editor.groups
    .flatMap((group) => group.fields)
    .filter(
      (field) =>
        field.definition.scope === scope &&
        (scope === null || field.definition.field_type !== "collection"),
    );
  return (
    <QueryBuilderAntD>
      <QueryBuilder
        query={query}
        onQueryChange={(next) =>
          onChange(
            normalizeBuilderEdit(
              query,
              next,
              editor.groups.flatMap((group) => group.fields),
              scope,
            ),
          )
        }
        autoSelectField={false}
        enableMountQueryChange={false}
        fields={[
          ...(scope === null && !relatedTarget
            ? [
                {
                  label: "Related entries",
                  options: [
                    { name: RELATED_CONDITION, label: "Related entries" },
                    { name: LINK_CONDITION, label: "Record links" },
                  ],
                },
              ]
            : []),
          ...editor.groups
            .map((group) => ({
              label: group.label,
              options: visible
                .filter((field) => group.fields.includes(field))
                .map((field) => ({
                  name: field.definition.id,
                  label:
                    field.label +
                    (field.definition.units ? " (" + field.definition.units + ")" : ""),
                })),
            }))
            .filter((group) => group.options.length),
        ]}
        combinators={[
          { name: "and", label: "All conditions match" },
          { name: "or", label: "At least one condition matches" },
        ]}
        getOperators={(id) =>
          id === LINK_CONDITION || id === RELATED_CONDITION
            ? [
                { name: "incoming", label: "Links to" },
                { name: "outgoing", label: "Linked from" },
              ]
            : id === BOOLEAN_CONDITION
              ? [{ name: "constant", label: "Matches" }]
              : (
                  visible.find((field) => field.definition.id === id)?.operators ?? []
                ).map((operator) => ({
                  name: operator.operator,
                  label: operator.label,
                }))
        }
        context={
          {
            editor,
            query,
            scope,
            narrowFields: useApplicability && isPositiveConjunction(query, visible),
            relatedTarget,
            workspace,
            pendingConditionRef,
            addCondition: (path, fieldId) => {
              const operator = visible
                .find((field) => field.definition.id === fieldId)
                ?.operators.find((option) => option.operator !== "state");
              const id = crypto.randomUUID();
              pendingConditionRef.current = id;
              onChange(
                add(
                  query,
                  {
                    id,
                    field: fieldId,
                    operator:
                      fieldId === LINK_CONDITION || fieldId === RELATED_CONDITION
                        ? "incoming"
                        : (operator?.operator ?? "eq"),
                    value: operator
                      ? transitionOperand(null, operator.operand, null)
                      : "",
                  },
                  path,
                ),
              );
            },
            replaceGroup: (path, group) =>
              onChange(
                update(query, { combinator: group.combinator, not: group.not }, path),
              ),
            replaceRule: (path, rule, normalize = true) => {
              const next = update(
                query,
                {
                  field: rule.field,
                  operator: rule.operator,
                  value: rule.value,
                  meta: rule.meta,
                },
                path,
              );
              onChange(
                normalize
                  ? normalizeBuilderEdit(
                      query,
                      next,
                      editor.groups.flatMap((group) => group.fields),
                      scope,
                    )
                  : next,
              );
            },
          } satisfies BuilderContext
        }
        translations={{
          addRule: { label: "Add condition", title: "Add condition" },
          addGroup: { label: "Add group", title: "Add group" },
          removeRule: { label: "Remove", title: "Remove condition" },
          removeGroup: { label: "Remove group", title: "Remove group" },
        }}
        controlElements={{
          removeRuleAction: RemoveCondition,
          addRuleAction: HiddenGroupAction,
          addGroupAction: HiddenGroupAction,
          ruleGroupBodyElements: FilterGroupBody,
          removeGroupAction: RemoveCondition,
          rule: FilterRule,
          valueEditor: BuilderValueEditor,
          fieldSelector: BuilderFieldSelector,
          operatorSelector: BuilderOperatorSelector,
          combinatorSelector: GroupMatchSelector,
        }}
      />
    </QueryBuilderAntD>
  );
}
const RuleEditContext = createContext<{
  rule: PredicateRule;
  replace: (rule: PredicateRule, normalize?: boolean) => void;
} | null>(null);
function FilterRule(props: RuleProps) {
  const rule = useRule(props);
  const { editor, replaceRule, pendingConditionRef, scope } =
    props.context as BuilderContext;
  const field = editor.groups
    .flatMap((group) => group.fields)
    .find((candidate) => candidate.definition.id === props.rule.field);
  const row = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    if (pendingConditionRef.current !== props.rule.id || !row.current) return;
    pendingConditionRef.current = null;
    const focusable = "input:not([disabled]), button:not([disabled])";
    const next =
      row.current
        .querySelector(
          ":scope > .filter-rule-value, :scope > .filter-collection-condition",
        )
        ?.querySelector<HTMLElement>(focusable) ??
      row.current.querySelector<HTMLElement>(
        ".filter-field-family > :last-child input",
      ) ??
      row.current.querySelector<HTMLElement>(focusable);
    next?.focus();
  }, [pendingConditionRef, props.rule.id]);
  const links = useContext(LinkEditingContext);
  const draftRule = props.rule as PredicateRule;
  const relatedEntryRule =
    draftRule.field === RELATED_CONDITION ||
    (draftRule.operator === "exists" && isEmbeddedEntryField(field));
  const relationship =
    draftRule.field === LINK_CONDITION && draftRule.value?.selection
      ? ({ ...draftRule.value, direction: draftRule.operator } as QueryRelationship)
      : null;
  const title =
    relationship &&
    links?.title({ kind: "relationship", clause_id: draftRule.id, relationship });
  const direction =
    relationship?.direction === "incoming"
      ? draftRule.meta?.atlasExcluded
        ? "Does not link to"
        : "Links to"
      : draftRule.meta?.atlasExcluded
        ? "Not linked from"
        : "Linked from";
  return (
    <RuleEditContext.Provider
      value={{
        rule: props.rule as PredicateRule,
        replace: (next, normalize) => replaceRule(props.path, next, normalize),
      }}
    >
      <div
        ref={row}
        className={rule.outerClassName}
        data-rule-id={props.rule.id}
        data-filter-layout={
          draftRule.field === LINK_CONDITION || relatedEntryRule
            ? "relationship"
            : scope === null && actorMetricKind(draftRule.field)
              ? "grouped"
              : field && (contextualFieldLabel(field) ?? field.label).length > 24
                ? "wide-label"
                : undefined
        }
      >
        <RuleComponents {...rule} />
        {relatedEntryRule && (
          <RelatedEntrySummary
            rule={draftRule}
            fields={editor.groups.flatMap((group) => group.fields)}
          />
        )}
        {relationship && (
          <Typography.Text type="secondary" className="filter-link-summary">
            {direction} {title ?? "Record unavailable"} ·{" "}
            {relationship.include_contents ? "Whole entry" : "Main entry only"}
          </Typography.Text>
        )}
      </div>
    </RuleEditContext.Provider>
  );
}
// Current-result applicability is safe for additive conditions. Boolean groups
// that can broaden the result need the complete catalog while being edited.
function isPositiveConjunction(
  group: RuleGroupType,
  fields: FilterEditorFieldView[],
): boolean {
  return (
    group.combinator === "and" &&
    !group.not &&
    group.rules.every((rule) => {
      if ("rules" in rule) return isPositiveConjunction(rule, fields);
      if (!(rule as PredicateRule).meta?.atlasExcluded) return true;
      const field = fields.find((field) => field.definition.id === rule.field);
      return !!field && isClearedOperand(rule, field.definition);
    })
  );
}
function isAdditivePath(group: RuleGroupType, path: number[]): boolean {
  if (group.combinator !== "and" || group.not) return false;
  if (path.length <= 1) return true;
  const [index, ...rest] = path;
  const child = group.rules[index];
  return !!child && "rules" in child && isAdditivePath(child, rest);
}
function contextualFieldLabel(field: FilterEditorFieldView) {
  switch (field.definition.id) {
    case "campaign_feature.category":
      return "Feature category";
    case "source.document_kind":
      return "Document category";
    default:
      return null;
  }
}
type FilterFieldOption = {
  value: string;
  label: string;
  searchText: string;
  rowLabel?: string;
};
function filterFieldGroups(
  {
    editor,
    scope,
    narrowFields,
    relatedTarget,
  }: Pick<BuilderContext, "editor" | "scope" | "narrowFields" | "relatedTarget">,
  addingCondition: boolean,
  currentField?: string,
): {
  label: string;
  options: FilterFieldOption[];
}[] {
  const seenKinds = new Set<ActorMetricKind>();
  return [
    ...editor.groups
      .map((group) => ({
        label: group.label,
        options: group.fields
          .filter(
            (field) =>
              field.definition.scope === scope &&
              !isEmbeddedEntryField(field) &&
              (field.definition.id === currentField ||
                !relatedTarget ||
                scope === null ||
                field.definition.path !== "kind" ||
                field.definition.choices.length !== 1) &&
              (scope === null || field.definition.field_type !== "collection") &&
              (!addingCondition || !narrowFields || field.applicable) &&
              (!field.unit_fields.length ||
                field.unit_fields[0].field_id === field.definition.id),
          )
          .flatMap<FilterFieldOption>((field) => {
            const kind = scope === null ? actorMetricKind(field.definition.id) : null;
            if (kind) {
              if (seenKinds.has(kind)) return [];
              seenKinds.add(kind);
              const label = kind === "ability" ? "Ability" : "Skill";
              return [
                {
                  value: ACTOR_METRIC_CHOOSERS[kind],
                  label,
                  searchText:
                    label +
                    " " +
                    group.fields
                      .filter(
                        (candidate) =>
                          actorMetricKind(candidate.definition.id) === kind,
                      )
                      .map((candidate) => candidate.label)
                      .join(" "),
                },
              ];
            }
            return [
              {
                value: field.definition.id,
                label: contextualFieldLabel(field) ? "Category" : field.label,
                rowLabel: contextualFieldLabel(field) ?? field.label,
                searchText: field.label,
              },
            ];
          }),
      }))
      .filter((group) => group.options.length),
    ...(scope === null && !relatedTarget
      ? [
          {
            label: "Relationships",
            options: [
              {
                value: RELATED_CONDITION,
                label: "Related entries",
                searchText:
                  "Related entries contains abilities spells equipment links to linked from",
              },
              ...(currentField === LINK_CONDITION
                ? [
                    {
                      value: LINK_CONDITION,
                      label: "Record links",
                      searchText: "Record links",
                    },
                  ]
                : []),
            ],
          },
        ]
      : []),
    ...(currentField === BOOLEAN_CONDITION
      ? [
          {
            label: "Technical condition",
            options: [
              {
                value: BOOLEAN_CONDITION,
                label: "Constant condition",
                searchText: "Constant condition",
              },
            ],
          },
        ]
      : []),
  ];
}
function BuilderFieldSelector(props: FieldSelectorProps) {
  const { editor, scope, narrowFields, relatedTarget } =
    props.context as BuilderContext;
  const editing = useContext(RuleEditContext);
  const selectedKind = scope === null ? actorMetricKind(props.value ?? "~") : null;
  const addingCondition = !props.value || props.value === "~";
  const groups = filterFieldGroups(
    { editor, scope, narrowFields, relatedTarget },
    addingCondition,
    props.value,
  );
  const related = editor.groups.some((group) =>
    group.fields.some(
      (field) => field.definition.id === props.value && isEmbeddedEntryField(field),
    ),
  );
  const chooser = (
    <FilterFieldSelect
      className={selectedKind ? undefined : props.className}
      aria-label="Filter field"
      placeholder="Choose a filter…"
      styles={{
        content: { whiteSpace: "normal", overflowWrap: "anywhere" },
        popup: { listItem: { whiteSpace: "normal", overflowWrap: "anywhere" } },
      }}
      showSearch={{ optionFilterProp: "searchText" }}
      options={groups}
      labelRender={({ value, label }) => {
        const option = groups
          .flatMap((group) => group.options)
          .find((entry) => entry.value === value);
        return option?.rowLabel ?? label;
      }}
      value={
        related
          ? RELATED_CONDITION
          : selectedKind
            ? ACTOR_METRIC_CHOOSERS[selectedKind]
            : props.value === "~"
              ? undefined
              : (editor.groups
                  .flatMap((group) => group.fields)
                  .find((field) => field.definition.id === props.value)?.unit_fields[0]
                  ?.field_id ?? props.value)
      }
      onChange={(value) => {
        if (isActorMetricChooser(value) && editing) {
          if (actorMetricKind(value) === selectedKind) return;
          editing.replace({
            ...editing.rule,
            field: value,
            operator: "eq",
            value: "",
            meta: { ...editing.rule.meta, atlasExcluded: false },
          });
        } else props.handleOnChange(value);
      }}
      disabled={props.disabled}
    />
  );
  if (!selectedKind) return chooser;
  return (
    <div className={`${props.className ?? ""} filter-field-family`}>
      {chooser}
      <ActorMetricPicker
        kind={selectedKind}
        fields={editor.groups
          .flatMap((group) => group.fields)
          .filter((field) => field.definition.scope === scope)}
        value={props.value ?? "~"}
        disabled={props.disabled}
        onChange={(field) =>
          editing?.replace({
            ...editing.rule,
            field,
            operator: editing.rule.operator || "eq",
          })
        }
      />
    </div>
  );
}
function RemoveCondition(props: ActionProps) {
  return (
    <Tooltip title={props.title || "Remove condition"} trigger={["hover", "focus"]}>
      <Button
        className={props.className}
        data-filter-remove
        type="text"
        icon={<CloseOutlined />}
        aria-label={props.title || "Remove condition"}
        disabled={props.disabled}
        onClick={(event) => {
          const control = event.currentTarget;
          const item = control.closest(".rule") ?? control.closest(".ruleGroup");
          const group = item?.parentElement?.closest(".ruleGroup");
          const addControl = group?.querySelector<HTMLElement>(
            ":scope > .ruleGroup-body > .filter-group-actions [role=combobox]",
          );
          restoreOverlayFocus(addControl ?? null, control);
          props.handleOnClick(event);
        }}
      />
    </Tooltip>
  );
}
function GroupMatchSelector(props: CombinatorSelectorProps) {
  const { replaceGroup } = props.context as BuilderContext;
  const group = props.ruleGroup as RuleGroupType;
  return (
    <Select
      className="filter-text-select filter-match-mode"
      aria-label="Match conditions"
      variant="borderless"
      styles={{
        root: { maxWidth: "100%" },
        content: { whiteSpace: "normal" },
        popup: { listItem: { whiteSpace: "normal" } },
      }}
      popupMatchSelectWidth={false}
      value={(group.not ? "not:" : "") + group.combinator}
      options={[
        { value: "and", label: "All conditions match" },
        { value: "or", label: "At least one condition matches" },
        { value: "not:or", label: "No conditions match" },
        { value: "not:and", label: "At least one condition doesn’t match" },
      ]}
      onChange={(value) =>
        replaceGroup(props.path, {
          ...group,
          combinator: value.replace("not:", ""),
          not: value.startsWith("not:"),
        })
      }
    />
  );
}
function HiddenGroupAction() {
  return null;
}
function hasConfiguredCondition(group: RuleGroupType): boolean {
  return group.rules.some((rule) =>
    "rules" in rule ? hasConfiguredCondition(rule) : rule.field !== "~",
  );
}
function FilterGroupBody(props: UseRuleGroup) {
  const context = props.context as BuilderContext;
  const { token } = theme.useToken();
  const [choosingField, setChoosingField] = useState(false);
  return (
    <>
      <RuleGroupBodyComponents {...props} />
      <Space className="filter-group-actions">
        <FilterFieldSelect
          className="filter-text-select filter-add-condition"
          aria-label="Add condition"
          placeholder="Add condition"
          variant="borderless"
          styles={{
            placeholder: { color: token.colorText },
            popup: { listItem: { whiteSpace: "normal", overflowWrap: "anywhere" } },
          }}
          value={null}
          open={choosingField}
          onOpenChange={setChoosingField}
          showSearch={{ optionFilterProp: "searchText" }}
          options={filterFieldGroups(context, true)}
          disabled={props.disabled}
          onChange={(fieldId) => {
            setChoosingField(false);
            if (fieldId) context.addCondition(props.path, fieldId);
          }}
        />
        {props.schema.maxLevels > props.path.length && (
          <Button type="text" disabled={props.disabled} onClick={props.addGroup}>
            Add group
          </Button>
        )}
      </Space>
    </>
  );
}
function levelBounds(rule: PredicateRule): [string, string] | null {
  if (
    (rule.field !== "level" && rule.field !== "level_or_rank") ||
    rule.meta?.atlasExcluded
  )
    return null;
  const text = (value: unknown) =>
    typeof value === "string" || typeof value === "number" ? String(value) : null;
  const { operator, value } = rule;
  if (operator === "between" && Array.isArray(value) && value.length === 2) {
    const min = text(value[0]);
    const max = text(value[1]);
    return min !== null && max !== null ? [min, max] : null;
  }
  const bound = text(value);
  if (bound === null) return null;
  if (operator === "eq") return [bound, bound];
  if (operator === "gte") return [bound, ""];
  if (operator === "lte") return ["", bound];
  // Indexed levels can be fractional, so strict comparisons need their operator.
  return null;
}
function booleanComparison(
  field: FilterEditorFieldView | undefined,
  rule: PredicateRule,
) {
  return (
    field?.definition.field_type === "boolean" &&
    field.operators.some(({ operator }) => operator === "eq") &&
    (rule.operator === "eq" || rule.operator === "neq") &&
    (rule.value === "" ||
      rule.value == null ||
      ["true", "false"].includes(String(rule.value)))
  );
}
function BuilderOperatorSelector(props: OperatorSelectorProps) {
  const editing = useContext(RuleEditContext);
  if (isActorMetricChooser(props.field)) return null;
  if (props.field === LINK_CONDITION || props.field === RELATED_CONDITION) return null;
  const { editor } = props.context as BuilderContext;
  const field = editor.groups
    .flatMap((group) => group.fields)
    .find((field) => field.definition.id === props.field);
  if (isEmbeddedEntryField(field) && props.value === "exists") return null;
  if (editing && (levelBounds(editing.rule) || booleanComparison(field, editing.rule)))
    return null;
  const excluded = !!editing?.rule.meta?.atlasExcluded;
  const operators =
    field?.operators.filter(
      (operator) => operator.operator !== "state" || props.value === "state",
    ) ?? [];
  const options = operators.map((operator) => ({
    value: operator.operator,
    label:
      operator.operator === "exists"
        ? "has"
        : operator.operator === "excludes_any"
          ? "includes none"
          : operator.label,
  }));
  for (const operator of operators) {
    if (
      ["in", "exists"].includes(operator.operator) ||
      (excluded && operator.operator === props.value)
    ) {
      options.push({
        value: "not:" + operator.operator,
        label:
          operator.operator === "exists"
            ? "has no"
            : operator.operator === "in"
              ? "is none of"
              : operator.operator === "eq"
                ? "is not"
                : operator.operator === "neq"
                  ? "is"
                  : operator.operator === "includes"
                    ? "excludes"
                    : operator.operator === "includes_all"
                      ? "doesn’t include all"
                      : "not " + operator.label,
      });
    }
  }
  return (
    <CompactSelect
      className={props.className}
      aria-label={"Condition for " + (field?.label ?? "this field")}
      popupMatchSelectWidth={false}
      value={excluded ? "not:" + props.value : props.value}
      options={
        props.field === BOOLEAN_CONDITION
          ? [{ value: "constant", label: "Matches" }]
          : options
      }
      onChange={(next) => {
        if (!editing) return props.handleOnChange(next);
        const negate = next.startsWith("not:");
        editing.replace({
          ...editing.rule,
          operator: negate ? next.slice(4) : next,
          meta: { ...editing.rule.meta, atlasExcluded: negate },
        });
      }}
      disabled={props.disabled}
    />
  );
}
function countOtherLinkRules(group: RuleGroupType, ruleId: string | undefined): number {
  return group.rules.reduce((count, node) => {
    if ("rules" in node) return count + countOtherLinkRules(node, ruleId);
    if (node.id === ruleId) return count;
    if (node.field === LINK_CONDITION && node.value?.selection) return count + 1;
    if (node.field === RELATED_CONDITION && node.value?.query) return count + 1;
    return count;
  }, 0);
}
function BuilderValueEditor(props: ValueEditorProps) {
  const editing = useContext(RuleEditContext);
  const { editor, query, scope, workspace } = props.context as BuilderContext;
  const [editingRange, setEditingRange] = useState(false);
  const links = useContext(LinkEditingContext);
  const fields = editor.groups.flatMap((group) => group.fields);
  const embedded = fields.find(
    (field) => field.definition.id === props.field && isEmbeddedEntryField(field),
  );
  if (editing && props.field === LINK_CONDITION) {
    return (
      <div className="filter-rule-value">
        <Button
          disabled={props.disabled}
          onClick={(event) =>
            links?.open(
              editing.rule,
              (replacement) => editing.replace(replacement, false),
              event.currentTarget,
            )
          }
        >
          Edit link
        </Button>
      </div>
    );
  }
  if (
    editing &&
    scope === null &&
    ((embedded && props.operator === "exists") || props.field === RELATED_CONDITION)
  ) {
    return (
      <RelatedEntryEditor
        rule={editing.rule}
        fields={fields}
        linkLimitReached={
          countOtherLinkRules(query, editing.rule.id) >=
          editor.limits.relationship_leaves
        }
        disabled={props.disabled}
        onApply={(next) => editing.replace(next, false)}
        renderConditions={(draft, change, memberScope) => (
          <AdvancedFilterSurface label="Related entry filters">
            <StructuredBuilder
              editor={editor}
              query={draft}
              onChange={change}
              scope={memberScope}
              relatedTarget
              workspace={{
                ...workspace,
                filterCountsEnabled: false,
                vocabularyScope: { kind: "artifact" },
              }}
            />
          </AdvancedFilterSurface>
        )}
      />
    );
  }

  if (props.field === BOOLEAN_CONDITION)
    return (
      <Select
        className="filter-rule-value"
        aria-label="Structural condition"
        virtual={false}
        value={String(props.value)}
        options={[
          { value: "true", label: "All records" },
          { value: "false", label: "No records" },
        ]}
        onChange={props.handleOnChange}
      />
    );
  const field = fields.find(
    (field) => field.definition.id === props.field && field.definition.scope === scope,
  );
  if (!field)
    return props.field === "~" || isActorMetricChooser(props.field) ? null : (
      <Typography.Text type="danger" role="status" className="filter-rule-value">
        This field is unavailable. Choose another field or remove this condition.
      </Typography.Text>
    );
  if (editing) {
    const bounds = levelBounds(editing.rule);
    if (bounds)
      return (
        <div className="filter-rule-value">
          <CommonRange
            field={field}
            compact
            range={bounds}
            onChange={(next) => editing.replace(rangeRule(editing.rule, next), false)}
          />
        </div>
      );
    if (booleanComparison(field, editing.rule)) {
      const rule = editing.rule;
      const invert = (rule.operator === "neq") !== !!rule.meta?.atlasExcluded;
      const value =
        rule.value === "" || rule.value == null
          ? undefined
          : String((String(rule.value) === "true") !== invert);
      return (
        <CompactSelect
          className="filter-rule-value"
          aria-label={field.label}
          placeholder="—"
          allowClear
          options={[
            { label: "Yes", value: "true" },
            { label: "No", value: "false" },
          ]}
          value={value}
          disabled={props.disabled}
          onChange={(next) =>
            editing.replace(
              {
                ...rule,
                operator: "eq",
                value: next ?? "",
                meta: { ...rule.meta, atlasExcluded: false },
              },
              false,
            )
          }
        />
      );
    }
  }
  if (props.operator === "exists") {
    const query = isBuilderGroup(props.value) ? props.value : emptyGroup();
    if (isCompactCollection(field, query))
      return (
        <div className="filter-collection-condition">
          <CollectionValueInput
            field={field}
            editor={editor}
            query={query}
            onChange={props.handleOnChange}
            workspace={workspace}
          />
        </div>
      );
    return (
      <div className="filter-collection-condition">
        <StructuredBuilder
          editor={editor}
          query={isBuilderGroup(props.value) ? props.value : emptyGroup()}
          onChange={props.handleOnChange}
          scope={field.definition.path}
          workspace={workspace}
        />
      </div>
    );
  }
  let error: string | null = null;
  try {
    builderPredicate(
      { combinator: "and", rules: [props.rule] },
      [field.definition],
      scope,
    );
  } catch (reason) {
    error = reason instanceof Error ? reason.message : String(reason);
  }
  const incomplete =
    props.value == null ||
    props.value === "" ||
    (Array.isArray(props.value) && !props.value.length);
  const isRange = props.operator === "between";
  const range = isRange && Array.isArray(props.value) ? props.value : [];
  const hasBound = (value: unknown) => value != null && String(value).trim() !== "";
  const enteredBounds = range.filter(hasBound).length;
  const issue = isRange
    ? enteredBounds === 0
      ? null
      : enteredBounds === 1
        ? "Enter minimum and maximum"
        : error
    : incomplete
      ? null
      : error;
  const message = isRange && editingRange ? null : issue;
  const clauseId = props.rule.id ?? null;
  // An unaddressed facet evaluates an added conjunction. That cannot describe
  // inserting a condition beneath OR or group negation.
  const countsEnabled =
    workspace.filterCountsEnabled !== false &&
    (isAdditivePath(query, props.path) ||
      !!existingClauseId(workspace.filterDiscoveryContext, clauseId));
  return (
    <div
      className="filter-rule-value"
      onFocus={isRange ? () => setEditingRange(true) : undefined}
      onBlur={
        isRange
          ? (event) => {
              if (!event.currentTarget.contains(event.relatedTarget))
                setEditingRange(false);
            }
          : undefined
      }
    >
      <Form.Item
        style={{ marginBottom: 0 }}
        validateStatus={message ? "error" : undefined}
        help={
          issue ? (
            <span
              role={message ? "status" : undefined}
              aria-hidden={message ? undefined : true}
              className={message ? undefined : "filter-validation-deferred"}
            >
              {issue}
            </span>
          ) : undefined
        }
      >
        <div className={field.unit_fields.length ? "filter-duration-value" : undefined}>
          <FilterValueInput
            excluded={!!props.rule.meta?.atlasExcluded}
            field={field}
            operator={props.operator}
            value={props.value}
            onChange={props.handleOnChange}
            scope={workspace.vocabularyScope ?? { kind: "artifact" }}
            countContext={workspace.filterDiscoveryContext}
            countsEnabled={countsEnabled}
            countQueryKey={workspace.filterCountQueryKey}
            countLimit={workspace.filterEditor?.limits.literal_list ?? 0}
            clauseId={clauseId}
          />
          {field.unit_fields.length > 0 && (
            <Select
              aria-label="Duration unit"
              value={field.definition.id}
              options={field.unit_fields.map((option) => ({
                value: option.field_id,
                label: option.unit,
              }))}
              onChange={(id) =>
                editing && editing.replace({ ...editing.rule, field: id })
              }
            />
          )}
        </div>
      </Form.Item>
    </div>
  );
}
