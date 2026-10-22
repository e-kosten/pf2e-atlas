import { Button, Select, Space } from "antd";
import { Pencil } from "lucide-react";
import { useState } from "react";
import type {
  EncounterParticipantView,
  HpEventContextView,
} from "../../generated/atlas";

export function EncounterKnockout({
  participant,
  participants,
  currentTurnKey,
  initialContext,
  initiativeOnly = false,
  disabled,
  onApply,
}: {
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  currentTurnKey?: string | null;
  initialContext?: HpEventContextView;
  initiativeOnly?: boolean;
  disabled: boolean;
  onApply: (context: HpEventContextView) => Promise<void>;
}) {
  const options = participant.knockout_options;
  const [selected, setSelected] = useState(0);
  const [target, setTarget] = useState(
    initialContext?.keep_current_initiative
      ? ""
      : (initialContext?.responsible_participant_key ??
          (currentTurnKey === participant.participant_key ? "" : currentTurnKey) ??
          ""),
  );
  const [editing, setEditing] = useState(false);
  const option = options[selected] ?? options[0];
  return (
    <div className="encounter-knockout">
      {!initiativeOnly && (
        <>
          <Select
            aria-label="Knockout outcome"
            disabled={disabled}
            value={selected}
            onChange={setSelected}
            options={options.map((entry, index) => ({
              value: index,
              label: entry.label,
            }))}
          />
          <span>{option?.result}</span>
        </>
      )}
      <div className="encounter-knockout__initiative">
        {editing ? (
          <Select
            aria-label="Initiative placement"
            disabled={disabled}
            value={target}
            onChange={setTarget}
            options={[
              { value: "", label: "Keep current position" },
              ...participants
                .filter(
                  (p) =>
                    p.initiative !== undefined &&
                    p.participant_key !== participant.participant_key,
                )
                .map((p) => ({
                  value: p.participant_key,
                  label: `Before ${p.display_name}`,
                })),
            ]}
          />
        ) : (
          <>
            <span>
              Initiative ·{" "}
              {target
                ? `Before ${participants.find((p) => p.participant_key === target)?.display_name ?? "selected turn"}`
                : "Keep current position"}
            </span>
            <Button
              size="small"
              type="text"
              icon={<Pencil size={14} />}
              aria-label="Edit initiative placement"
              disabled={disabled}
              onClick={() => setEditing(true)}
            />
          </>
        )}
      </div>
      <Space className="encounter-knockout__actions" align="center">
        <Button
          size="small"
          type="primary"
          disabled={disabled || (!initiativeOnly && !option)}
          onClick={() =>
            void onApply({
              ...(option?.context ?? initialContext!),
              ...Object.fromEntries(
                Object.entries(initialContext ?? {}).filter(
                  ([, value]) => value !== null,
                ),
              ),
              keep_current_initiative: !target,
              responsible_participant_key: target || null,
            })
          }
        >
          Apply
        </Button>
      </Space>
    </div>
  );
}
