import {
  Alert,
  AutoComplete,
  Button,
  Form,
  Input,
  InputNumber,
  Select,
  Space,
  Typography,
} from "antd";
import { useMemo } from "react";
import {
  QueryBuilder,
  type RuleGroupType,
  type ValueEditorProps,
} from "react-querybuilder";
import { QueryBuilderAntD } from "@react-querybuilder/antd";
import type { QueryFieldDefinition } from "../../generated/atlas";
import type { SearchWorkspaceState } from "../../features/search/useSearchWorkspace";
import { builderPredicate, emptyGroup, predicateBuilder } from "./queryBuilder";
import { clearAllFilters, type FilterPanelState } from "./filterControls";
import { useState } from "react";
import "react-querybuilder/dist/query-builder.css";
export function FilterPanel({ workspace }: { workspace: SearchWorkspaceState }) {
  return <FilterControls filterState={workspace} />;
}
export function FilterControls({
  filterState: workspace,
  includeSearch = true,
  includeResultOptions = true,
}: {
  filterState: FilterPanelState;
  includeSearch?: boolean;
  includeResultOptions?: boolean;
}) {
  const { search, setSearch } = workspace;
  const fields = useMemo(
    () =>
      workspace.filterEditor?.groups.flatMap((g) =>
        g.fields.map((f) => f.definition),
      ) || [],
    [workspace.filterEditor],
  );
  const [draft, setDraft] = useState<RuleGroupType>(emptyGroup);
  const [error, setError] = useState<string | null>(null);
  const prepared = useMemo(() => {
    if (search.filterError) return { query: draft, error: search.filterError };
    try {
      return { query: predicateBuilder(search.filter, fields), error: null };
    } catch (e) {
      return { query: null, error: e instanceof Error ? e.message : String(e) };
    }
  }, [fields, search.filter, search.filterError, draft]);
  function change(group: RuleGroupType) {
    setDraft(group);
    try {
      const filter = builderPredicate(group, fields);
      setError(null);
      const selected = new Set<string>();
      const visit = (g: RuleGroupType) =>
        g.rules.forEach((r) => {
          if ("rules" in r) visit(r as RuleGroupType);
          else {
            selected.add(r.field);
            if (r.operator === "exists" && r.value && "rules" in r.value)
              visit(r.value);
          }
        });
      visit(group);
      setSearch({
        ...search,
        filter,
        filterError: null,
        visibleFilterIds: [...selected],
      });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      setSearch({ ...search, filterError: message });
    }
  }
  return (
    <aside className="filter-panel">
      <Form layout="vertical">
        {includeSearch && (
          <>
            <Form.Item label="Search">
              <Input
                placeholder="Search records"
                value={search.query}
                onChange={(e) =>
                  setSearch({
                    ...search,
                    query: e.target.value,
                    mode: e.target.value.trim() ? "text_search" : "browse",
                  })
                }
              />
            </Form.Item>
            <Form.Item label="Retrieval">
              <Select
                value={search.retrievalMode}
                options={[
                  { value: "hybrid", label: "Hybrid" },
                  { value: "lexical", label: "Names and definitions" },
                  { value: "semantic", label: "Meaning" },
                ]}
                onChange={(retrievalMode) => setSearch({ ...search, retrievalMode })}
              />
            </Form.Item>
          </>
        )}
        <Typography.Title level={5}>Filters</Typography.Title>
        {fields.length > 0 && prepared.query && (
          <StructuredBuilder
            fields={fields}
            query={prepared.query}
            onChange={change}
            facets={workspace.filterValuesByField}
          />
        )}
        {Object.entries(workspace.filterCountsByField || {}).map(([id, result]) =>
          result ? (
            <Typography.Paragraph key={id} type="secondary">
              {fields.find((f) => f.id === result.counts.field)?.label}:{" "}
              {result.counts.minimum ?? "unknown"} –{" "}
              {result.counts.maximum ?? "unknown"}; {result.counts.count_basis}
            </Typography.Paragraph>
          ) : null,
        )}
        <Button
          onClick={() => {
            setDraft(emptyGroup());
            setSearch(clearAllFilters(search));
          }}
        >
          Clear filters
        </Button>
        {(error || prepared.error) && (
          <Alert type="error" message={error || prepared.error} />
        )}
        {workspace.errorMessage && (
          <Alert type="info" message={workspace.errorMessage} />
        )}
        {includeResultOptions && (
          <Form.Item label="Page size">
            <InputNumber
              min={1}
              max={100}
              value={search.pageSize}
              onChange={(v) => setSearch({ ...search, pageSize: v || 25 })}
            />
          </Form.Item>
        )}
      </Form>
    </aside>
  );
}
function StructuredBuilder({
  fields,
  query,
  onChange,
  scope = null,
  facets,
}: {
  fields: QueryFieldDefinition[];
  query: RuleGroupType;
  onChange: (q: RuleGroupType) => void;
  scope?: string | null;
  facets: FilterPanelState["filterValuesByField"];
}) {
  const visible = fields.filter((f) => f.scope === scope);

  return (
    <QueryBuilderAntD>
      <QueryBuilder
        query={query}
        onQueryChange={onChange}
        showNotToggle
        fields={visible.map((f) => ({
          name: f.id,
          label: f.label + (f.units ? " (" + f.units + ")" : ""),
        }))}
        getOperators={(id) =>
          (visible.find((f) => f.id === id)?.operators || []).map((name) => ({
            name,
            label: name.replace(/_/g, " "),
          }))
        }
        context={{ fields, facets, scope }}
        controlElements={{ valueEditor: BuilderValueEditor }}
      />
    </QueryBuilderAntD>
  );
}

