import {
  Alert,
  Button,
  Checkbox,
  Drawer,
  Form,
  Input,
  Popover,
  Select,
  Space,
  Spin,
  Typography,
} from "antd";
import { type ReactNode, useEffect, useMemo, useState } from "react";
import {
  QueryBuilder,
  type RuleGroupType,
  type ValueEditorProps,
  type FieldSelectorProps,
  type NotToggleProps,
  type OperatorSelectorProps,
  type CombinatorSelectorProps,
} from "react-querybuilder";
import { QueryBuilderAntD } from "@react-querybuilder/antd";
import type {
  FilterEditorFieldView,
  FilterEditorView,
  QueryPredicate,
} from "../../generated/atlas";
import type { SearchWorkspaceState } from "../../features/search/useSearchWorkspace";
import {
  builderPredicate,
  BOOLEAN_CONDITION,
  emptyGroup,
  numericLiteral,
  predicateBuilder,
} from "./queryBuilder";
import {
  advancedResidual,
  clearAllFilters,
  commonCondition,
  conditionRange,
  replaceCommonCondition,
  replaceAdvancedResidual,
  type FilterPanelState,
} from "./filterControls";
import { FilterValueInput } from "./FilterValueInput";
import { SearchEntry } from "./SearchEntry";
import { CommonChoicePicker } from "./CommonChoicePicker";
import "react-querybuilder/dist/query-builder.css";
import { ATLAS_ROUTE_CHANGE_EVENT } from "../../app/routes";

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
  children,
}: {
  filterState: FilterPanelState;
  includeSearch?: boolean;
  layout?: "panel" | "toolbar";
  children?: ReactNode;
}) {
  const { search, setSearch } = workspace;
  const [advanced, setAdvanced] = useState(false);
  const [draftRevision, setDraftRevision] = useState(0);
  const [rangeDrafts, setRangeDrafts] = useState<Record<string, RangeDraft>>({});
  const [draft, setDraft] = useState<{
    baseline: QueryPredicate | null;
    query: RuleGroupType;
    error: string | null;
  } | null>(null);
  useEffect(() => {
    const reset = () => {
      setDraft(null);
      setRangeDrafts({});
      setDraftRevision((revision) => revision + 1);
    };
    window.addEventListener("popstate", reset);
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, reset);
    return () => {
      window.removeEventListener("popstate", reset);
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, reset);
    };
  }, []);
  const editor = workspace.filterEditor;
  const fields = useMemo(
    () => editor?.groups.flatMap((group) => group.fields) ?? [],
    [editor],
  );
  const definitions = useMemo(() => fields.map((field) => field.definition), [fields]);
  const commonDefinitions = useMemo(
    () => definitions.filter((field) => editor?.common_field_ids.includes(field.id)),
    [definitions, editor],
  );
  const residual = useMemo(
    () => advancedResidual(search.filter, commonDefinitions).residual,
    [search.filter, commonDefinitions],
  );
  const prepared = useMemo(() => {
    if (!editor) return { query: null, error: search.filterError };
    if (draft?.baseline === search.filter)
      return { query: draft.query, error: draft.error };
    try {
      return {
        query: predicateBuilder(residual, definitions),
        error: search.filterError,
      };
    } catch (error) {
      return {
        query: null,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }, [definitions, editor, draft, search.filter, search.filterError, residual]);
  function change(query: RuleGroupType) {
    try {
      const filter = replaceAdvancedResidual(
        search.filter,
        commonDefinitions,
        builderPredicate(query, definitions),
      );
      setDraft(null);
      setSearch({ ...search, filter, filterError: null });
    } catch (error) {
      setDraft({
        baseline: search.filter,
        query,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
  const advancedEditor =
    editor && prepared.query ? (
      <StructuredBuilder
        editor={editor}
        query={prepared.query}
        onChange={change}
        workspace={workspace}
      />
    ) : null;
  const inlineError =
    prepared.error && !prepared.query ? (
      <Typography.Text type="danger" role="status">
        {prepared.error}
      </Typography.Text>
    ) : null;
  return (
    <aside className={layout === "toolbar" ? "filter-toolbar" : "filter-panel"}>
      {children}
      {!editor && workspace.filterDiscoveryLoading && (
        <Spin description="Loading filters…">
          <div className="filter-loading-placeholder" />
        </Spin>
      )}
      {includeSearch && (
        <SearchEntry
          query={search.query}
          onSearch={(query) =>
            setSearch({
              ...search,
              query,
              mode: query.trim() ? "text_search" : "browse",
            })
          }
        />
      )}
      <div className="common-filters">
        {(editor?.common_field_ids ?? []).map((id) => {
          const field = fields.find((candidate) => candidate.definition.id === id);
          return field ? (
            <CommonFilter
              key={id + ":" + draftRevision}
              field={field}
              layout={layout}
              rangeDraft={rangeDrafts[id] ?? null}
              onRangeDraftChange={(next) =>
                setRangeDrafts((current) => ({ ...current, [id]: next }))
              }
              workspace={{
                ...workspace,
                setSearch: (next) => {
                  setDraft((current) =>
                    current?.baseline === search.filter
                      ? { ...current, baseline: next.filter }
                      : current,
                  );
                  setSearch(next);
                },
              }}
              onAdvanced={() => setAdvanced(true)}
            />
          ) : null;
        })}
      </div>
      <div className="filter-panel-actions">
        <Button onClick={() => setAdvanced(!advanced)} aria-expanded={advanced}>
          Advanced filters
        </Button>
        <Button
          type="text"
          onClick={() => {
            setDraft(null);
            setRangeDrafts({});
            setDraftRevision((revision) => revision + 1);
            setSearch(clearAllFilters(search));
          }}
        >
          Clear filters
        </Button>
      </div>
      {
        <Drawer
          title="Advanced filters"
          placement="left"
          size="min(100vw, 640px)"
          open={advanced}
          onClose={() => setAdvanced(false)}
        >
          <section className="advanced-filters" aria-label="Advanced filters">
            <Typography.Text type="secondary">Additional conditions</Typography.Text>
            {advancedEditor}
            {inlineError}
          </section>
        </Drawer>
      }
      {!advanced && prepared.error && inlineError}
      {workspace.filterDiscoveryError && (
        <Alert
          type="error"
          title="Filter information could not be loaded"
          description={workspace.filterDiscoveryError}
        />
      )}
    </aside>
  );
}

function CommonFilter({
  field,
  rangeDraft,
  onRangeDraftChange,
  workspace,
  onAdvanced,
  layout,
}: {
  field: FilterEditorFieldView;
  layout: "panel" | "toolbar";
  rangeDraft: RangeDraft | null;
  onRangeDraftChange: (draft: RangeDraft) => void;
  workspace: FilterPanelState;
  onAdvanced: () => void;
}) {
  const condition = commonCondition(workspace.search.filter, field.definition);
  function update(replacement: QueryPredicate | null) {
    workspace.setSearch({
      ...workspace.search,
      filter: replaceCommonCondition(
        workspace.search.filter,
        field.definition,
        replacement,
      ),
      filterError: null,
    });
  }
  return (
    <section
      className={
        layout === "toolbar" ? "common-filter common-filter--toolbar" : "common-filter"
      }
      aria-label={field.label + " filter"}
    >
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
      {condition.status === "advanced" ? (
        <Space orientation="vertical" size="small">
          <Typography.Text type="secondary">
            Defined in advanced filters
          </Typography.Text>
          <Button size="small" onClick={onAdvanced}>
            Edit {field.label.toLowerCase()}
          </Button>
        </Space>
      ) : field.definition.field_type === "number" ? (
        layout === "panel" ? (
          <CommonRange
            field={field}
            clause={condition.clause}
            onChange={update}
            draft={rangeDraft}
            onDraftChange={onRangeDraftChange}
          />
        ) : (
          <Popover
            trigger="click"
            placement="bottomLeft"
            title={field.label}
            content={
              <CommonRange
                field={field}
                clause={condition.clause}
                onChange={update}
                draft={rangeDraft}
                onDraftChange={onRangeDraftChange}
              />
            }
          >
            <Button aria-label={"Edit " + field.label + " range"}>
              {conditionRange(condition.clause).filter(Boolean).join(" – ") || "Any"}
            </Button>
          </Popover>
        )
      ) : (
        <CommonChoicePicker
          field={field}
          clause={condition.clause}
          scope={workspace.vocabularyScope ?? { kind: "artifact" }}
          onChange={update}
        />
      )}
      <CountAvailability fieldId={field.definition.id} workspace={workspace} />
    </section>
  );
}

type RangeDraft = {
  baseline: string;
  range: [string, string];
  error: string | null;
};

function CommonRange({
  field,
  clause,
  onChange,
  draft,
  onDraftChange,
}: {
  field: FilterEditorFieldView;
  clause: QueryPredicate | null;
  onChange: (next: QueryPredicate | null) => void;
  draft: RangeDraft | null;
  onDraftChange: (draft: RangeDraft) => void;
}) {
  const baseline = JSON.stringify(clause);
  const range = draft?.baseline === baseline ? draft.range : conditionRange(clause);
  const error = draft?.baseline === baseline ? draft.error : null;
  function change(next: [string, string]) {
    try {
      const min = next[0].trim() ? numericLiteral(next[0]) : null;
      const max = next[1].trim() ? numericLiteral(next[1]) : null;
      if (min !== null && max !== null && min > max)
        throw new Error("Minimum must not exceed maximum");
      const existing =
        clause?.kind === "all_of" ? clause.children : clause ? [clause] : [];
      const clauseId = (op: string) =>
        existing.find((child) => child.kind === "compare" && child.op === op)
          ?.clause_id ??
        (existing.length === 1 &&
        existing[0].kind === "compare" &&
        existing[0].op === "eq" &&
        (op === "gte" || min === null)
          ? existing[0].clause_id
          : undefined) ??
        crypto.randomUUID();
      const children: QueryPredicate[] = [];
      if (min !== null)
        children.push({
          kind: "compare",
          clause_id: clauseId("gte"),
          field: field.definition.path,
          op: "gte",
          value: min,
        });
      if (max !== null)
        children.push({
          kind: "compare",
          clause_id: clauseId("lte"),
          field: field.definition.path,
          op: "lte",
          value: max,
        });
      const replacement: QueryPredicate | null =
        children.length === 2
          ? {
              kind: "all_of",
              clause_id:
                clause?.kind === "all_of" ? clause.clause_id : crypto.randomUUID(),
              children,
            }
          : (children[0] ?? null);
      onDraftChange({
        baseline: JSON.stringify(replacement),
        range: next,
        error: null,
      });
      onChange(replacement);
    } catch (error) {
      onDraftChange({
        baseline,
        range: next,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }
  return (
    <div>
      <div className="filter-range">
        <Input
          aria-label={field.label + " minimum"}
          name={field.definition.id + "-minimum"}
          inputMode="decimal"
          autoComplete="off"
          placeholder="Minimum…"
          status={error ? "error" : undefined}
          value={range[0]}
          onChange={(event) => change([event.target.value, range[1]])}
        />
        <span>to</span>
        <Input
          aria-label={field.label + " maximum"}
          name={field.definition.id + "-maximum"}
          inputMode="decimal"
          autoComplete="off"
          placeholder="Maximum…"
          status={error ? "error" : undefined}
          value={range[1]}
          onChange={(event) => change([range[0], event.target.value])}
        />
      </div>
      {error && (
        <Typography.Text type="danger" role="status">
          {error}
        </Typography.Text>
      )}
    </div>
  );
}

function CountAvailability({
  fieldId,
  clauseId,
  workspace,
}: {
  fieldId: string;
  clauseId?: string;
  workspace: FilterPanelState;
}) {
  const outcomes = Object.entries(workspace.filterCountsByField ?? {})
    .filter(([key]) =>
      clauseId
        ? key === fieldId + ":" + clauseId
        : key === fieldId || key.startsWith(fieldId + ":"),
    )
    .map(([, outcome]) => outcome?.counts);
  return outcomes.some((outcome) => outcome?.status === "unavailable") ? (
    <Typography.Text type="secondary" className="filter-count-note">
      Counts unavailable for this condition
    </Typography.Text>
  ) : null;
}

type BuilderContext = {
  editor: FilterEditorView;
  scope: string | null;
  workspace: FilterPanelState;
};
function StructuredBuilder({
  editor,
  query,
  onChange,
  scope = null,
  workspace,
}: {
  editor: FilterEditorView;
  query: RuleGroupType;
  onChange: (query: RuleGroupType) => void;
  scope?: string | null;
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
        onQueryChange={onChange}
        showNotToggle
        enableMountQueryChange={false}
        fields={[
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
          {
            label: "Structural conditions",
            options: [{ name: BOOLEAN_CONDITION, label: "All records / No records" }],
          },
        ]}
        combinators={[
          { name: "and", label: "Match all" },
          { name: "or", label: "Match any" },
        ]}
        getOperators={(id) =>
          id === BOOLEAN_CONDITION
            ? [{ name: "constant", label: "Matches" }]
            : (
                visible.find((field) => field.definition.id === id)?.operators ?? []
              ).map((operator) => ({ name: operator.operator, label: operator.label }))
        }
        context={{ editor, scope, workspace } satisfies BuilderContext}
        translations={{
          addRule: { label: "Add condition", title: "Add condition" },
          addGroup: { label: "Add group", title: "Add group" },
          removeRule: { label: "Remove", title: "Remove condition" },
          removeGroup: { label: "Remove group", title: "Remove group" },
        }}
        controlElements={{
          valueEditor: BuilderValueEditor,
          fieldSelector: BuilderFieldSelector,
          operatorSelector: BuilderOperatorSelector,
          combinatorSelector: BuilderCombinatorSelector,
          notToggle: ExcludeGroup,
        }}
      />
    </QueryBuilderAntD>
  );
}
function BuilderFieldSelector(props: FieldSelectorProps) {
  const { editor, scope } = props.context as BuilderContext;
  const groups = [
    ...editor.groups
      .map((group) => ({
        label: group.label,
        options: group.fields
          .filter(
            (field) =>
              field.definition.scope === scope &&
              (scope === null || field.definition.field_type !== "collection"),
          )
          .map((field) => ({ value: field.definition.id, label: field.label })),
      }))
      .filter((group) => group.options.length),
    {
      label: "Structural conditions",
      options: [{ value: BOOLEAN_CONDITION, label: "All records / No records" }],
    },
  ];
  return (
    <Select
      className={props.className}
      aria-label="Filter field"
      showSearch={{ optionFilterProp: "label" }}
      options={groups}
      value={props.value}
      onChange={props.handleOnChange}
      disabled={props.disabled}
    />
  );
}
function ExcludeGroup(props: NotToggleProps) {
  return (
    <Checkbox
      className={props.className}
      checked={props.checked}
      onChange={(event) => props.handleOnChange(event.target.checked)}
      disabled={props.disabled}
    >
      Exclude this group
    </Checkbox>
  );
}
function BuilderOperatorSelector(props: OperatorSelectorProps) {
  const { editor } = props.context as BuilderContext;
  const field = editor.groups
    .flatMap((group) => group.fields)
    .find((field) => field.definition.id === props.field);
  return (
    <Select
      className={props.className}
      aria-label={"Condition for " + (field?.label ?? "this field")}
      value={props.value}
      options={
        props.field === BOOLEAN_CONDITION
          ? [{ value: "constant", label: "Matches" }]
          : (field?.operators.map((operator) => ({
              value: operator.operator,
              label: operator.label,
            })) ?? [])
      }
      onChange={props.handleOnChange}
      disabled={props.disabled}
    />
  );
}
function BuilderCombinatorSelector(props: CombinatorSelectorProps) {
  return (
    <Select
      className={props.className}
      aria-label="Match conditions"
      value={props.value}
      options={[
        { value: "and", label: "Match all" },
        { value: "or", label: "Match any" },
      ]}
      onChange={props.handleOnChange}
      disabled={props.disabled}
    />
  );
}
function BuilderValueEditor(props: ValueEditorProps) {
  const { editor, scope, workspace } = props.context as BuilderContext;
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
  if (!field) return null;
  if (props.operator === "exists")
    return (
      <div className="filter-collection-condition">
        <StructuredBuilder
          editor={editor}
          query={props.value || emptyGroup()}
          onChange={props.handleOnChange}
          scope={field.definition.path}
          workspace={workspace}
        />
      </div>
    );
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
  return (
    <div className="filter-rule-value">
      <Form.Item
        validateStatus={error && !incomplete ? "error" : undefined}
        help={error ? <span role="status">{error}</span> : undefined}
      >
        <FilterValueInput
          field={field}
          operator={props.operator}
          value={props.value}
          onChange={props.handleOnChange}
          scope={workspace.vocabularyScope ?? { kind: "artifact" }}
        />
      </Form.Item>
      <CountAvailability
        fieldId={field.definition.id}
        clauseId={props.rule.id}
        workspace={workspace}
      />
    </div>
  );
}
