import type { MetadataFilterNode, MetadataPredicate } from "../../search/filters/types.js";
import { createEmptyStringPolicy } from "../search/policies.js";
import type { Pf2eTerminalFilterValuePolicy, Pf2eTerminalQueryFieldOption } from "../search/service-types.js";

export function buildFilterExplorerMetadataNodeFromPolicy(
  fieldOption: Pf2eTerminalQueryFieldOption,
  policy: Pf2eTerminalFilterValuePolicy<string>,
): MetadataFilterNode | null {
  const clauses: MetadataFilterNode[] = [];

  if (fieldOption.fieldType === "set") {
    if (policy.any.length > 0) {
      clauses.push({ field: fieldOption.value, op: "includesAny", values: [...policy.any] } as MetadataFilterNode);
    }
    if (policy.all.length > 0) {
      clauses.push({ field: fieldOption.value, op: "includesAll", values: [...policy.all] } as MetadataFilterNode);
    }
    if (policy.exclude.length > 0) {
      clauses.push({ field: fieldOption.value, op: "excludesAny", values: [...policy.exclude] } as MetadataFilterNode);
    }
  }

  if (fieldOption.fieldType === "enumString") {
    if (policy.any.length === 1) {
      clauses.push({ field: fieldOption.value, op: "eq", value: policy.any[0]! } as MetadataFilterNode);
    } else if (policy.any.length > 1) {
      clauses.push({ field: fieldOption.value, op: "in", values: [...policy.any] } as MetadataFilterNode);
    }
    if (policy.exclude.length > 0) {
      clauses.push({ field: fieldOption.value, op: "notIn", values: [...policy.exclude] } as MetadataFilterNode);
    }
  }

  if (clauses.length === 0) {
    return null;
  }

  return clauses.length === 1 ? clauses[0]! : { and: clauses };
}

export function buildFilterExplorerPolicyFromPredicate(
  node: MetadataPredicate,
): Pf2eTerminalFilterValuePolicy<string> | null {
  const policy = createEmptyStringPolicy();

  if ("values" in node) {
    if (node.op === "includesAny" || node.op === "in") {
      policy.any = [...node.values.map((value) => String(value))];
    } else if (node.op === "includesAll") {
      policy.all = [...node.values.map((value) => String(value))];
    } else {
      policy.exclude = [...node.values.map((value) => String(value))];
    }
    return policy;
  }

  if ("value" in node && node.op === "eq") {
    policy.any = [String(node.value)];
    return policy;
  }

  return null;
}

export function buildFilterExplorerPolicyFromMetadataNode(
  node: MetadataFilterNode | null,
): Pf2eTerminalFilterValuePolicy<string> {
  if (!node) {
    return createEmptyStringPolicy();
  }

  if ("and" in node) {
    return node.and.reduce((policy, child) => {
      const childPolicy = buildFilterExplorerPolicyFromMetadataNode(child);
      return {
        any: [...policy.any, ...childPolicy.any],
        all: [...policy.all, ...childPolicy.all],
        exclude: [...policy.exclude, ...childPolicy.exclude],
      };
    }, createEmptyStringPolicy());
  }

  if ("or" in node || "not" in node) {
    return createEmptyStringPolicy();
  }

  return buildFilterExplorerPolicyFromPredicate(node) ?? createEmptyStringPolicy();
}
