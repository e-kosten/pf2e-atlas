/** UI families retain the executable catalog's individual typed fields. */
export const ACTOR_METRIC_CHOOSERS = {
  ability: "atlas:ability",
  skill: "atlas:skill",
} as const;
export type ActorMetricKind = keyof typeof ACTOR_METRIC_CHOOSERS;
export type ActorMetric = {
  kind: ActorMetricKind;
  entity: string;
  measure: "modifier" | "rank";
};
export function actorMetric(id: string): ActorMetric | null {
  const ability = /^abilities\.([a-z]+)\.modifier$/.exec(id);
  if (ability) return { kind: "ability", entity: ability[1], measure: "modifier" };
  const skill = /^skills\.([a-z]+)\.(modifier|rank)$/.exec(id);
  return skill
    ? {
        kind: "skill",
        entity: skill[1],
        measure: skill[2] as "modifier" | "rank",
      }
    : null;
}
export function actorMetricKind(id: string): ActorMetricKind | null {
  return (
    actorMetric(id)?.kind ??
    (id === ACTOR_METRIC_CHOOSERS.ability
      ? "ability"
      : id === ACTOR_METRIC_CHOOSERS.skill
        ? "skill"
        : null)
  );
}
export function isActorMetricChooser(id: string): boolean {
  return Object.values(ACTOR_METRIC_CHOOSERS).some((value) => value === id);
}
export function sameActorMetricMeasure(left: string, right: string): boolean {
  const a = actorMetric(left),
    b = actorMetric(right);
  return !!a && !!b && a.kind === b.kind && a.measure === b.measure;
}
