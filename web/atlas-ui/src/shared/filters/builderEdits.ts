import type { RuleGroupType, RuleType } from "react-querybuilder";
import type { FilterEditorFieldView } from "../../generated/atlas";
import { BOOLEAN_CONDITION, LINK_CONDITION } from "./queryBuilder";
import { transitionOperand } from "./operatorTransitions";
import { sameActorMetricMeasure } from "./actorMetricFields";

/** Apply an operator/field change atomically before the new control is rendered. */
export function normalizeBuilderEdit(
  previous: RuleGroupType,
  next: RuleGroupType,
  fields: FilterEditorFieldView[],
  scope: string | null,
): RuleGroupType {
  return {
    ...next,
    rules: next.rules.map((node, index) => {
      const old = node.id
        ? (previous.rules.find((candidate) => candidate.id === node.id) ??
          (previous.rules[index]?.id ? undefined : previous.rules[index]))
        : previous.rules[index];
      if ("rules" in node)
        return old && "rules" in old
          ? normalizeBuilderEdit(old, node, fields, scope)
          : node;
      const oldRule = old && !("rules" in old) ? old : undefined;
      // A newly inserted complete condition has no previous operand to convert.
      if (!oldRule) return node;
      const field = fields.find(
        (candidate) =>
          candidate.definition.id === node.field &&
          candidate.definition.scope === scope,
      );
      const operand = field?.operators.find(
        (operator) => operator.operator === node.operator,
      )?.operand;
      if (node.field === LINK_CONDITION && oldRule.field !== node.field)
        return {
          ...node,
          operator: "incoming",
          value: undefined,
          meta: { ...node.meta, atlasExcluded: false },
        };
      if (node.field === BOOLEAN_CONDITION && oldRule?.field !== node.field)
        return { ...node, value: true };
      if (!operand) return node;
      const changedField = oldRule?.field !== node.field;
      if (
        changedField &&
        (field?.unit_fields.some((option) => option.field_id === oldRule.field) ||
          sameActorMetricMeasure(oldRule.field, node.field))
      )
        return node;
      if (changedField || oldRule?.operator !== node.operator) {
        const oldOperand = changedField
          ? null
          : (field?.operators.find(
              (operator) => operator.operator === oldRule?.operator,
            )?.operand ?? null);
        return {
          ...node,
          value: transitionOperand(oldOperand, operand, oldRule?.value),
        };
      }
      if (operand === "predicate" && isBuilderGroup(node.value))
        return {
          ...node,
          value: normalizeBuilderEdit(
            isBuilderGroup(oldRule?.value)
              ? oldRule.value
              : { combinator: "and", rules: [] },
            node.value,
            fields,
            field.definition.path,
          ),
        } satisfies RuleType;
      return node;
    }),
  };
}

export function isBuilderGroup(value: unknown): value is RuleGroupType {
  return (
    !!value &&
    typeof value === "object" &&
    "rules" in value &&
    Array.isArray(value.rules) &&
    "combinator" in value &&
    (value.combinator === "and" || value.combinator === "or")
  );
}
