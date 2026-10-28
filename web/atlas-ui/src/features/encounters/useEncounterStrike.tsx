import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { queryOptions, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Select, Spin } from "antd";
import { useState } from "react";
import { previewEncounterStrikeContext } from "../../api/atlasApi";
import type {
  EncounterParticipantView,
  EncounterStrikeOptionsRequest,
} from "../../generated/atlas";
import { FlatCheckRoll } from "../../shared/dice/FlatCheckRoll";
import { useRetainedRecord } from "../../shared/records/useRetainedRecord";
import { isArtifactGenerationChanged } from "../../shared/queries/immutableReferenceQueries";
import { actionIssueSourceLabel } from "./actionIssueLabels";
import type { useEncounterMutations } from "./useEncounterMutations";
const checkIdentity = (check: unknown) =>
  JSON.stringify(check, (_key, value) =>
    typeof value === "bigint" ? value.toString() : value,
  );

export type StrikeLaunch = {
  strike: EncounterStrikeOptionsRequest;
  attackNumber: number;
  rollId: string;
  rolledModifier?: number | null;
};

function strikeContextQuery(
  encounterRef: string,
  revision: bigint,
  generation: string | null,
  strike: EncounterStrikeOptionsRequest | null,
  targetKey?: string,
) {
  return queryOptions({
    queryKey: [
      "strike-assistance",
      encounterRef,
      revision.toString(),
      generation,
      strike,
      targetKey,
    ],
    queryFn: () =>
      previewEncounterStrikeContext(encounterRef, {
        strike: strike!,
        target_key: targetKey ?? null,
      }),
    enabled: !!strike,
    staleTime: Infinity,
    retry: false,
    refetchOnWindowFocus: false,
  });
}

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
  const client = useQueryClient();
  const preview = useQuery(
    strikeContextQuery(
      encounterRef,
      revision,
      generation,
      launch?.strike ?? null,
      targetKey,
    ),
  );
  // A new revision updates the same target. A different target, strike or source
  // cannot borrow checks or optional dice results from the previous subject.
  const identity = JSON.stringify([
    encounterRef,
    generation,
    launch?.strike,
    launch?.rollId,
    targetKey,
  ]);
  const retained = useRetainedRecord(identity, preview.data, preview.error);
  const generationIdentity = JSON.stringify([encounterRef, generation]);
  const [retired, setRetired] = useState<{ identity: string; error: Error }>();
  if (
    isArtifactGenerationChanged(preview.error) &&
    retired?.identity !== generationIdentity
  )
    setRetired({ identity: generationIdentity, error: preview.error! });
  const generationError =
    retired?.identity === generationIdentity ? retired.error : null;
  const displayed = generationError ? undefined : retained;
  const session = JSON.stringify([encounterRef, generation, launch?.rollId]);
  const [readySession, setReadySession] = useState<string>();
  if ((displayed || generationError || preview.error) && readySession !== session)
    setReadySession(session);
  const blocked =
    mutations.pending ||
    !!mutations.refreshError ||
    preview.isFetching ||
    !!preview.error ||
    !!generationError ||
    !preview.data;
  const action = displayed?.action;
  const targeting = action?.targeting;
  const currentModifier = displayed?.options.attacks.find(
    (a) => a.attack_number === launch?.attackNumber,
  )?.modifier;
  const content = (
    <div className="strike-assistance" aria-busy={preview.isFetching}>
      {preview.isFetching && (
        <div className="detail-refresh" aria-label="Updating attack context">
          <Spin size="small" delay={500} />
        </div>
      )}
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
      {(generationError || preview.error) && (
        <ErrorNotice
          error={generationError ?? preview.error}
          compact
          title={
            generationError
              ? "Reference changed. Reload the encounter."
              : "Could not load attack context"
          }
          description={displayed ? "Previous checks shown." : undefined}
          action={
            generationError ? (
              <Button onClick={() => window.location.reload()}>Reload encounter</Button>
            ) : (
              <Button onClick={() => void preview.refetch()}>Retry</Button>
            )
          }
        />
      )}
      {displayed && (
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
          {action?.confused && <p>Confused · Choose the attack target at the table.</p>}
          {action?.disruption && (
            <FlatCheckRoll
              key={`disruption:${identity}:${checkIdentity(action.disruption)}`}
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
          {displayed.targeting_check && (
            <FlatCheckRoll
              key={`target:${identity}:${checkIdentity(displayed.targeting_check)}`}
              dc={displayed.targeting_check.dc}
              label={
                displayed.targeting_check.sources
                  .map((s) => s.name)
                  .filter(Boolean)
                  .join(", ") || "Check detection"
              }
              disabled={blocked}
            />
          )}
          {targeting?.check_senses && (
            <p>Check detection · Consider usable senses and range.</p>
          )}
          {displayed.options.unavailable && <p>{displayed.options.unavailable}</p>}
          {!action?.issues.length &&
            !action?.confused &&
            !action?.disruption &&
            !targeting?.prevents_effect &&
            !targeting?.requires_guessed_location &&
            !targeting?.check_senses &&
            !displayed.targeting_check &&
            !displayed.options.unavailable && (
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
    prepare: (request: StrikeLaunch) => {
      void client.prefetchQuery(
        strikeContextQuery(encounterRef, revision, generation, request.strike),
      );
    },
    launch: (request: StrikeLaunch) => {
      if (request.rollId === launch?.rollId) return;
      setTargetKey(undefined);
      setLaunch(request);
    },
    renderContext: (requestId: string) =>
      launch?.rollId === requestId && readySession === session ? content : null,
  };
}
