import { Select } from "antd";
import type { FilterEditorFieldView } from "../../generated/atlas";
import { actorMetric, type ActorMetricKind } from "./actorMetricFields";

/** Entity/statistic selection changes only the rule's concrete catalog field. */
export function ActorMetricPicker({
  kind,
  fields,
  value,
  disabled,
  onChange,
}: {
  kind: ActorMetricKind;
  fields: FilterEditorFieldView[];
  value: string;
  disabled?: boolean;
  onChange: (field: string) => void;
}) {
  const current = actorMetric(value);
  const candidates = fields.flatMap((field) => {
    const metric = actorMetric(field.definition.id);
    return metric?.kind === kind ? [{ field, metric }] : [];
  });
  const entities = [...new Set(candidates.map(({ metric }) => metric.entity))];
  const measure =
    current?.measure ??
    (candidates.some(({ metric }) => metric.measure === "modifier")
      ? "modifier"
      : "rank");
  return (
    <>
      <Select
        virtual={false}
        aria-label={kind === "ability" ? "Ability" : "Skill"}
        placeholder={kind === "ability" ? "Choose an ability…" : "Choose a skill…"}
        showSearch={{ optionFilterProp: "label" }}
        disabled={disabled}
        value={current?.entity}
        options={entities.map((entity) => ({
          value: entity,
          label: candidates
            .find(({ metric }) => metric.entity === entity)!
            .field.label.replace(/ (Modifier|Rank)$/, ""),
        }))}
        onChange={(entity) => {
          const candidate =
            candidates.find(
              ({ metric }) => metric.entity === entity && metric.measure === measure,
            ) ?? candidates.find(({ metric }) => metric.entity === entity);
          if (candidate) onChange(candidate.field.definition.id);
        }}
      />
      {kind === "skill" && current && (
        <Select
          virtual={false}
          aria-label="Skill statistic"
          disabled={disabled}
          value={measure}
          options={candidates
            .filter(({ metric }) => metric.entity === current.entity)
            .map(({ metric }) => ({
              value: metric.measure,
              label: metric.measure === "modifier" ? "Modifier" : "Proficiency rank",
            }))}
          onChange={(next) => {
            const candidate = candidates.find(
              ({ metric }) =>
                metric.entity === current.entity && metric.measure === next,
            );
            if (candidate) onChange(candidate.field.definition.id);
          }}
        />
      )}
    </>
  );
}
