import { ErrorNotice } from "../feedback/ErrorNotice";
import { Button } from "antd";
import { rollFlatCheck } from "../../api/atlasApi";
import type { FlatCheckRollView } from "../../generated/atlas";
import { useDiceRoll } from "./useDiceRoll";
import { RollResult } from "./RollResult";

/** Optional reference dice; never commits an encounter action. */
export function FlatCheckRoll({
  dc,
  label,
  disabled = false,
}: {
  dc: bigint;
  label: string;
  disabled?: boolean;
}) {
  const roll = useDiceRoll<FlatCheckRollView>();
  return (
    <div className="atlas-flat-check">
      <div className="atlas-flat-check__heading">
        <span>
          {label} · DC {dc.toString()} flat check
        </span>
        <Button
          disabled={disabled || roll.busy}
          loading={roll.busy}
          onClick={() => void roll.run(() => rollFlatCheck({ dc }))}
        >
          {roll.result ? "Reroll" : "Roll"}
        </Button>
      </div>
      {roll.error && <ErrorNotice error={roll.error} title={roll.error} />}
      {roll.result && (
        <>
          <RollResult outcome={roll.result.outcome} pending={roll.animating} />
          {!roll.animating && (
            <span>{roll.result.success ? "Success" : "Failure"}</span>
          )}
        </>
      )}
    </div>
  );
}
