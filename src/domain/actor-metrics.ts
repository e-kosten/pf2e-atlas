import { normalizeText } from "../utils.js";

export const ACTOR_METRIC_NUMERIC_OPERATORS = ["==", "!=", ">", ">=", "<", "<="] as const;
export const ACTOR_METRIC_SCALAR_OPERATORS = ["==", "!="] as const;

export type ActorMetricNumericOperator = (typeof ACTOR_METRIC_NUMERIC_OPERATORS)[number];
export type ActorMetricScalarOperator = (typeof ACTOR_METRIC_SCALAR_OPERATORS)[number];
export type ActorMetricOperator = ActorMetricNumericOperator;
export type ActorMetricValueType = "number" | "text" | "boolean";
export type ActorMetricValue = number | string | boolean;
export type ActorMetricMap = Record<string, ActorMetricValue>;

export const ACTOR_ABILITY_KEYS = ["str", "dex", "con", "int", "wis", "cha"] as const;
export const ACTOR_SAVE_KEYS = ["fort", "ref", "will"] as const;
const RAW_SAVE_KEY_MAP: Record<string, (typeof ACTOR_SAVE_KEYS)[number]> = {
  fort: "fort",
  fortitude: "fort",
  ref: "ref",
  reflex: "ref",
  will: "will",
};

export const ACTOR_METRIC_DISCOVERY_NAMESPACES = [
  {
    prefix: "ability.",
    description: "Ability modifiers such as ability.int.mod or ability.cha.mod.",
  },
  {
    prefix: "perception.",
    description: "Perception modifier metrics such as perception.mod.",
  },
  {
    prefix: "save.",
    description: "Saving throw metrics such as save.fort.mod, save.best, and save.worst.",
  },
  {
    prefix: "skill.",
    description: "Skill metrics such as skill.arcana.mod, skill.arcana.rank, and skill.arcana.proficient.",
  },
] as const;

export function slugifyActorMetricSegment(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .replace(/_+/g, "_");
}

export function normalizeActorMetricKey(metric: string): string {
  return metric
    .split(".")
    .map((segment) => slugifyActorMetricSegment(segment))
    .filter(Boolean)
    .join(".");
}

export function normalizeActorMetricPrefix(prefix: string): string {
  const normalized = normalizeActorMetricKey(prefix);
  if (!normalized) {
    return "";
  }

  return normalized.endsWith(".") ? normalized : `${normalized}.`;
}

export function normalizeActorMetricTextValue(value: string): string {
  return normalizeText(value).replace(/\s+/g, "_");
}

export function normalizeActorMetricBooleanValue(value: boolean): boolean {
  return value;
}

export function normalizeRawSaveKey(value: string): (typeof ACTOR_SAVE_KEYS)[number] | null {
  return RAW_SAVE_KEY_MAP[slugifyActorMetricSegment(value)] ?? null;
}

export function inferActorMetricValueType(metric: string): ActorMetricValueType | null {
  const normalized = normalizeActorMetricKey(metric);

  if (/^ability\.(str|dex|con|int|wis|cha)\.mod$/.test(normalized)) {
    return "number";
  }

  if (normalized === "perception.mod") {
    return "number";
  }

  if (/^save\.(fort|ref|will)\.mod$/.test(normalized)) {
    return "number";
  }

  if (/^save\.(best|worst)$/.test(normalized)) {
    return "text";
  }

  if (/^skill\.[a-z0-9_]+\.(mod|rank)$/.test(normalized)) {
    return "number";
  }

  if (/^skill\.[a-z0-9_]+\.proficient$/.test(normalized)) {
    return "boolean";
  }

  return null;
}

export function isActorMetricNumericOperator(op: string): op is ActorMetricNumericOperator {
  return ACTOR_METRIC_NUMERIC_OPERATORS.includes(op as ActorMetricNumericOperator);
}

export function isActorMetricScalarOperator(op: string): op is ActorMetricScalarOperator {
  return ACTOR_METRIC_SCALAR_OPERATORS.includes(op as ActorMetricScalarOperator);
}
