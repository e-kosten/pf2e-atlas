import { ConditionHpPreview } from "./ConditionHpPreview";
import { Alert, Button, Checkbox } from "antd";
import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  AddEncounterParticipantConditionRequest,
  ConditionEventPreviewView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
export type ConditionEventInput =
  | Omit<AddEncounterParticipantConditionRequest, "request_id">
  | Omit<UpdateEncounterParticipantConditionRequest, "request_id">;
// Notes are persisted metadata, not input to application consequences.
const identityOf = (request: ConditionEventInput) =>
  JSON.stringify({ ...request, note: undefined }, (_key, value) =>
    typeof value === "bigint" ? value.toString() : value,
  );
const consequencesOf = (preview: ConditionEventPreviewView) =>
  JSON.stringify(
    {
      consequences: preview.consequences,
      persistent_precedence: preview.persistent_precedence,
    },
    (_key, value) => (typeof value === "bigint" ? value.toString() : value),
  );
export function useConditionEventPreview(
  participants: EncounterParticipantView[],
  loadPreview: (request: ConditionEventInput) => Promise<ConditionEventPreviewView>,
) {
  const client = useQueryClient();
  const [draft, updateDraft] = useState<ConditionEventInput>();
  const identity = draft ? identityOf(draft) : undefined;
  const [settled, setSettled] = useState<string>();
  useEffect(() => {
    const timer = setTimeout(() => setSettled(identity), 150);
    return () => clearTimeout(timer);
  }, [identity]);
  const enabled = identity !== undefined && settled === identity;
  const query = useQuery({
    queryKey: ["condition-event-preview", identity],
    queryFn: () => loadPreview(draft!),
    enabled,
    retry: false,
    placeholderData: (previous, previousQuery) => {
      if (!previous || !previousQuery?.queryKey[1] || !identity) return undefined;
      const prior = JSON.parse(previousQuery.queryKey[1] as string);
      // Retain only the same form subject. A changed counter updates the existing
      // preview; switching condition or participant must not show another effect.
      const next = JSON.parse(identity!);
      return prior.participant_key === next.participant_key &&
        prior.condition_ref === next.condition_ref &&
        prior.condition_id === next.condition_id &&
        prior.expected_encounter_revision === next.expected_encounter_revision
        ? previous
        : undefined;
    },
    gcTime: 0,
  });
  const displayed = draft ? query.data : undefined;
  const current = enabled && !query.isPlaceholderData ? query.data : undefined;
  const loading = !!draft && (!enabled || query.isFetching);
  const error =
    enabled && query.error
      ? query.error.message || "Could not preview this change."
      : undefined;
  const consequenceIdentity = displayed ? consequencesOf(displayed) : undefined;
  const [choice, setChoice] = useState<{
    identity?: string;
    correction?: boolean;
    keepUpright?: boolean;
    persistentOrder?: import("../../generated/atlas").PersistentDamageSourceRefView[];
  }>({});
  if (current && choice.identity !== consequenceIdentity)
    setChoice({ identity: consequenceIdentity });
  const correction = choice.identity === consequenceIdentity && !!choice.correction;
  const keepUpright = choice.identity === consequenceIdentity && !!choice.keepUpright;
  const setCorrection = (value: boolean) =>
    setChoice({
      ...choice,
      identity: consequenceIdentity,
      correction: value,
      keepUpright,
    });
  const setKeepUpright = (value: boolean) =>
    setChoice({
      ...choice,
      identity: consequenceIdentity,
      correction,
      keepUpright: value,
    });
  const persistentOrder =
    choice.identity === consequenceIdentity ? choice.persistentOrder : undefined;
  const onlyHp =
    displayed?.consequences.length === 1 &&
    displayed.consequences.every(
      (c) => c.kind === "direct_hp_loss" || c.kind === "drained_level_required",
    );
  const apply = <T extends ConditionEventInput>(request: T): T => {
    if (!current || identityOf(request) !== identity || loading || error)
      throw new Error("Wait for the condition preview.");
    return {
      ...request,
      event_intent: correction ? "correction" : "apply",
      decisions: {
        ...request.decisions,
        keep_upright: keepUpright,
        persistent_precedence:
          persistentOrder ??
          current.persistent_precedence?.sources.map((entry) => entry.source),
      },
    };
  };
  return {
    setDraft: (request: ConditionEventInput) =>
      updateDraft((previous) =>
        previous && identityOf(previous) === identityOf(request) ? previous : request,
      ),
    apply,
    prepare: async <T extends ConditionEventInput>(
      request: T,
    ): Promise<T | undefined> => {
      if (current && identityOf(request) === identity && !loading && !error)
        return apply(request);
      updateDraft(request);
      setSettled(identityOf(request));
      const result = await client.fetchQuery({
        queryKey: ["condition-event-preview", identityOf(request)],
        queryFn: () => loadPreview(request),
        staleTime: 0,
      });
      // A click during typing can finish ordinary edits immediately. Newly changed
      // consequences must be displayed before the user accepts them.
      if (
        (result.consequences.length || result.persistent_precedence) &&
        consequencesOf(result) !== consequenceIdentity
      )
        return undefined;
      const retainChoices = consequencesOf(result) === consequenceIdentity;
      return {
        ...request,
        event_intent: retainChoices && correction ? "correction" : "apply",
        decisions: {
          ...request.decisions,
          keep_upright: retainChoices && keepUpright,
          persistent_precedence: retainChoices
            ? (persistentOrder ??
              result.persistent_precedence?.sources.map((entry) => entry.source))
            : result.persistent_precedence?.sources.map((entry) => entry.source),
        },
      };
    },
    persistentPrecedence: displayed?.persistent_precedence,
    persistentOrder:
      persistentOrder ??
      displayed?.persistent_precedence?.sources.map((entry) => entry.source),
    setPersistentOrder: (
      order: import("../../generated/atlas").PersistentDamageSourceRefView[],
    ) =>
      setChoice({ ...choice, identity: consequenceIdentity, persistentOrder: order }),
    loading,
    error,
    ready: !!current && !loading && !error,
    reset: () => {
      updateDraft(undefined);
      setChoice({});
    },
    review: displayed ? (
      <div aria-label="Changes on application">
        {displayed.hp_change && (
          <ConditionHpPreview
            change={{
              ...displayed.hp_change,
              current_after: correction
                ? displayed.hp_change.current_before
                : displayed.hp_change.current_after,
            }}
          />
        )}
        {displayed.consequences.map((c, index) => {
          if (c.kind === "become_prone")
            return (
              <p key={index}>
                <Checkbox
                  checked={!keepUpright}
                  disabled={loading || !!error}
                  onChange={(e) => setKeepUpright(!e.target.checked)}
                >
                  Applies Prone
                </Checkbox>
              </p>
            );
          if (onlyHp)
            return (
              <p key={index}>
                <Checkbox
                  checked={!correction}
                  disabled={loading || !!error}
                  onChange={(e) => setCorrection(!e.target.checked)}
                >
                  {c.kind === "direct_hp_loss"
                    ? `Lose ${c.amount} HP`
                    : "Applies HP loss (Level needed)"}
                </Checkbox>
              </p>
            );
          const name =
            participants.find((p) => p.participant_key === c.participant_key)
              ?.display_name ?? c.participant_key;
          return (
            <p key={index}>
              {name}:{" "}
              {c.kind === "direct_hp_loss"
                ? `Lose ${c.amount} HP`
                : c.kind === "drop_held_items"
                  ? "Drop held items"
                  : c.kind === "prone_prevented"
                    ? c.reason
                    : "Level needed to calculate HP loss"}
            </p>
          );
        })}
      </div>
    ) : null,
    errorNotice: error ? (
      <Alert
        type="error"
        title="Could not preview condition"
        description={error}
        action={
          <Button
            onClick={() => {
              void query.refetch();
            }}
          >
            Retry
          </Button>
        }
      />
    ) : null,
  };
}
