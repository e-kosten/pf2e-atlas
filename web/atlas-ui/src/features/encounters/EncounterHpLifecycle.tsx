import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Collapse, Select, Space } from "antd";
import { useId, useState } from "react";
import {
  getEncounterHpLifecycle,
  previewEncounterHpLifecycle,
} from "../../api/atlasApi";
import type {
  EncounterParticipantView,
  HpEventContextView,
  PendingHpLifecycleView,
  ResolveEncounterHpLifecycleRequest,
} from "../../generated/atlas";
export function EncounterHpLifecycle({
  encounterRef,
  revision,
  participants,
  disabled,
  onResolve,
}: {
  encounterRef: string;
  revision: bigint;
  participants: EncounterParticipantView[];
  disabled: boolean;
  onResolve: (
    request: Omit<ResolveEncounterHpLifecycleRequest, "request_id">,
  ) => Promise<boolean>;
}) {
  const query = useQuery({
    queryKey: ["hp-lifecycle", encounterRef, revision.toString()],
    queryFn: () => getEncounterHpLifecycle(encounterRef),
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef ? previous : undefined,
  });
  if (query.error)
    return (
      <Alert
        type="error"
        title="Could not load HP follow-up"
        action={<Button onClick={() => void query.refetch()}>Retry</Button>}
      />
    );
  if (!query.data?.length) return null;
  return (
    <Collapse
      defaultActiveKey={["hp"]}
      items={[
        {
          key: "hp",
          label: "HP follow-up",
          children: query.data.map((row) => (
            <HpDecision
              key={row.consequence_key}
              row={row}
              encounterRef={encounterRef}
              revision={revision}
              participants={participants}
              disabled={disabled || query.isFetching}
              onResolve={onResolve}
            />
          )),
        },
      ]}
    />
  );
}
function HpDecision({
  row,
  encounterRef,
  revision,
  participants,
  disabled,
  onResolve,
}: {
  row: PendingHpLifecycleView;
  encounterRef: string;
  revision: bigint;
  participants: EncounterParticipantView[];
  disabled: boolean;
  onResolve: (
    request: Omit<ResolveEncounterHpLifecycleRequest, "request_id">,
  ) => Promise<boolean>;
}) {
  const id = useId();
  const [context, setContext] = useState(row.context);
  const [resolution, setResolution] = useState<"apply" | "keep_current_state">("apply");
  const [openingRevision, setOpeningRevision] = useState(revision);
  const [error, setError] = useState<string>();
  const [saving, setSaving] = useState(false);
  const [seen, setSeen] = useState(row.required_facts);
  const changed = revision !== openingRevision;
  const request = {
    request_id: id,
    expected_encounter_revision: openingRevision,
    consequence_key: row.consequence_key,
    context,
    resolution,
  };
  const query = useQuery({
    queryKey: [
      "hp-lifecycle-preview",
      encounterRef,
      row.consequence_key,
      openingRevision.toString(),
      resolution,
      JSON.stringify(context, (_key, value) =>
        typeof value === "bigint" ? value.toString() : value,
      ),
    ],
    queryFn: () => previewEncounterHpLifecycle(encounterRef, request),
    enabled: !changed,
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef && query.queryKey[2] === row.consequence_key
        ? previous
        : undefined,
  });
  const required = query.data?.required_facts ?? row.required_facts;
  const fields = [...new Set([...seen, ...required])];
  function update(value: Partial<HpEventContextView>) {
    setSeen(fields);
    setContext((previous) => ({ ...previous, ...value }));
  }
  const blocked = disabled || saving || changed;
  return (
    <section>
      <h4>
        {participants.find((p) => p.participant_key === row.participant_key)
          ?.display_name ?? row.participant_key}
      </h4>
      <p>
        HP {row.before_hp.toString()} → {row.after_hp.toString()}
      </p>
      {changed && (
        <Alert
          type="warning"
          title="Encounter changed"
          action={
            <Button
              onClick={() => {
                setOpeningRevision(revision);
                setContext(row.context);
                setResolution("apply");
                setSeen(row.required_facts);
              }}
            >
              Reload follow-up
            </Button>
          }
        />
      )}
      {(error || query.error) && (
        <Alert
          type="error"
          title="Could not review HP follow-up"
          description={error ?? query.error?.message}
          action={
            query.error ? (
              <Button onClick={() => void query.refetch()}>Retry</Button>
            ) : undefined
          }
        />
      )}
      <Space wrap>
        <Select
          aria-label="HP follow-up resolution"
          value={resolution}
          disabled={blocked}
          onChange={setResolution}
          options={[
            { value: "apply", label: "Apply consequences" },
            { value: "keep_current_state", label: "Keep current state" },
          ]}
        />
        {resolution === "apply" && fields.includes("lethality") && (
          <Select
            aria-label="Damage lethality"
            placeholder="Damage lethality"
            disabled={blocked}
            value={context.lethality ?? undefined}
            onChange={(lethality) => update({ lethality })}
            options={[
              { value: "lethal", label: "Lethal" },
              { value: "nonlethal", label: "Nonlethal" },
            ]}
          />
        )}
        {resolution === "apply" && fields.includes("critical") && (
          <Select
            aria-label="Critical damage"
            placeholder="Critical hit?"
            disabled={blocked}
            value={context.critical === null ? undefined : String(context.critical)}
            onChange={(value) => update({ critical: value === "true" })}
            options={[
              { value: "true", label: "Critical hit" },
              { value: "false", label: "Not a critical hit" },
            ]}
          />
        )}
        {resolution === "apply" && fields.includes("enemy_source") && (
          <Select
            aria-label="Enemy damage source"
            placeholder="Enemy source?"
            disabled={blocked}
            value={
              context.enemy_source === null ? undefined : String(context.enemy_source)
            }
            onChange={(value) => update({ enemy_source: value === "true" })}
            options={[
              { value: "true", label: "Enemy source" },
              { value: "false", label: "Other source" },
            ]}
          />
        )}
        {resolution === "apply" && fields.includes("responsible_participant") && (
          <Select
            aria-label="Initiative follows"
            placeholder="Move before…"
            disabled={blocked}
            value={
              context.keep_current_initiative
                ? "__keep"
                : (context.responsible_participant_key ?? undefined)
            }
            onChange={(value) =>
              update({
                keep_current_initiative: value === "__keep",
                responsible_participant_key:
                  value === "__keep" ? context.responsible_participant_key : value,
              })
            }
            options={[
              { value: "__keep", label: "Keep current initiative" },
              ...participants
                .filter((p) => p.participant_key !== row.participant_key)
                .map((p) => ({ value: p.participant_key, label: p.display_name })),
            ]}
          />
        )}
        <Button
          disabled={
            blocked ||
            query.isFetching ||
            !!query.error ||
            !query.data ||
            required.length > 0
          }
          loading={saving}
          onClick={async () => {
            setSaving(true);
            setError(undefined);
            try {
              await onResolve({
                expected_encounter_revision: openingRevision,
                consequence_key: row.consequence_key,
                context,
                resolution,
              });
            } catch (value) {
              setError(
                value instanceof Error ? value.message : "Could not save HP follow-up.",
              );
            } finally {
              setSaving(false);
            }
          }}
        >
          Apply follow-up
        </Button>
      </Space>
    </section>
  );
}
