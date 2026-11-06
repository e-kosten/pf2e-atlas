import { restoreOverlayFocus } from "../ui/overlays/restoreOverlayFocus";
import { createPortal } from "react-dom";
import { AtlasDrawer as Drawer } from "../ui/overlays/AtlasDrawer";
import { ErrorNotice } from "../feedback/ErrorNotice";
import {
  Button,
  Badge,
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
  PlusOutlined,
} from "@ant-design/icons";
import {
  createContext,
  useContext,
  useId,
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
  emptyGroup,
  isClearedOperand,
  numericLiteral,
  replaceCommonDraft,
  replaceCommonRangeDraft,
  type PredicateRule,
} from "./queryBuilder";
import {
  clearAllFilters,
  hasAdvancedFilterConditions,
  type CommonCondition,
  type FilterPanelState,
} from "./filterControls";
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
import { LinkFilterControls, linkClauses, type LinkLeaf } from "./LinkFilterControls";
import { sameRelationship } from "./linkPredicates";
import { CollectionValueInput, isCompactCollection } from "./CollectionValueInput";
import { ActorMetricPicker } from "./ActorMetricPicker";
import {
  ACTOR_METRIC_CHOOSERS,
  actorMetricKind,
  isActorMetricChooser,
  type ActorMetricKind,
} from "./actorMetricFields";

