import {
  Alert,
  Button,
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
import {
  ClearOutlined,
  CloseOutlined,
  FilterOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import {
  createContext,
  useContext,
  type ReactNode,
  useMemo,
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
import { PrimaryToolbar } from "../ui/PrimaryToolbar";
import { useFilterEditor } from "./useFilterEditor";
import { isBuilderGroup, normalizeBuilderEdit } from "./builderEdits";
import { CollectionValueInput, isCompactCollection } from "./CollectionValueInput";

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
  similarSearch,
  additionalFilters,
  children,
}: {
  filterState: FilterPanelState;
  includeSearch?: boolean;
  layout?: "panel" | "toolbar" | "inline-toolbar";
  searchRowEnd?: ReactNode;
  similarSearch?: { title: string; onClear: () => void; onSearch: () => void };
  additionalFilters?: ReactNode;
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
        onChange={prepared.change}
        workspace={countWorkspace}
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
      similar={similarSearch}
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
              workspace={countWorkspace}
              onAdvanced={() => setAdvanced(true)}
            />
          ) : null;
        })}
      </div>
      {additionalFilters}
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
          {searchRowEnd}
          {criteria}
        </PrimaryToolbar>
      ) : (
        <>
          {includeSearch && (
            <PrimaryToolbar className="filter-controls__search-row">
              {searchEntry}
              {searchRowEnd}
            </PrimaryToolbar>
          )}
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
          countContext={workspace.filterDiscoveryContext}
          countsEnabled={workspace.filterCountsEnabled !== false}
          countQueryKey={workspace.filterCountQueryKey}
          countLimit={workspace.filterEditor?.limits.literal_list ?? 0}
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
        autoSelectField={false}
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
        context={
          {
            editor,
            scope,
            narrowFields: isPositiveConjunction(query),
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
      <div className={rule.outerClassName} data-rule-id={props.rule.id}>
        <RuleComponents {...rule} />
      </div>
    </RuleEditContext.Provider>
  );
}
// Current-result applicability is safe for additive conditions. Boolean groups
// that can broaden the result need the complete catalog while being edited.
function isPositiveConjunction(group: RuleGroupType): boolean {
  return (
    group.combinator === "and" &&
    !group.not &&
    group.rules.every((rule) => !("rules" in rule) || isPositiveConjunction(rule))
  );
}
function BuilderFieldSelector(props: FieldSelectorProps) {
  const { editor, scope, narrowFields } = props.context as BuilderContext;
  const groups = [
    ...editor.groups
      .map((group) => ({
        label: group.label,
        options: group.fields
          .filter(
            (field) =>
              field.definition.scope === scope &&
              (scope === null || field.definition.field_type !== "collection") &&
              (!narrowFields ||
                field.applicable ||
                field.definition.id === props.value ||
                field.unit_fields.some((unit) => unit.field_id === props.value)) &&
              (!field.unit_fields.length ||
                field.unit_fields[0].field_id === field.definition.id),
          )
          .map((field) => ({ value: field.definition.id, label: field.label })),
      }))
      .filter((group) => group.options.length),
    ...(props.value === BOOLEAN_CONDITION
      ? [
          {
            label: "Technical condition",
            options: [{ value: BOOLEAN_CONDITION, label: "Constant condition" }],
          },
        ]
      : []),
  ];
  return (
    <Select
      className={props.className}
      aria-label="Filter field"
      placeholder="Choose a filter…"
      showSearch={{ optionFilterProp: "label" }}
      options={groups}
      value={
        props.value === "~"
          ? undefined
          : (editor.groups
              .flatMap((group) => group.fields)
              .find((field) => field.definition.id === props.value)?.unit_fields[0]
              ?.field_id ?? props.value)
      }
      onChange={props.handleOnChange}
      disabled={props.disabled}
    />
  );
}
function RemoveCondition(props: ActionProps) {
  return (
    <Button
      className={props.className}
      type="text"
      icon={<CloseOutlined />}
      aria-label={props.title || "Remove condition"}
      disabled={props.disabled}
      onClick={props.handleOnClick}
    />
  );
}
function GroupMatchSelector(props: CombinatorSelectorProps) {
  const { replaceGroup } = props.context as BuilderContext;
  const group = props.ruleGroup as RuleGroupType;
  return (
    <Select
      aria-label="Match conditions"
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
  const editing = useContext(RuleEditContext);
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
    return props.field === "~" ? null : (
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
            countsEnabled={workspace.filterCountsEnabled !== false}
            countQueryKey={workspace.filterCountQueryKey}
            countLimit={workspace.filterEditor?.limits.literal_list ?? 0}
            clauseId={String(props.rule.id)}
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
