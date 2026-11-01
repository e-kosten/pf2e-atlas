import { Button, Form, InputNumber, Segmented, Select, Space, Tooltip } from "antd";
import type { FormItemProps } from "antd";
import { X } from "lucide-react";
import type { EncounterParticipantView } from "../../generated/atlas";

/** The encounter duration controls shared by conditions and manual trackers. */
export function EncounterDurationFields({
  participants,
  visible = true,
  timed = true,
  amountRules,
  timingRules,
  participantRules = timingRules,
  maximumAmount,
  onUnitChange,
  onRemove,
}: {
  participants: EncounterParticipantView[];
  visible?: boolean;
  timed?: boolean;
  amountRules: FormItemProps["rules"];
  timingRules: FormItemProps["rules"];
  participantRules?: FormItemProps["rules"];
  maximumAmount?: number;
  onUnitChange?: () => void;
  onRemove?: () => void;
}) {
  return (
    <>
      <div style={{ display: visible ? undefined : "none" }}>
        <Form.Item label="Duration">
          <Space.Compact>
            <Form.Item name="durationAmount" noStyle rules={amountRules}>
              <InputNumber
                aria-label="Duration amount"
                min={0}
                max={maximumAmount}
                step={1}
                onKeyDown={(event) => {
                  if (event.key === "-") event.preventDefault();
                }}
                onPaste={(event) => {
                  if (/^\s*[-−]/.test(event.clipboardData.getData("text")))
                    event.preventDefault();
                }}
                style={{ width: 88 }}
              />
            </Form.Item>
            <Form.Item name="durationUnit" noStyle>
              <Select
                aria-label="Duration unit"
                onChange={onUnitChange}
                style={{ width: 128 }}
                options={["rounds", "minutes", "hours", "days"].map((value) => ({
                  value,
                  label: value,
                }))}
              />
            </Form.Item>
            {onRemove && (
              <Tooltip title="Remove duration" trigger={["hover", "focus"]}>
                <Button
                  aria-label="Remove duration"
                  icon={<X size={14} aria-hidden="true" />}
                  onClick={onRemove}
                />
              </Tooltip>
            )}
          </Space.Compact>
        </Form.Item>
      </div>
      {timed && (
        <Space align="start" wrap>
          <Form.Item name="durationEvent" rules={timingRules}>
            <Segmented
              aria-label="Duration boundary"
              options={[
                { label: "Starts on", value: "start" },
                { label: "Ends on", value: "end" },
              ]}
            />
          </Form.Item>
          <Form.Item name="durationParticipant" rules={participantRules}>
            <Select
              aria-label="Whose turn"
              placeholder="Whose turn"
              style={{ minWidth: 180 }}
              options={participants.map((p) => ({
                value: p.participant_key,
                label: `${p.display_name}’s turn`,
              }))}
            />
          </Form.Item>
        </Space>
      )}
    </>
  );
}
