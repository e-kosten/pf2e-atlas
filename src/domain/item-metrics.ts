import type {
  ActorMetricNumericOperator,
  ActorMetricScalarOperator,
  ActorMetricValue,
  ActorMetricValueType,
} from "./actor-metrics.js";
import {
  normalizeActorMetricKey,
  normalizeActorMetricPrefix,
  slugifyActorMetricSegment,
} from "./actor-metrics.js";

export const ITEM_METRIC_DISCOVERY_NAMESPACES = [
  {
    prefix: "armor.",
    description: "Armor metrics such as armor.ac_bonus, armor.check_penalty, and armor.speed_penalty.",
  },
  {
    prefix: "shield.",
    description: "Shield metrics such as shield.hardness, shield.hp, and shield.bt.",
  },
  {
    prefix: "weapon.",
    description: "Weapon metrics such as weapon.range_increment, weapon.reload, weapon.damage_dice, and weapon.damage_die_faces.",
  },
] as const;

export type ItemMetricNumericOperator = ActorMetricNumericOperator;
export type ItemMetricScalarOperator = ActorMetricScalarOperator;
export type ItemMetricOperator = ItemMetricNumericOperator;
export type ItemMetricValueType = ActorMetricValueType;
export type ItemMetricValue = ActorMetricValue;
export type ItemMetricMap = Record<string, ItemMetricValue>;

export function normalizeItemMetricKey(metric: string): string {
  return normalizeActorMetricKey(metric);
}

export function normalizeItemMetricPrefix(prefix: string): string {
  return normalizeActorMetricPrefix(prefix);
}

export function slugifyItemMetricSegment(value: string): string {
  return slugifyActorMetricSegment(value);
}

export function inferItemMetricValueType(metric: string): ItemMetricValueType | null {
  const normalized = normalizeItemMetricKey(metric);

  if (/^weapon\.(range_increment|reload|damage_dice|damage_die_faces)$/.test(normalized)) {
    return "number";
  }

  if (/^armor\.(ac_bonus|check_penalty|speed_penalty)$/.test(normalized)) {
    return "number";
  }

  if (/^shield\.(hardness|hp|bt)$/.test(normalized)) {
    return "number";
  }

  return null;
}
