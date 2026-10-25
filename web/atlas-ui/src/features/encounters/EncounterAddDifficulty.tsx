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
    const timer = setTimeout(() => setDebounced(addition), 250);
    return () => clearTimeout(timer);
  }, [addition]);
  const settled = JSON.stringify(addition) === JSON.stringify(debounced);
  const preview = useQuery({
    queryKey: [
      "encounter-add-difficulty",
      encounterKey,
      revision.toString(),
      value.reference,
      debounced,
      artifactGeneration,
    ],
    queryFn: async () => ({
      value: await onPreview({
        expected_revision: revision,
        reference: value.reference,
        addition: debounced,
      }),
      quantity: debounced!.quantity,
      name: recordName,
    }),
    enabled: valid && settled && !disabled,
    retry: false,
    // Retain only the same encounter revision/reference and selected source identity.
    // Another selected creature must never inherit this creature's projected XP.
    placeholderData: (previous, previousQuery) => {
      const key = previousQuery?.queryKey;
      const previousAddition = key?.[4] as EncounterDifficultyAddition | undefined;
      return key?.[1] === encounterKey &&
        key?.[2] === revision.toString() &&
        key?.[5] === artifactGeneration &&
        JSON.stringify(key?.[3]) === JSON.stringify(value.reference) &&
        previousAddition?.record_ref === selectedRecordKey
        ? previous
        : undefined;
    },
  });
  const matches = valid && debounced?.record_ref === selectedRecordKey;
  const identity = JSON.stringify([
    encounterKey,
    revision.toString(),
    artifactGeneration,
    value.reference,
    selectedRecordKey,
  ]);
  const [retained, setRetained] = useState<{
    identity: string;
    data: typeof preview.data;
  }>();
  const received = matches ? preview.data : undefined;
  if (received && (retained?.identity !== identity || retained.data !== received))
    setRetained({ identity, data: received });
  const shown = valid
    ? (received ?? (retained?.identity === identity ? retained.data : undefined))
    : undefined;
  const display = shown?.value ?? value;
  const error = valid && settled && matches ? preview.error : null;
  const pending = valid && (!settled || preview.isFetching);
  const context = shown
    ? `After adding ${shown.quantity} × ${shown.name}`
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
          {error
            ? `Update failed · ${shown ? "previous preview" : "current encounter"}`
            : pending
              ? "Updating preview…"
              : addition && !valid
                ? "Enter quantity 1–50"
                : context}
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
