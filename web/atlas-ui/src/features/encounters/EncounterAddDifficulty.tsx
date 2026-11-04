import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { isSharedFailure } from "../../shared/feedback/errorMessages";
import { useQuery } from "@tanstack/react-query";
import { Button, Form, theme } from "antd";
import type { FormInstance } from "antd";
import { useEffect, useMemo, useState } from "react";
import type {
  EncounterDifficultyAddition,
  EncounterParticipantVariantView,
} from "../../generated/atlas";
import {
  difficultySummary,
  type EncounterDifficultyProps,
} from "./EncounterDifficulty";
import { EncounterDifficultyMeter } from "./EncounterDifficultyMeter";

export type EncounterRecordAddForm = {
  quantity?: number | null;
  adjustment?: EncounterParticipantVariantView;
};

export function encounterRecordAddition(
  recordKey: string,
  creature: boolean,
  form: FormInstance<EncounterRecordAddForm>,
): EncounterDifficultyAddition {
  const quantity = form.getFieldValue("quantity");
  return {
    record_ref: recordKey,
    quantity: quantity === undefined ? 1 : (quantity ?? Number.NaN),
    participant_variant:
      creature && form.isFieldTouched("adjustment")
        ? (form.getFieldValue("adjustment") ?? null)
        : null,
  };
}

/** Footer-only projection: no reference editor and no speculative saved participants. */
export function EncounterAddDifficulty({
  encounterKey,
  revision,
  artifactGeneration,
  value,
  onPreview,
  form,
  selectedRecordKey,
  creature,
  recordName,
  disabled,
}: Pick<
  EncounterDifficultyProps,
  | "encounterKey"
  | "revision"
  | "value"
  | "onPreview"
  | "disabled"
  | "artifactGeneration"
> & {
  form: FormInstance<EncounterRecordAddForm>;
  selectedRecordKey: string | null;
  creature: boolean;
  recordName: string;
}) {
  const { token } = theme.useToken();
  // Subscribe to field edits, then read the same form snapshot as submission.
  Form.useWatch("quantity", form);
  Form.useWatch("adjustment", form);
  const fields = selectedRecordKey
    ? encounterRecordAddition(selectedRecordKey, creature, form)
    : null;
  const recordRef = fields?.record_ref;
  const quantity = fields?.quantity;
  const variant = fields?.participant_variant;
  const addition = useMemo<EncounterDifficultyAddition | null>(
    () =>
      recordRef && quantity !== undefined
        ? {
            record_ref: recordRef,
            quantity,
            participant_variant: variant ?? null,
          }
        : null,
    [recordRef, quantity, variant],
  );
  const valid =
    addition !== null &&
    Number.isInteger(addition.quantity) &&
    addition.quantity >= 1 &&
    addition.quantity <= 50;
  const [debounced, setDebounced] = useState(addition);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(addition), 125);
    return () => clearTimeout(timer);
  }, [addition]);
  // A different selection starts immediately; only edits to the same candidate
  // wait for typing to settle. The query key still owns each exact projection.
  const requested =
    addition?.record_ref !== debounced?.record_ref ? addition : debounced;
  const settled = JSON.stringify(addition) === JSON.stringify(requested);
  const preview = useQuery({
    meta: referenceQueryMeta,
    queryKey: [
      "encounter-add-difficulty",
      encounterKey,
      revision.toString(),
      value.reference,
      requested,
      artifactGeneration,
    ],
    queryFn: async () => ({
      value: await onPreview({
        expected_revision: revision,
        reference: value.reference,
        addition: requested,
      }),
      addition: requested!,
      name: recordName,
    }),
    enabled: valid && settled && !disabled,
    retry: false,
  });
  // The last what-if belongs to this encounter basis, with its own candidate
  // identity. Keep that complete scenario while another is requested; never
  // relabel its values as the new selection or carry it across a saved revision.
  const identity = JSON.stringify([
    encounterKey,
    revision.toString(),
    artifactGeneration,
    value.reference,
  ]);
  const [retained, setRetained] = useState<{
    identity: string;
    data: typeof preview.data;
  }>();
  const received = valid && settled ? preview.data : undefined;
  if (received && (retained?.identity !== identity || retained.data !== received))
    setRetained({ identity, data: received });
  const shown = selectedRecordKey
    ? (received ?? (retained?.identity === identity ? retained.data : undefined))
    : undefined;
  const display = shown?.value ?? value;
  const error = valid && settled ? preview.error : null;
  const pending = valid && (!settled || preview.isFetching);
  const requestIdentity = JSON.stringify([identity, addition]);
  const [feedback, setFeedback] = useState({
    identity: requestIdentity,
    pending,
    slow: false,
  });
  if (feedback.identity !== requestIdentity || feedback.pending !== pending)
    setFeedback({ identity: requestIdentity, pending, slow: false });
  useEffect(() => {
    // Typing debounce is not a slow request. Keep the entire last scenario
    // unchanged for quick replacements, including its caption.
    if (!pending || !settled) return;
    const timer = setTimeout(
      () =>
        setFeedback((current) =>
          current.identity === requestIdentity && current.pending
            ? { ...current, slow: true }
            : current,
        ),
      200,
    );
    return () => clearTimeout(timer);
  }, [pending, requestIdentity, settled]);
  const slowPending = pending && feedback.identity === requestIdentity && feedback.slow;
  const scenario = shown ? `${shown.addition.quantity} × ${shown.name}` : null;
  const context = error
    ? `Update failed · ${scenario ? `previous: ${scenario}` : "current encounter"}`
    : addition && !valid
      ? "Enter quantity 1–50 · previous preview"
      : slowPending
        ? "Updating…"
        : scenario
          ? `After adding ${scenario}`
          : "Current encounter";
  return (
    <section
      className="encounter-add-difficulty"
      aria-label="Encounter difficulty preview"
      aria-busy={pending}
    >
      <div
        className="encounter-add-difficulty__heading"
        role="status"
        style={{ minHeight: token.controlHeightSM }}
      >
        <span>
          <span>Current encounter</span> · <strong>{difficultySummary(value)}</strong>
        </span>
        {(!selectedRecordKey || shown || pending || error || (addition && !valid)) && (
          <span className="encounter-add-difficulty__context">
            <span title={context}>
              {selectedRecordKey ? context : "No creature or hazard selected"}
            </span>
            {shown && (
              <>
                {" "}
                · <strong>{difficultySummary(display)}</strong>
              </>
            )}
          </span>
        )}
        {error && !isSharedFailure(error) && (
          <Button size="small" onClick={() => void preview.refetch()}>
            Retry
          </Button>
        )}
      </div>
      <EncounterDifficultyMeter
        value={display.total_xp == null ? value : display}
        current={value}
        compact
      />
    </section>
  );
}
