import { PrimaryToolbar } from "../ui/PrimaryToolbar";
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
  Tooltip,
  Typography,
} from "antd";
import { ClearOutlined, FilterOutlined } from "@ant-design/icons";
import { type ReactNode, useMemo, useRef, useState } from "react";
import {
  QueryBuilder,
  RuleComponents,
  useRule,
  type RuleProps,
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
  commonDraftCondition,
  commonDraftRange,
  BOOLEAN_CONDITION,
  emptyGroup,
  numericLiteral,
  replaceCommonDraft,
  replaceCommonRangeDraft,
  type PredicateRule,
} from "./queryBuilder";
import {
  clearAllFilters,
  type CommonCondition,
  type FilterPanelState,
} from "./filterControls";
import { FilterValueInput } from "./FilterValueInput";
import { SearchEntry } from "./SearchEntry";
import { CommonChoicePicker } from "./CommonChoicePicker";
import { useFilterEditor } from "./useFilterEditor";
import { isBuilderGroup, normalizeBuilderEdit } from "./builderEdits";

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
  layout?: "panel" | "toolbar" | "inline-toolbar";
  children?: ReactNode;
}) {
  const { search, setSearch } = workspace;
  const [advanced, setAdvanced] = useState(false);
  const advancedTrigger = useRef<HTMLButtonElement>(null);
  const [draftRevision, setDraftRevision] = useState(0);
  const editor = workspace.filterEditor;
  const fields = useMemo(
    () => editor?.groups.flatMap((group) => group.fields) ?? [],
    [editor],
  );
  const prepared = useFilterEditor(search, fields, setSearch);
  const advancedEditor =
    editor && prepared.query ? (
      <StructuredBuilder
        editor={editor}
        query={prepared.query}
        onChange={prepared.change}
        workspace={workspace}
      />
    ) : null;
  const commonLayout = layout === "panel" ? "panel" : "toolbar";
  const CriteriaHost = commonLayout === "toolbar" ? PrimaryToolbar : "div";
  const inlineError =
    prepared.error && (search.filterError || !prepared.query) ? (
      <Typography.Text type="danger" role="status">
        {prepared.error}
      </Typography.Text>
    ) : null;
  const searchEntry = includeSearch && (
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
  );
  const criteria = (
    <>
      <div className="common-filters">
        {(editor?.common_field_ids ?? []).map((id) => {
          const field = fields.find((candidate) => candidate.definition.id === id);
          return field ? (
            <CommonFilter
              key={id + ":" + draftRevision}
              field={field}
              layout={commonLayout}
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
              workspace={workspace}
              onAdvanced={() => setAdvanced(true)}
            />
          ) : null;
        })}
      </div>
      <div className="filter-panel-actions">
        <Button
          ref={advancedTrigger}
          icon={<FilterOutlined aria-hidden="true" />}
          onClick={() => setAdvanced(!advanced)}
          aria-expanded={advanced}
        >
          Advanced filters
        </Button>
        {(search.filter ||
          search.filterError ||
          prepared.error ||
          (prepared.query?.rules.length ?? 0) > 0) && (
          <Tooltip title="Clear filters">
            <Button
              type="text"
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
    </>
  );
  return (
    <aside
      className={
        layout === "panel"
          ? "filter-panel"
          : layout === "inline-toolbar"
            ? "filter-toolbar filter-toolbar--inline"
            : "filter-toolbar"
      }
    >
      {children}
      {!editor && workspace.filterDiscoveryLoading && (
        <Spin description="Loading filters…">
          <div className="filter-loading-placeholder" />
        </Spin>
      )}
      {layout === "inline-toolbar" ? (
        <PrimaryToolbar className="filter-controls__criteria filter-controls__inline">
          {searchEntry}
          {criteria}
        </PrimaryToolbar>
      ) : (
        <>
          {searchEntry}
          <CriteriaHost className="filter-controls__criteria">{criteria}</CriteriaHost>
        </>
      )}

      <Drawer
        title="Advanced filters"
        placement="left"
        size="min(100vw, 640px)"
        open={advanced}
        onClose={() => setAdvanced(false)}
      >
        <section className="advanced-filters" aria-label="Advanced filters">
          {advancedEditor}
          {inlineError}
        </section>
      </Drawer>
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
  range,
  onRangeChange,
  condition,
  onChange,
  workspace,
  onAdvanced,
  layout,
}: {
  field: FilterEditorFieldView;
  layout: "panel" | "toolbar";
  condition: CommonCondition;
  range: [string, string] | null;
  onRangeChange: (range: [string, string]) => void;
  onChange: (replacement: QueryPredicate | null) => void;
  workspace: FilterPanelState;
  onAdvanced: () => void;
}) {
  const update = onChange;
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
        <Button onClick={onAdvanced}>{field.label}: Advanced</Button>
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
            placement="bottomLeft"
            title={field.label}
            content={
              <Space orientation="vertical">
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
            <Button aria-label={"Edit " + field.label + " range"}>
              {field.label}: {range?.filter(Boolean).join(" – ") || "Any"}
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
    </section>
  );
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
        <Input
          aria-label={field.label + " minimum"}
          name={field.definition.id + "-minimum"}
          inputMode="decimal"
          autoComplete="off"
          placeholder="Minimum…"
          status={error ? "error" : undefined}
          value={range[0]}
          onChange={(event) => onChange([event.target.value, range[1]])}
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
          onChange={(event) => onChange([range[0], event.target.value])}
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
          rule: FilterRule,
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
function FilterRule(props: RuleProps) {
  const rule = useRule(props);
  const excluded = !!(props.rule as PredicateRule).meta?.atlasExcluded;
  return (
    <div className={rule.outerClassName} data-rule-id={props.rule.id}>
      <RuleComponents {...rule} />
      <Checkbox
        className="filter-rule-exclude"
        checked={excluded}
        disabled={props.disabled}
        onChange={(event) =>
          rule.generateOnChangeHandler("meta")({
            ...props.rule.meta,
            atlasExcluded: event.target.checked,
          })
        }
      >
        Exclude
      </Checkbox>
    </div>
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
  if (!field)
    return (
      <Typography.Text type="danger" role="status" className="filter-rule-value">
        This field is unavailable. Choose another field or remove this condition.
      </Typography.Text>
    );
  if (props.operator === "exists")
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
        style={{ marginBottom: 0 }}
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
    </div>
  );
}
