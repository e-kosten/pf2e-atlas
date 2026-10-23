import { Alert, Button, Popover, Space, Typography } from "antd";
import { Info } from "lucide-react";
import { useEffect } from "react";
import { rollTable } from "../../api/atlasApi";
import type { RecordDetailRequest, TableRollView } from "../../generated/atlas";
import { RollResult } from "../dice/RollResult";
import { useDiceRoll } from "../dice/useDiceRoll";

/** A reference roll. Linked tables are readable references, never additional rolls. */
export function TableRollControl({
  selection,
  artifactGeneration,
  formula,
  rollNote,
  onResult,
}: {
  selection: RecordDetailRequest;
  artifactGeneration: string;
  formula: string | null;
  rollNote: string;
  onResult: (result: TableRollView | undefined) => void;
}) {
  const roll = useDiceRoll<TableRollView>({ retainResultOnError: true });
  const result = roll.result;
  useEffect(() => {
    if (!roll.animating) onResult(roll.result);
  }, [roll.result, roll.animating, onResult]);
  return (
    <Space orientation="vertical" size="small" style={{ width: "100%" }}>
      <Space wrap>
        {formula && <span>{formula}</span>}
        <Button
          disabled={roll.busy}
          loading={roll.busy && !roll.animating ? { delay: 300 } : false}
          onClick={() =>
            void roll.run(() =>
              rollTable({ selection, artifact_generation: artifactGeneration }),
            )
          }
        >
          {result ? "Reroll" : "Roll"}
        </Button>
        <Popover content={rollNote} trigger={["click", "hover", "focus"]}>
          <Button
            type="text"
            aria-label="About table rolls"
            icon={<Info size={16} />}
          />
        </Popover>
        {result && <RollResult outcome={result.outcome} pending={roll.animating} />}
      </Space>
      {roll.error && (
        <Typography.Text type="danger" role="alert">
          {roll.error}
          {result && " Previous result shown."}
        </Typography.Text>
      )}
      {result && !roll.animating && (
        <>
          {result.invalid_ranges.length > 0 && (
            <Alert
              type="warning"
              title="Some result ranges are unavailable. Matches may be incomplete."
            />
          )}
          {result.matches.length === 0 && (
            <Alert
              type="info"
              title={
                result.invalid_ranges.length
                  ? "No valid result range matches this roll."
                  : "No result matches this roll."
              }
            />
          )}
        </>
      )}
    </Space>
  );
}
