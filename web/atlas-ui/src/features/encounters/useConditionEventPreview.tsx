import { Alert, Button, Checkbox } from "antd";
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type {
  AddEncounterParticipantConditionRequest,
  ConditionEventPreviewView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
export type ConditionEventInput =
  | Omit<AddEncounterParticipantConditionRequest, "request_id">
  | Omit<UpdateEncounterParticipantConditionRequest, "request_id">;
const identityOf = (request: ConditionEventInput) =>
  JSON.stringify(request, (_key, value) =>
    typeof value === "bigint" ? value.toString() : value,
  );
export function useConditionEventPreview(
  participants: EncounterParticipantView[],
  loadPreview: (request: ConditionEventInput) => Promise<ConditionEventPreviewView>,
) {
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
    gcTime: 0,
  });
  const current = enabled ? query.data : undefined;
  const loading = !!draft && (!enabled || query.isFetching);
  const error =
    enabled && query.error
      ? query.error.message || "Could not preview this change."
      : undefined;
  const consequenceIdentity = current
    ? JSON.stringify(current.consequences, (_key, value) =>
        typeof value === "bigint" ? value.toString() : value,
      )
    : undefined;
  const [choice, setChoice] = useState<{
    identity?: string;
    correction?: boolean;
    keepUpright?: boolean;
  }>({});
  if (current && choice.identity !== consequenceIdentity)
    setChoice({ identity: consequenceIdentity });
  const correction = choice.identity === consequenceIdentity && !!choice.correction;
  const keepUpright = choice.identity === consequenceIdentity && !!choice.keepUpright;
  const setCorrection = (value: boolean) =>
    setChoice({ identity: consequenceIdentity, correction: value, keepUpright });
  const setKeepUpright = (value: boolean) =>
    setChoice({ identity: consequenceIdentity, correction, keepUpright: value });
  const onlyHp =
    current?.consequences.length === 1 &&
    current.consequences.every(
      (c) => c.kind === "direct_hp_loss" || c.kind === "drained_level_required",
    );
  const apply = <T extends ConditionEventInput>(request: T): T => {
    if (!current || identityOf(request) !== identity || loading || error)
      throw new Error("Wait for the condition preview.");
    return {
      ...request,
      event_intent: correction ? "correction" : "apply",
      decisions: { ...request.decisions, keep_upright: keepUpright },
    };
  };
  return {
    setDraft: (request: ConditionEventInput) =>
      updateDraft((previous) =>
        previous && identityOf(previous) === identityOf(request) ? previous : request,
      ),
    apply,
    loading,
    error,
    ready: !!current && !loading && !error,
    reset: () => {
      updateDraft(undefined);
      setChoice({});
    },
    review: current ? (
      <div aria-label="Changes on application">
        {current.consequences.map((c, index) => {
          if (c.kind === "become_prone")
            return (
              <p key={index}>
                <Checkbox
                  checked={!keepUpright}
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
