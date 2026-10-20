import { Form, InputNumber, Select, Tag } from "antd";
import type {
  EncounterActionTargetView,
  EncounterTargetingRequirementsView,
} from "../../generated/atlas";
const booleanFields = [
  { key: "vision_only_precise", label: "Vision is the only usable precise sense" },
  { key: "guessed_location_correct", label: "GM: guessed location is correct" },
  { key: "visual_effect", label: "Visual effect" },
  { key: "auditory_effect", label: "Auditory effect" },
] as const;
export function TargetContextFields({
  targeting,
  target,
  onChange,
}: {
  targeting: EncounterTargetingRequirementsView | null | undefined;
  target: EncounterActionTargetView | null;
  onChange: (target: EncounterActionTargetView) => void;
}) {
  if (!target || !targeting) return null;
  return (
    <>
      {targeting.required_facts.length > 0 && (
        <p>{targeting.required_facts.join(" · ")}</p>
      )}
      {targeting.visual_effects_immune && <Tag>Target immune to visual effects</Tag>}
      {targeting.auditory_effects_immune && (
        <Tag>Target immune to auditory effects</Tag>
      )}
      {targeting.prevents_effect && <Tag>No effect on target</Tag>}
      {booleanFields
        .filter((field) => targeting.required_inputs.includes(field.key))
        .map(({ key, label }) => (
          <Form.Item key={key} label={label}>
            <Select
              aria-label={label}
              value={target[key] === null ? "unknown" : target[key] ? "yes" : "no"}
              options={[
                { value: "unknown", label: "Unspecified" },
                { value: "yes", label: "Yes" },
                { value: "no", label: "No" },
              ]}
              onChange={(value) =>
                onChange({
                  ...target,
                  [key]: value === "unknown" ? null : value === "yes",
                })
              }
            />
          </Form.Item>
        ))}
      {targeting.required_inputs.includes("distance_feet") && (
        <Form.Item label="Distance (feet)">
          <InputNumber
            aria-label="Target distance in feet"
            min={0}
            value={target.distance_feet}
            onChange={(value) => onChange({ ...target, distance_feet: value })}
          />
        </Form.Item>
      )}
    </>
  );
}
