import { Button, Form, Input, Select } from "antd";
import { useState } from "react";
import type { RuleGroupType, RuleType } from "react-querybuilder";
import type { FilterEditorFieldView, FilterEditorView } from "../../generated/atlas";
import type { FilterPanelState } from "./filterControls";
import { FilterValueInput } from "./FilterValueInput";
import { numericLiteral } from "./queryBuilder";

/** A compact presentation of one same-entry conjunction, using catalog member IDs. */
export function CollectionValueInput({
  field,
  editor,
  query,
  onChange,
  workspace,
}: {
  field: FilterEditorFieldView;
  editor: FilterEditorView;
  query: RuleGroupType;
  onChange: (query: RuleGroupType) => void;
  workspace: FilterPanelState;
}) {
  const control = field.collection_control!;
  const [showAmount, setShowAmount] = useState(false);
  const [amountOperator, setAmountOperator] = useState("gte");
  const fields = editor.groups.flatMap((group) => group.fields);
  const category = fields.find(
    (candidate) => candidate.definition.id === control.category_field_id,
  )!;
  const amount = fields.find(
    (candidate) => candidate.definition.id === control.amount_field_id,
  );
  const rules = query.rules as RuleType[];
  const categoryRule = rules.find((rule) => rule.field === control.category_field_id);
  const amountRule = rules.find((rule) => rule.field === control.amount_field_id);
  let amountError: string | null = null;
  if (amountRule?.value != null && String(amountRule.value).trim()) {
    try {
      numericLiteral(String(amountRule.value));
    } catch (error) {
      amountError = error instanceof Error ? error.message : String(error);
    }
  }
  function replace(id: string, operator: string, value: unknown) {
    const previous = rules.find((rule) => rule.field === id);
    const next = {
      id: previous?.id ?? crypto.randomUUID(),
      field: id,
      operator,
      value,
    };
    onChange({
      ...query,
      rules: previous
        ? rules.map((rule) => (rule === previous ? next : rule))
        : [...rules, next],
    });
  }
  return (
    <div className="filter-collection-compact">
      <FilterValueInput
        field={category}
        operator={categoryRule?.operator ?? "eq"}
        value={categoryRule?.value ?? ""}
        onChange={(value) =>
          value === "" || value == null
            ? onChange({
                ...query,
                rules: rules.filter((rule) => rule !== categoryRule),
              })
            : replace(category.definition.id, categoryRule?.operator ?? "eq", value)
        }
        scope={workspace.vocabularyScope ?? { kind: "artifact" }}
        countContext={workspace.filterDiscoveryContext}
        countsEnabled={workspace.filterCountsEnabled !== false}
        countQueryKey={workspace.filterCountQueryKey}
        countLimit={editor.limits.literal_list}
        clauseId={categoryRule?.id ?? null}
      />
      {amount && (
        <div className="filter-collection-amount">
          {!showAmount && !amountRule ? (
            <Button
              className="filter-collection-add-amount"
              type="text"
              onClick={() => setShowAmount(true)}
            >
              Add {amount.label.toLowerCase()}
            </Button>
          ) : (
            <>
              <Select
                aria-label={amount.label + " comparison"}
                value={amountRule?.operator ?? amountOperator}
                options={amount.operators
                  .filter((operator) => operator.operand === "scalar")
                  .map((operator) => ({
                    value: operator.operator,
                    label: operator.label,
                  }))}
                onChange={(operator) => {
                  setAmountOperator(operator);
                  if (amountRule)
                    replace(amount.definition.id, operator, amountRule.value);
                }}
              />
              <Form.Item
                style={{ marginBottom: 0 }}
                validateStatus={amountError ? "error" : undefined}
                help={
                  amountError ? <span role="status">{amountError}</span> : undefined
                }
              >
                <Input
                  aria-label={amount.label}
                  inputMode="decimal"
                  suffix={amount.definition.units}
                  placeholder={"Any " + amount.label.toLowerCase()}
                  value={amountRule?.value ?? ""}
                  onChange={(event) =>
                    event.target.value === ""
                      ? onChange({
                          ...query,
                          rules: rules.filter((rule) => rule !== amountRule),
                        })
                      : replace(
                          amount.definition.id,
                          amountRule?.operator ?? amountOperator,
                          event.target.value,
                        )
                  }
                />
              </Form.Item>
            </>
          )}
        </div>
      )}
    </div>
  );
}

export function isCompactCollection(
  field: FilterEditorFieldView,
  query: RuleGroupType,
): boolean {
  const control = field.collection_control;
  if (!control || query.combinator !== "and" || query.not) return false;
  const seen = new Set<string>();
  return query.rules.every((rule) => {
    if ("rules" in rule || rule.meta?.atlasExcluded || seen.has(rule.field))
      return false;
    seen.add(rule.field);
    return (
      (rule.field === control.category_field_id &&
        ["eq", "in"].includes(rule.operator)) ||
      (rule.field === control.amount_field_id &&
        ["eq", "neq", "gt", "gte", "lt", "lte"].includes(rule.operator))
    );
  });
}
