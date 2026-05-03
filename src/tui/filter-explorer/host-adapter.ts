import {
  getFilterExplorerDiscreteClauseOperator,
  getFilterExplorerScalarClause,
  isFilterExplorerScalarTarget,
} from "./compose-state.js";
import type {
  FilterExplorerComposeMode,
  FilterExplorerControllerContext,
  FilterExplorerDiscreteClauseOperator,
  FilterExplorerHostAdapter,
  FilterExplorerInspectAndOpenMode,
  FilterExplorerNode,
  FilterExplorerScalarClause,
  FilterExplorerStateBadge,
  FilterExplorerTargetPresentation,
} from "./types.js";

function formatScalarClauseSummary(clause: FilterExplorerScalarClause): string {
  if (clause.summaryLabel) {
    return clause.summaryLabel;
  }

  if (clause.operator === "between") {
    return `between ${clause.min} and ${clause.max}`;
  }

  const operator =
    clause.operator === "eq"
      ? "="
      : clause.operator === "neq"
        ? "!="
        : clause.operator === "gt"
          ? ">"
        : clause.operator === "gte"
          ? ">="
          : clause.operator === "lt"
            ? "<"
          : "<=";
  return `${operator} ${String(clause.valueLabel ?? clause.value)}`;
}

function toDiscreteStateBadge(
  operator: FilterExplorerDiscreteClauseOperator | undefined,
): FilterExplorerStateBadge {
  return operator === "include" ? { kind: "include" } : operator === "exclude" ? { kind: "exclude" } : { kind: "off" };
}

function resolveScalarClauseForNode(
  controller: FilterExplorerControllerContext | undefined,
  node: FilterExplorerNode | undefined,
  target: ReturnType<NonNullable<FilterExplorerHostAdapter["resolveTarget"]>>,
): FilterExplorerScalarClause | undefined {
  if (!controller || !node || !target || !isFilterExplorerScalarTarget(target)) {
    return undefined;
  }

  if (controller.browser.currentNode?.id === node.id) {
    return controller.selectedScalarClause;
  }

  return getFilterExplorerScalarClause(target, controller.draft);
}

export function resolveFilterExplorerHostTarget(
  host: FilterExplorerHostAdapter,
  node: FilterExplorerNode | undefined,
) {
  return host.resolveTarget?.(node);
}

export function describeFilterExplorerHostNode(args: {
  host: FilterExplorerHostAdapter;
  node: FilterExplorerNode | undefined;
  target?: ReturnType<NonNullable<FilterExplorerHostAdapter["resolveTarget"]>>;
  isFocused: boolean;
  controller?: FilterExplorerControllerContext;
}): FilterExplorerTargetPresentation | undefined {
  const target = args.target ?? resolveFilterExplorerHostTarget(args.host, args.node);
  return args.host.describeNode({
    node: args.node,
    target,
    isFocused: args.isFocused,
    controller: args.controller,
  });
}

export function createComposeFilterExplorerHostAdapter(args: {
  resolveTarget: NonNullable<FilterExplorerHostAdapter["resolveTarget"]>;
  onEditScalarTarget?: FilterExplorerComposeMode["onEditScalarTarget"];
}): FilterExplorerHostAdapter {
  return {
    resolveTarget: args.resolveTarget,
    describeNode: ({ controller, node, target }) => {
      if (!target) {
        return { activationStyle: "none" };
      }

      if (isFilterExplorerScalarTarget(target)) {
        const clause = resolveScalarClauseForNode(controller, node, target);
        return {
          activationStyle: args.onEditScalarTarget ? "edit" : "none",
          stateBadge: clause ? { kind: "custom", text: "ƒ", tone: "accent" } : { kind: "custom", text: "·", tone: "dim" },
          suffixText: controller && node && controller.browser.currentNode?.id === node.id && clause
            ? formatScalarClauseSummary(clause)
            : undefined,
        };
      }

      const operator = controller ? getFilterExplorerDiscreteClauseOperator(target, controller.draft) : undefined;
      return {
        activationStyle: "toggle",
        stateBadge: toDiscreteStateBadge(operator),
      };
    },
  };
}

export function createInspectFilterExplorerHostAdapter(args: {
  resolveTarget?: FilterExplorerHostAdapter["resolveTarget"];
  onEditScalarTarget?: FilterExplorerInspectAndOpenMode["onEditScalarTarget"];
}): FilterExplorerHostAdapter {
  return {
    resolveTarget: args.resolveTarget,
    describeNode: ({ node, target }) => ({
      activationStyle:
        target?.kind === "scalar" && args.onEditScalarTarget
          ? "edit"
          : node?.query
            ? "open"
            : "none",
      tone: node?.kind === "field" ? "default" : undefined,
    }),
  };
}
