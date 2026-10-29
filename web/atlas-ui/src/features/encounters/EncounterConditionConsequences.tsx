import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { userErrorMessage } from "../../shared/feedback/errorMessages";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useQuery } from "@tanstack/react-query";
import { Button, Checkbox, InputNumber, Popover, Space, Spin } from "antd";
import { useEffect, useState } from "react";
import { previewEncounterDrainedConsequence } from "../../api/atlasApi";
import type {
  PendingConditionConsequenceView,
  ResolveDrainedConsequenceRequest,
} from "../../generated/atlas";
import { ConditionHpPreview } from "./ConditionHpPreview";
import { useRetainedRecord } from "../../shared/records/useRetainedRecord";
export function EncounterConditionConsequences({
  encounterRef,
  row,
  encounterRevision,
  level,
  disabled,
  pending,
  onResolve,
}: {
  encounterRef: string;
  row: PendingConditionConsequenceView;
  encounterRevision: bigint;
  level?: bigint;
  disabled: boolean;
  pending: boolean;
  onResolve: (
    request: Omit<ResolveDrainedConsequenceRequest, "request_id">,
  ) => Promise<boolean>;
}) {
  const [open, setOpen] = useState(false),
    [input, setInput] = useState<number | null>(
      level === undefined ? null : Number(level),
    );
  const [settled, setSettled] = useState<number | null>(input),
    [applyLoss, setApplyLoss] = useState(true),
    [error, setError] = useState<string>();
  useEffect(() => {
    const timer = setTimeout(() => setSettled(input), 150);
    return () => clearTimeout(timer);
  }, [input]);
  const valid = settled !== null && Number.isSafeInteger(settled);
  const request = {
    consequence_key: row.consequence_key,
    expected_encounter_revision: encounterRevision,
    level: BigInt(settled ?? 0),
    apply_hp_loss: applyLoss,
  };
  const preview = useQuery({
    meta: referenceQueryMeta,
    queryKey: [
      "drained-preview",
      encounterRef,
      row.consequence_key,
      encounterRevision.toString(),
      settled,
      applyLoss,
    ],
    queryFn: () =>
      previewEncounterDrainedConsequence(encounterRef, {
        ...request,
        request_id: "preview",
      }),
    enabled: open && valid,
    retry: false,
  });
  const displayed = useRetainedRecord(
    JSON.stringify([encounterRef, row.consequence_key]),
    preview.data,
    preview.error,
  );
  return (
    <Popover
      title="Drained"
      trigger="click"
      placement="bottom"
      open={open}
      onOpenChange={setOpen}
      content={
        <div className="encounter-condition-change">
          {preview.isFetching && (
            <div className="detail-refresh" aria-label="Updating HP change">
              <Spin size="small" delay={500} />
            </div>
          )}
          <label>
            Level{" "}
            <InputNumber
              aria-label="Level for Drained"
              min={-1}
              max={Number.MAX_SAFE_INTEGER}
              precision={0}
              value={input}
              onChange={setInput}
            />
          </label>
          <Checkbox
            checked={applyLoss}
            onChange={(e) => setApplyLoss(e.target.checked)}
          >
            Reduce current HP
          </Checkbox>
          {displayed && <ConditionHpPreview change={displayed} />}
          {(error || preview.error) && (
            <ErrorNotice
              error={preview.error ?? error}
              title={error || "HP change unavailable"}
              description={!error && displayed ? "Previous preview shown." : undefined}
              action={
                !error ? (
                  <Button size="small" onClick={() => void preview.refetch()}>
                    Retry
                  </Button>
                ) : undefined
              }
            />
          )}
          <Space>
            <Button
              size="small"
              type="primary"
              disabled={
                disabled ||
                pending ||
                !valid ||
                input !== settled ||
                preview.isFetching ||
                !preview.data ||
                !!preview.error
              }
              onClick={async () => {
                setError(undefined);
                try {
                  await onResolve(request);
                  setOpen(false);
                } catch (e) {
                  setError(userErrorMessage(e, "Could not apply HP loss."));
                }
              }}
            >
              Apply
            </Button>
          </Space>
        </div>
      }
    >
      <Button size="small" type="link" disabled={disabled || pending}>
        {level === undefined ? "Set level" : "HP loss"}
      </Button>
    </Popover>
  );
}