const LinkEditingContext = createContext<{
  open: (rule: PredicateRule, replace: (next: PredicateRule) => void) => void;
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
  layout = "panel",
  searchRowEnd,
  searchInputRef,
  similarSearch,
  additionalFilters,
  children,
}: {
  filterState: FilterPanelState;
  includeSearch?: boolean;
  layout?: "panel" | "toolbar";
  searchRowEnd?: ReactNode;
  searchInputRef?: RefObject<InputRef | null>;
  similarSearch?: SimilarSearchCriterion;
  additionalFilters?: ReactNode;
  children?: ReactNode;
}) {
  const { search, setSearch } = workspace;
  const { token: filterTokens } = theme.useToken();
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
              ? ` · ${presentation.record.title}`
              : "")
        : "Record unavailable";
    return workspace.filterDiscoveryError ? "Record unavailable" : "Loading record…";
  }
  function openPicker(
    initial: QueryRelationship | undefined,
    title: string | undefined,
    id: string,
    confirm: (relationship: QueryRelationship) => string | null,
    advanced = false,
  ) {
    if (!Picker) throw new Error("Link condition picker provider is required");
    const openingQuery = prepared.query;
    setPicker({
      key: crypto.randomUUID(),
      advanced,
      open: true,
      initial,
      initialTitle: title,
      onCancel: () =>
        setPicker((current) => (current ? { ...current, open: false } : null)),
      onConfirm: (relationship, selection) => {
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
          !linkClauses(latestContext.current.filter).some(
            (leaf) =>
              leaf.clause_id === id && sameRelationship(leaf.relationship, initial),
          )
        )
          return "This condition changed. Reopen it before applying changes.";
        const error = confirm(relationship);
        if (error) return error;
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
    open: (rule: PredicateRule, replace: (next: PredicateRule) => void) => {
      const initial = rule.value?.selection
        ? ({ ...rule.value, direction: rule.operator } as QueryRelationship)
        : undefined;
      const id = rule.id ?? crypto.randomUUID();
      openPicker(
        initial,
        initial
          ? linkTitle({ kind: "relationship", clause_id: id, relationship: initial })
          : undefined,
        id,
        (relationship) => {
          if (initial && sameRelationship(initial, relationship)) return null;
          replace({
            ...rule,
            id,
            operator: relationship.direction,
            value: relationship,
          });
          return null;
        },
        true,
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
  const advancedApplied =
    editor &&
    hasAdvancedFilterConditions(
      search.filter,
      fields
        .filter((field) => editor.common_field_ids.includes(field.definition.id))
        .map((field) => field.definition),
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
        <Button
          ref={linksTrigger}
          onClick={() => {
            const limit = editor?.limits.relationship_leaves;
            if (limit !== undefined && linkClauses(search.filter).length >= limit) {
              setAdvanced(true);
              return;
            }
            const id = crypto.randomUUID();
            openPicker(undefined, undefined, id, (relationship) =>
              latestEditor.current.addLink(relationship, id),
            );
          }}
        >
          Links <DownOutlined aria-hidden="true" />
        </Button>
        {additionalFilters}
        <div className="filter-panel-actions">
          <Tooltip
            title={advancedApplied ? "Advanced filters applied" : undefined}
            trigger={["hover", "focus"]}
          >
            <Badge dot={advancedApplied} color={filterTokens.colorPrimary}>
              <Button
                ref={advancedTrigger}
                className="filter-controls__advanced"
                color={advancedApplied ? "primary" : "default"}
                variant="outlined"
                aria-description={
                  advancedApplied ? "Advanced filters applied" : undefined
                }
                icon={<FilterOutlined aria-hidden="true" />}
                onClick={() => setAdvanced(!advanced)}
                aria-expanded={advanced}
                aria-label="Advanced filters"
              >
                Advanced
              </Button>
            </Badge>
          </Tooltip>
          {(search.filter ||
            search.filterError ||
            prepared.error ||
            (prepared.query?.rules.length ?? 0) > 0) && (
            <Tooltip title="Clear filters">
              <Button
                icon={<ClearOutlined aria-hidden="true" />}
                aria-label="Clear filters"
                onClick={() => {
                  prepared.clear();
                  setDraftRevision((revision) => revision + 1);
                  setSearch(clearAllFilters(search));
                  advancedTrigger.current?.focus();
                }}
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
          onEdit={(leaf, relationship) => {
            if (!relationship) {
              openPicker(leaf.relationship, linkTitle(leaf), leaf.clause_id!, (next) =>
                latestEditor.current.editLink(leaf, next),
              );
              return null;
            }
            const error = latestEditor.current.editLink(leaf, relationship);
            const title = linkTitle(leaf);
            if (
              !error &&
              generation &&
              title !== "Loading record…" &&
              title !== "Record unavailable"
            )
              setDraftLabels((current) => ({
                ...current,
                [leaf.clause_id!]: { relationship, title, generation },
              }));
            return error;
          }}
          onRemove={(leaf) => {
            restoreOverlayFocus(
              linksTrigger.current,
              linksTrigger.current?.ownerDocument.activeElement as HTMLElement | null,
            );
            latestEditor.current.editLink(leaf, null);
          }}
        />
      </div>
    </>
  );
  return (
    <LinkEditingContext value={linkEditing}>
      <aside className={layout === "panel" ? "filter-panel" : "filter-toolbar"}>
        {children}
        <div className="filter-controls__chrome">
          {includeSearch && (
            <PrimaryToolbar className="filter-controls__search-row">
              {searchEntry}
              {searchRowEnd}
            </PrimaryToolbar>
          )}
          <div className="filter-controls__criteria">{criteria}</div>
        </div>
        <Drawer
          title="Advanced filters"
          placement="left"
          size="min(100vw, 640px)"
          open={advanced}
          onClose={() => setAdvanced(false)}
        >
          <section className="advanced-filters" aria-label="Advanced filters">
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
          </section>
        </Drawer>
        {!advanced && prepared.error && inlineError}
        {workspace.filterDiscoveryError && (
          <ErrorNotice
            error={workspace.filterDiscoveryError}
            title="Filter information could not be loaded"
            description={workspace.filterDiscoveryError}
          />
        )}
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
  selectionContainer,
}: {
  field: FilterEditorFieldView;
  selectionContainer?: HTMLElement | null;
  layout: "panel" | "toolbar";
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
          {condition.status === "editable" && (
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
}: {
  field: FilterEditorFieldView;
  range: [string, string];
  onChange: (range: [string, string]) => void;
}) {
  const id = useId();
  let error: string | null = null;
  try {
    const min = range[0].trim() ? numericLiteral(range[0]) : null;
    const max = range[1].trim() ? numericLiteral(range[1]) : null;
    if (min !== null && max !== null && min > max)
      throw new Error("Minimum exceeds maximum");
  } catch (failure) {
    error = failure instanceof Error ? failure.message : String(failure);
  }
  return (
    <div>
      <div className="filter-range">
        <label className="filter-range__bound" htmlFor={id + "-minimum"}>
          <span>Minimum</span>
          <Input
            id={id + "-minimum"}
            aria-label={field.label + " minimum"}
            name={field.definition.id + "-minimum"}
            inputMode="decimal"
            autoComplete="off"
            placeholder="Any"
            status={error ? "error" : undefined}
            value={range[0]}
            onChange={(event) => onChange([event.target.value, range[1]])}
          />
        </label>
        <label className="filter-range__bound" htmlFor={id + "-maximum"}>
          <span>Maximum</span>
          <Input
            id={id + "-maximum"}
            aria-label={field.label + " maximum"}
            name={field.definition.id + "-maximum"}
            inputMode="decimal"
            autoComplete="off"
            placeholder="Any"
            status={error ? "error" : undefined}
            value={range[1]}
            onChange={(event) => onChange([range[0], event.target.value])}
          />
        </label>
      </div>
      {error && (
        <Typography.Text type="danger" role="status">
          {error}
        </Typography.Text>
      )}
    </div>
  );
}

type BuilderContext = {
  editor: FilterEditorView;
  query: RuleGroupType;
  scope: string | null;
  narrowFields: boolean;
  workspace: FilterPanelState;
  replaceGroup: (path: number[], group: RuleGroupType) => void;
  replaceRule: (path: number[], rule: RuleType) => void;
};
function StructuredBuilder({
  editor,
  query,
  onChange,
  scope = null,
  useApplicability = false,
  workspace,
}: {
  editor: FilterEditorView;
  query: RuleGroupType;
  onChange: (query: RuleGroupType) => void;
  scope?: string | null;
  useApplicability?: boolean;
  workspace: FilterPanelState;
}) {
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
          ...(scope === null
            ? [
                {
                  label: "Record links",
                  options: [{ name: LINK_CONDITION, label: "Record links" }],
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
          { name: "and", label: "Match all" },
          { name: "or", label: "Match any" },
        ]}
        getOperators={(id) =>
          id === LINK_CONDITION
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
            workspace,
            replaceGroup: (path, group) =>
              onChange(
                update(query, { combinator: group.combinator, not: group.not }, path),
              ),
            replaceRule: (path, rule) =>
              onChange(
                normalizeBuilderEdit(
                  query,
                  update(
                    query,
                    {
                      field: rule.field,
                      operator: rule.operator,
                      value: rule.value,
                      meta: rule.meta,
                    },
                    path,
                  ),
                  editor.groups.flatMap((group) => group.fields),
                  scope,
                ),
              ),
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
  replace: (rule: PredicateRule) => void;
} | null>(null);
function FilterRule(props: RuleProps) {
  const rule = useRule(props);
  const { replaceRule } = props.context as BuilderContext;
  return (
    <RuleEditContext.Provider
      value={{
        rule: props.rule as PredicateRule,
        replace: (next) => replaceRule(props.path, next),
      }}
    >
      <div
        className={rule.outerClassName}
        data-rule-id={props.rule.id}
        data-link-condition={props.rule.field === LINK_CONDITION || undefined}
      >
        <RuleComponents {...rule} />
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
function BuilderFieldSelector(props: FieldSelectorProps) {
  const { editor, scope, narrowFields } = props.context as BuilderContext;
  const editing = useContext(RuleEditContext);
  const selectedKind = scope === null ? actorMetricKind(props.value ?? "~") : null;
  const addingCondition = !props.value || props.value === "~";
  const seenKinds = new Set<ActorMetricKind>();
  const groups = [
    ...editor.groups
      .map((group) => ({
        label: group.label,
        options: group.fields
          .filter(
            (field) =>
              field.definition.scope === scope &&
              (scope === null || field.definition.field_type !== "collection") &&
              (!addingCondition || !narrowFields || field.applicable) &&
              (!field.unit_fields.length ||
                field.unit_fields[0].field_id === field.definition.id),
          )
          .flatMap((field) => {
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
                label: field.label,
                searchText: field.label,
              },
            ];
          }),
      }))
      .filter((group) => group.options.length),
    ...(scope === null
      ? [
          {
            label: "Record links",
            options: [
              {
                value: LINK_CONDITION,
                label: "Record links",
                searchText: "Record links",
              },
            ],
          },
        ]
      : []),
    ...(props.value === BOOLEAN_CONDITION
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
  const chooser = (
    <Select
      className={selectedKind ? undefined : props.className}
      aria-label="Filter field"
      placeholder="Choose a filter…"
      showSearch={{ optionFilterProp: "searchText" }}
      options={groups}
      value={
        selectedKind
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
        type="text"
        icon={<CloseOutlined />}
        aria-label={props.title || "Remove condition"}
        disabled={props.disabled}
        onClick={props.handleOnClick}
      />
    </Tooltip>
  );
}
function GroupMatchSelector(props: CombinatorSelectorProps) {
  const { replaceGroup } = props.context as BuilderContext;
  const group = props.ruleGroup as RuleGroupType;
  return (
    <Select
      aria-label="Match conditions"
      popupMatchSelectWidth={false}
      value={(group.not ? "not:" : "") + group.combinator}
      options={[
        { value: "and", label: "Match all" },
        { value: "or", label: "Match any" },
        { value: "not:or", label: "Match none" },
        { value: "not:and", label: "Not all match" },
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
  return (
    <>
      <RuleGroupBodyComponents {...props} />
      <Space className="filter-group-actions">
        <Button
          type="primary"
          icon={<PlusOutlined />}
          aria-label="Add condition"
          disabled={props.disabled}
          onClick={props.addRule}
        >
          Add condition
        </Button>
        {props.schema.maxLevels > props.path.length && (
          <Button type="text" disabled={props.disabled} onClick={props.addGroup}>
            Add group
          </Button>
        )}
      </Space>
    </>
  );
}
function BuilderOperatorSelector(props: OperatorSelectorProps) {
  const editing = useContext(RuleEditContext);
  if (isActorMetricChooser(props.field)) return null;
  if (props.field === LINK_CONDITION)
    return (
      <Select
        className={props.className}
        aria-label="Link condition"
        value={(editing?.rule.meta?.atlasExcluded ? "not:" : "") + props.value}
        disabled={props.disabled}
        options={[
          { value: "incoming", label: "Links to" },
          { value: "outgoing", label: "Linked from" },
          { value: "not:incoming", label: "Does not link to" },
          { value: "not:outgoing", label: "Not linked from" },
        ]}
        onChange={(value) =>
          editing?.replace({
            ...editing.rule,
            operator: value.replace("not:", ""),
            meta: { ...editing.rule.meta, atlasExcluded: value.startsWith("not:") },
          })
        }
      />
    );
  const { editor } = props.context as BuilderContext;
  const field = editor.groups
    .flatMap((group) => group.fields)
    .find((field) => field.definition.id === props.field);
  const excluded = !!editing?.rule.meta?.atlasExcluded;
  const operators =
    field?.operators.filter(
      (operator) => operator.operator !== "state" || props.value === "state",
    ) ?? [];
  const options = operators.map((operator) => ({
    value: operator.operator,
    label: operator.operator === "exists" ? "has" : operator.label,
  }));
  for (const operator of operators) {
    if (
      ["in", "includes_all", "exists"].includes(operator.operator) ||
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
                      ? "does not include all"
                      : "not " + operator.label,
      });
    }
  }
  return (
    <Select
      className={props.className}
      aria-label={"Condition for " + (field?.label ?? "this field")}
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
function BuilderValueEditor(props: ValueEditorProps) {
  const { token } = theme.useToken();
  const editing = useContext(RuleEditContext);
  const { editor, query, scope, workspace } = props.context as BuilderContext;
  const links = useContext(LinkEditingContext);
  if (props.field === LINK_CONDITION) {
    const rule = props.rule as PredicateRule;
    const relationship = rule.value?.selection
      ? ({ ...rule.value, direction: rule.operator } as QueryRelationship)
      : undefined;
    const leaf = relationship
      ? { kind: "relationship" as const, clause_id: rule.id!, relationship }
      : null;
    return (
      <div className="filter-rule-value">
        {scope !== null ? (
          <Typography.Text type="danger" role="status">
            Link conditions belong outside collection conditions.
          </Typography.Text>
        ) : (
          <Button
            disabled={props.disabled}
            styles={{
              root: {
                maxWidth: "100%",
                height: "auto",
                minHeight: token.controlHeight,
                whiteSpace: "normal",
                textAlign: "start",
                overflowWrap: "anywhere",
              },
            }}
            onClick={() => {
              if (!links || !editing)
                throw new Error("Link condition picker provider is required");
              links.open(editing.rule, editing.replace);
            }}
          >
            {leaf
              ? `${links?.title(leaf) ?? "Record unavailable"} · ${relationship?.include_contents ? "Whole entry" : "Main entry only"}`
              : "Choose record"}
          </Button>
        )}
      </div>
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
  const field = editor.groups
    .flatMap((group) => group.fields)
    .find(
      (field) =>
        field.definition.id === props.field && field.definition.scope === scope,
    );
  if (!field)
    return props.field === "~" || isActorMetricChooser(props.field) ? null : (
      <Typography.Text type="danger" role="status" className="filter-rule-value">
        This field is unavailable. Choose another field or remove this condition.
      </Typography.Text>
    );
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
  const clauseId = props.rule.id ?? null;
  // An unaddressed facet evaluates an added conjunction. That cannot describe
  // inserting a condition beneath OR or group negation.
  const countsEnabled =
    workspace.filterCountsEnabled !== false &&
    (isAdditivePath(query, props.path) ||
      !!existingClauseId(workspace.filterDiscoveryContext, clauseId));
  return (
    <div className="filter-rule-value">
      <Form.Item
        style={{ marginBottom: 0 }}
        validateStatus={error && !incomplete ? "error" : undefined}
        help={error && !incomplete ? <span role="status">{error}</span> : undefined}
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
