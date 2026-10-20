import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Form, Select, Spin } from "antd";
import { useState } from "react";
import { previewEncounterStrikeAttempt } from "../../api/atlasApi";
import type {
  EncounterActionTargetView,
  EncounterParticipantView,
  EncounterStrikeOptionsRequest,
  RollEncounterStrikeRequest,
} from "../../generated/atlas";
import { FlatCheckRoll } from "../../shared/dice/FlatCheckRoll";
import { actionIssueSourceLabel } from "./actionIssueLabels";
import type { useEncounterMutations } from "./useEncounterMutations";
export type StrikeLaunch = {
  strike: EncounterStrikeOptionsRequest;
  attackNumber: number;
  existingRoll: RollEncounterStrikeRequest;
  rolledModifier?: number | null;
};

export function useEncounterStrike({
  encounterRef,
  revision,
  generation,
  participants,
  mutations,
}: {
  encounterRef: string;
  revision: bigint;
  generation: string | null;
  participants: EncounterParticipantView[];
  mutations: ReturnType<typeof useEncounterMutations>;
}) {
  const [launch, setLaunch] = useState<StrikeLaunch | null>(null);
  const [targetKey, setTargetKey] = useState<string>();
  const target: EncounterActionTargetView | null = targetKey
    ? {
        participant_key: targetKey,
        mode: "direct",
        vision_only_precise: null,
        distance_feet: null,
        guessed_location_correct: null,
        visual_effect: null,
        auditory_effect: null,
      }
    : null;
  const preview = useQuery({
    queryKey: [
      "strike-assistance",
      encounterRef,
      revision.toString(),
      generation,
      launch?.strike,
      targetKey,
    ],
    queryFn: () =>
      previewEncounterStrikeAttempt(encounterRef, {
        strike: launch!.strike,
        judgments: [],
        target,
        confusion: null,
      }),
    enabled: !!launch,
  });
  const blocked = mutations.pending || !!mutations.refreshError || preview.isFetching;
  const action = preview.data?.action;
  const targeting = action?.targeting;
  const currentModifier = preview.data?.options.attacks.find(
    (a) => a.attack_number === launch?.attackNumber,
  )?.modifier;
  const content = (
    <div className="strike-assistance">
      <Form layout="vertical" className="strike-assistance__target">
        <Form.Item label="Target" style={{ marginBottom: 0 }}>
          <Select
            allowClear
            aria-label="Strike target"
            placeholder="Choose a target"
            disabled={mutations.pending}
            value={targetKey}
            options={participants
              .filter((p) => p.participant_key !== launch?.strike.participant_key)
              .map((p) => ({ value: p.participant_key, label: p.display_name }))}
            onChange={setTargetKey}
          />
        </Form.Item>
      </Form>
      {preview.isPending && <Spin />}
      {preview.error && (
        <Alert
          type="error"
          title="Could not load attack context"
          action={<Button onClick={() => void preview.refetch()}>Retry</Button>}
        />
      )}
      {preview.data && !preview.isFetching && (
        <>
          {launch?.rolledModifier != null &&
            currentModifier != null &&
            currentModifier !== launch.rolledModifier && (
              <p>
                Current modifier {currentModifier >= 0 ? "+" : ""}
                {currentModifier} · Rolled with {launch.rolledModifier >= 0 ? "+" : ""}
                {launch.rolledModifier}
              </p>
            )}
          {action?.issues.map((issue, i) => (
            <p key={i}>
              {actionIssueSourceLabel(issue, participants)} · {issue.reason}
            </p>
          ))}
          {action?.confusion && (
            <p>Confused · Choose the attack target at the table.</p>
          )}
          {action?.disruption && (
            <FlatCheckRoll
              key={`disruption:${revision}`}
              dc={action.disruption.dc}
              label={
                action.disruption.sources.map((s) => s.name).join(", ") || "Disruption"
              }
              disabled={blocked}
            />
          )}
          {targeting?.prevents_effect && <p>No effect on this target.</p>}
          {targeting?.requires_guessed_location && (
            <p>
              Undetected · Choose a location at the table. The attack misses if the
              target isn’t there.
            </p>
          )}
          {preview.data.targeting_check && (
            <FlatCheckRoll
              key={`target:${targetKey}:${revision}`}
              dc={preview.data.targeting_check.dc}
              label={
                preview.data.targeting_check.sources
                  .map((s) => s.name)
                  .filter(Boolean)
                  .join(", ") || "Check detection"
              }
              disabled={blocked}
            />
          )}
          {targeting?.required_inputs.includes("vision_only_precise") && (
            <p>Check detection · Consider usable senses and range.</p>
          )}
          {preview.data.options.unavailable && (
            <p>{preview.data.options.unavailable}</p>
          )}
          {!action?.issues.length &&
            !action?.confusion &&
            !action?.disruption &&
            !targeting?.prevents_effect &&
            !targeting?.requires_guessed_location &&
            !targeting?.required_inputs.includes("vision_only_precise") &&
            !preview.data.targeting_check &&
            !preview.data.options.unavailable && (
              <p>
                {targetKey
                  ? "No additional checks from tracked conditions."
                  : "Choose a target to check detection."}
              </p>
            )}
        </>
      )}
    </div>
  );
  return {
    launch: (request: StrikeLaunch) => {
      if (request.existingRoll.request_id === launch?.existingRoll.request_id) return;
      setTargetKey(undefined);
      setLaunch(request);
    },
    renderContext: (requestId: string) =>
      launch?.existingRoll.request_id === requestId ? content : null,
  };
}
