import { Alert, Button, Checkbox } from "antd";
import { useEffect, useEffectEvent, useRef, useState } from "react";
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
  const [result, setResult] = useState<{
    identity: string;
    preview: ConditionEventPreviewView;
  }>();
  const [error, setError] = useState<string>();
  const [retryVersion, setRetryVersion] = useState(0);
  const [loading, setLoading] = useState(false);
  const [correction, setCorrection] = useState(false);
  const [keepUpright, setKeepUpright] = useState(false);
  const consequenceIdentity = useRef<string | undefined>(undefined);
  const load = useEffectEvent((request: ConditionEventInput) => loadPreview(request));
  const identity = draft ? identityOf(draft) : undefined;
  useEffect(() => {
    if (!draft) return;
    let active = true;
    const timer = setTimeout(() => {
      void load(draft)
        .then((preview) => {
          if (active) {
            const nextIdentity = JSON.stringify(preview.consequences, (_key, value) =>
              typeof value === "bigint" ? value.toString() : value,
            );
            if (consequenceIdentity.current !== nextIdentity) {
              setCorrection(false);
              setKeepUpright(false);
              consequenceIdentity.current = nextIdentity;
            }
            setResult({ identity: identity!, preview });
          }
        })
        .catch((cause) => {
          if (active)
            setError(
              cause instanceof Error ? cause.message : "Could not preview this change.",
            );
        })
        .finally(() => {
          if (active) setLoading(false);
        });
    }, 150);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [identity, draft, retryVersion]);
  const current = result && result.identity === identity ? result.preview : undefined;
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
    setDraft: (request: ConditionEventInput) => {
      if (draft && identityOf(draft) === identityOf(request)) return;
      setLoading(true);
      setError(undefined);
      updateDraft(request);
    },
    apply,
    loading,
    error,
    ready: !!current && !loading && !error,
    reset: () => {
      consequenceIdentity.current = undefined;
      setCorrection(false);
      setKeepUpright(false);
      updateDraft(undefined);
      setLoading(false);
      setResult(undefined);
      setError(undefined);
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
              setLoading(true);
              setError(undefined);
              setRetryVersion((version) => version + 1);
            }}
          >
            Retry
          </Button>
        }
      />
    ) : null,
  };
}