type BuilderContext = {
  fields: QueryFieldDefinition[];
  facets: FilterPanelState["filterValuesByField"];
  scope: string | null;
};
function BuilderValueEditor(props: ValueEditorProps) {
  const { fields, facets, scope } = props.context as BuilderContext;
  const visible = fields.filter((f) => f.scope === scope);
  const field = visible.find((f) => f.id === props.field);
  if (!field) return null;
  if (props.operator === "exists")
    return (
      <StructuredBuilder
        fields={fields}
        query={props.value || emptyGroup()}
        onChange={props.handleOnChange}
        scope={field.path}
        facets={facets}
      />
    );
  if (props.operator === "state")
    return (
      <Select
        aria-label="Field state"
        value={props.value || undefined}
        options={["value", "missing", "null", "invalid", "not_applicable"].map(
          (value) => ({ value, label: value.replace(/_/g, " ") }),
        )}
        onChange={props.handleOnChange}
      />
    );
  if (field.field_type === "boolean")
    return (
      <Select
        aria-label={field.label}
        value={props.value === "" ? undefined : String(props.value)}
        options={[
          { value: "true", label: "True" },
          { value: "false", label: "False" },
        ]}
        onChange={props.handleOnChange}
      />
    );
  const options = field.choices.length
    ? field.choices
    : (facets[`${field.id}:${props.rule.id}`] ?? facets[field.id])?.values.options.map(
        (o) => String(o.value),
      ) || [];
  if (field.field_type === "string" && !field.choices.length && props.operator !== "in")
    return (
      <AutoComplete
        aria-label={field.label}
        value={props.value || ""}
        options={options.map((value) => ({ value }))}
        onChange={props.handleOnChange}
      />
    );
  if (field.field_type !== "number" && options.length) {
    const multiple = field.field_type === "set" || props.operator === "in";
    return (
      <Select
        aria-label={field.label}
        mode={multiple ? (field.choices.length ? "multiple" : "tags") : undefined}
        showSearch
        value={props.value || undefined}
        options={options.map((value) => ({ value, label: value }))}
        onChange={props.handleOnChange}
      />
    );
  }
  if (props.operator === "between") {
    const range = Array.isArray(props.value) ? props.value : ["", ""];
    return (
      <Space>
        <Input
          aria-label="Minimum"
          value={range[0]}
          onChange={(e) => props.handleOnChange([e.target.value, range[1]])}
        />
        <Input
          aria-label="Maximum"
          value={range[1]}
          onChange={(e) => props.handleOnChange([range[0], e.target.value])}
        />
      </Space>
    );
  }
  return (
    <Input
      aria-label={field.label}
      inputMode={field.field_type === "number" ? "decimal" : undefined}
      placeholder={
        field.units ||
        (field.field_type === "set" ? "Comma-separated values" : undefined)
      }
      value={Array.isArray(props.value) ? props.value.join(", ") : (props.value ?? "")}
      onChange={(e) => props.handleOnChange(e.target.value)}
    />
  );
}
