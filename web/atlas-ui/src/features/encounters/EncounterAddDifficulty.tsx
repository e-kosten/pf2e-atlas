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
  initiative?: number | null;
  adjustment?: EncounterParticipantVariantView;
};

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
  const quantity = Form.useWatch("quantity", form);
  const adjustment = Form.useWatch("adjustment", form);
  const touched = form.isFieldTouched("adjustment");
  const addition = useMemo<EncounterDifficultyAddition | null>(
    () =>
      selectedRecordKey
        ? {
            record_ref: selectedRecordKey,
            quantity: quantity === undefined ? 1 : (quantity ?? Number.NaN),
            participant_variant: creature && touched ? (adjustment ?? null) : null,
          }
        : null,
    [selectedRecordKey, quantity, creature, touched, adjustment],
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
    if (!pending) return;
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
  }, [pending, requestIdentity]);
  const slowPending = pending && feedback.identity === requestIdentity && feedback.slow;
  const previous = shown && JSON.stringify(shown.addition) !== JSON.stringify(addition);
  const scenario = shown ? `${shown.addition.quantity} × ${shown.name}` : null;
  const context = error
    ? `Update failed · ${scenario ? `previous: ${scenario}` : "current encounter"}`
    : addition && !valid
      ? "Enter quantity 1–50 · previous preview"
      : previous
        ? `Previous: ${scenario}`
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
        <span className="encounter-add-difficulty__context">
          {slowPending ? `Updating · ${context}` : context}
        </span>
        <strong>{difficultySummary(display)}</strong>
        {error && (
          <Button size="small" onClick={() => void preview.refetch()}>
            Retry
          </Button>
        )}
      </div>
      <EncounterDifficultyMeter value={display} compact />
    </section>
  );
}
