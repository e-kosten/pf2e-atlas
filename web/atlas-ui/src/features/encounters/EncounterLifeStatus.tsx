import { userErrorMessage } from "../../shared/feedback/errorMessages";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { Button, Popover, Space, Tooltip } from "antd";
import { ChevronDown } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { getEncounterHpLifecycle } from "../../api/atlasApi";
import type {
  EncounterParticipantView,
  PendingHpLifecycleView,
  ResolveEncounterHpLifecycleRequest,
  SetEncounterLifeSettingsRequest,
} from "../../generated/atlas";
import { EncounterKnockout } from "./EncounterKnockout";

export function EncounterLifeStatus({
  encounterRef,
  revision,
  participant,
  participants,
  currentTurnKey,
  disabled,
  onSave,
  onDefeated,
  onResolve,
}: {
  encounterRef: string;
  revision: bigint;
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  currentTurnKey?: string | null;
  disabled: boolean;
  onSave: (
    request: Omit<
      SetEncounterLifeSettingsRequest,
      "request_id" | "expected_encounter_revision"
    >,
  ) => Promise<boolean>;
  onDefeated: () => Promise<unknown>;
  onResolve: (
    request: Omit<ResolveEncounterHpLifecycleRequest, "request_id">,
  ) => Promise<boolean>;
}) {
  const [opened, setOpened] = useState<string>(),
    [error, setError] = useState<string>();
  const seen = useRef<Set<string> | undefined>(undefined);
  const query = useQuery({
    queryKey: ["hp-lifecycle", encounterRef, revision.toString()],
    queryFn: () => getEncounterHpLifecycle(encounterRef),
    placeholderData: (previous, previousQuery) =>
      previousQuery?.queryKey[1] === encounterRef ? previous : undefined,
  });
  const rows =
    query.data?.filter((row) => row.participant_key === participant.participant_key) ??
    [];
  useEffect(() => {
    if (!query.data || query.isFetching) return;
    const keys = new Set(
      query.data
        .filter((row) => row.participant_key === participant.participant_key)
        .map((row) => row.consequence_key),
    );
    if (seen.current) {
      const fresh = [...keys].find((key) => !seen.current!.has(key));
      if (fresh) setOpened(fresh);
    }
    seen.current = keys;
  }, [query.data, query.isFetching, participant.participant_key]);
  const dead = participant.mortality === "dead";
  const row = rows.find((row) => row.consequence_key === opened);
  const close = () => setOpened(undefined);
  const save = async (request: Parameters<typeof onSave>[0]) => {
    setError(undefined);
    try {
      if (await onSave(request)) close();
    } catch (e) {
      setError(userErrorMessage(e, "Could not save status."));
    }
  };
  const resolve = async (
    row: PendingHpLifecycleView,
    context: PendingHpLifecycleView["context"],
  ) => {
    setError(undefined);
    try {
      if (
        await onResolve({
          expected_encounter_revision: revision,
          consequence_key: row.consequence_key,
          context,
          resolution: "apply",
        })
      )
        close();
    } catch (e) {
      setError(userErrorMessage(e, "Could not save knockout."));
    }
  };
  const mainAction = (
    <Button
      size="small"
      disabled={disabled}
      onClick={() => {
        if (dead)
          void save({ track_dying: participant.track_dying, mortality: "alive" });
        else void onDefeated().catch(() => {});
      }}
    >
      {dead ? "Mark alive" : participant.defeated ? "Clear defeated" : "Mark defeated"}
    </Button>
  );
  const canTrack = !participant.track_dying;
  if (!rows.length && !canTrack && !query.error) return mainAction;
  return (
    <Space.Compact size="small">
      {mainAction}
      <Popover
        trigger="click"
        placement="bottom"
        open={opened !== undefined}
        onOpenChange={(open) => {
          setError(undefined);
          setOpened(open ? (rows[0]?.consequence_key ?? "actions") : undefined);
        }}
        title={
          row
            ? row.initiative_only
              ? "Move initiative"
              : "Knockout"
            : opened === "tracking"
              ? "Knockout"
              : "Status"
        }
        content={
          <div className="encounter-knockout">
            {error && <ErrorNotice error={error} title={error} />}
            {query.error && (
              <ErrorNotice
                error={query.error}
                compact
                title="Knockout options unavailable"
                action={
                  <Button size="small" onClick={() => void query.refetch()}>
                    Retry knockout
                  </Button>
                }
              />
            )}
            {row ? (
              <EncounterKnockout
                key={row.consequence_key}
                participant={{
                  ...participant,
                  knockout_options: row.knockout_options,
                  knockout_unavailable_reason: row.unavailable_reason,
                }}
                participants={participants}
                currentTurnKey={currentTurnKey}
                initialContext={row.context}
                initiativeOnly={row.initiative_only}
                disabled={disabled || query.isFetching || !!query.error}
                onApply={(context) => resolve(row, context)}
              />
            ) : opened === "tracking" ? (
              <EncounterKnockout
                participant={participant}
                participants={participants}
                currentTurnKey={currentTurnKey}
                disabled={disabled}
                onApply={(context) =>
                  save({ track_dying: true, knockout_context: context })
                }
              />
            ) : (
              <>
                {canTrack && (
                  <Tooltip
                    title={participant.knockout_unavailable_reason}
                    trigger={["hover", "focus"]}
                  >
                    <span>
                      <Button
                        size="small"
                        disabled={disabled || !!participant.knockout_unavailable_reason}
                        onClick={() => {
                          if (participant.current_hp === 0n) setOpened("tracking");
                          else void save({ track_dying: true });
                        }}
                      >
                        Track dying
                      </Button>
                    </span>
                  </Tooltip>
                )}
              </>
            )}
          </div>
        }
      >
        <Button
          size="small"
          disabled={disabled}
          aria-label="More status actions"
          icon={<ChevronDown size={14} />}
        />
      </Popover>
    </Space.Compact>
  );
}
