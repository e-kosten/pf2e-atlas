import { Form, Input, Select } from "antd";
import type {
  EncounterConfusionContextView,
  EncounterConfusionRequirementsView,
  EncounterParticipantView,
} from "../../generated/atlas";
export function ConfusionContextFields({
  requirements,
  value,
  participants,
  onChange,
}: {
  requirements: EncounterConfusionRequirementsView | null | undefined;
  value: EncounterConfusionContextView | null;
  participants: EncounterParticipantView[];
  onChange: (value: EncounterConfusionContextView | null) => void;
}) {
  if (!requirements) return null;
  const label = (key: string) =>
    participants.find((participant) => participant.participant_key === key)
      ?.display_name ?? key;
  return (
    <>
      <Form.Item label="Confused action">
        <Select
          aria-label="Confused action"
          placeholder="Choose the action at the table"
          value={value?.kind}
          options={[
            { value: "offensive", label: "Offensive action" },
            { value: "facilitate", label: "Facilitate future attacks" },
            { value: "no_viable_targets", label: "No viable targets" },
            { value: "unable_to_attack", label: "Unable to attack" },
          ]}
          onChange={(kind) =>
            onChange(
              kind === "offensive"
                ? {
                    kind,
                    viable_target_keys: [],
                    random_target_key: null,
                    offensive: true,
                  }
                : kind === "facilitate"
                  ? { kind, note: null }
                  : { kind },
            )
          }
        />
      </Form.Item>
      {value?.kind === "offensive" && (
        <>
          <Form.Item label="Viable targets">
            <Select
              mode="multiple"
              aria-label="Viable targets"
              value={value.viable_target_keys}
              options={participants.map((participant) => ({
                value: participant.participant_key,
                label: participant.display_name,
              }))}
              onChange={(keys) =>
                onChange({
                  ...value,
                  viable_target_keys: keys,
                  random_target_key: keys.includes(value.random_target_key ?? "")
                    ? value.random_target_key
                    : null,
                })
              }
            />
          </Form.Item>
          <Form.Item label="Random target (table result)">
            <Select
              aria-label="Random target (table result)"
              value={value.random_target_key ?? undefined}
              options={value.viable_target_keys.map((key) => ({
                value: key,
                label: label(key),
              }))}
              onChange={(key) => onChange({ ...value, random_target_key: key })}
            />
          </Form.Item>
        </>
      )}
      {value?.kind === "facilitate" && (
        <Form.Item label="Action detail">
          <Input
            aria-label="Confused action detail"
            value={value.note ?? ""}
            onChange={(event) =>
              onChange({ ...value, note: event.target.value || null })
            }
          />
        </Form.Item>
      )}
      {requirements.required_facts.length > 0 && (
        <p>{requirements.required_facts.join(" · ")}</p>
      )}
      {requirements.selected_target_key && (
        <p>Target: {label(requirements.selected_target_key)}</p>
      )}
      {requirements.automatic_self_hit && <p>Automatic self-hit · Noncritical</p>}
      {requirements.wastes_action && <p>Waste the action babbling</p>}
    </>
  );
}
