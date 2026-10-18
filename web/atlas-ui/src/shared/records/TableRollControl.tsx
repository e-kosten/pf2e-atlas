import { Alert, Button, Space } from "antd";
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
  onResult,
}: {
  selection: RecordDetailRequest;
  artifactGeneration: string;
  formula: string | null;
  onResult: (result: TableRollView | undefined) => void;
}) {
  const roll = useDiceRoll<TableRollView>();
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
          {roll.result ? "Reroll" : "Roll"}
        </Button>
        {roll.result && (
          <RollResult outcome={roll.result.outcome} pending={roll.animating} />
        )}
      </Space>
      {roll.error && <Alert type="error" title={roll.error} />}
      {roll.result && !roll.animating && (
        <>
          {roll.result.invalid_ranges.length > 0 && (
            <Alert
              type="warning"
              title="Some result ranges are unavailable. Matches may be incomplete."
            />
          )}
          {roll.result.matches.length === 0 && (
            <Alert type="info" title="No result matches this roll." />
          )}
        </>
      )}
    </Space>
  );
}
