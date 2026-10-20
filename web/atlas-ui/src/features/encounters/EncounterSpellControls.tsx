import { statusReasonLabel } from "../../shared/records/statusReasonLabel";
import { useRetainedRecord } from "../../shared/records/useRetainedRecord";
import { EncounterModal } from "./EncounterModal";
import {
  Alert,
  App,
  Button,
  InputNumber,
  Popover,
  Space,
  Tooltip,
  Typography,
} from "antd";
import { Minus, Plus, Settings2 } from "lucide-react";
import type { CastingAnnotations } from "../../shared/records/ActorReference";
import { useQuery } from "@tanstack/react-query";
import { previewEncounterCastingAttempt } from "../../api/atlasApi";
import { spellResourceCount, spellResourceExhausted } from "./spellResourceDisplay";
import { useRef, useState, type ReactNode } from "react";
import type {
  EncounterParticipantView,
  EncounterSpellAction,
  EncounterSpellResourceView,
  RecordDetailView,
} from "../../generated/atlas";

type Actions = {
  participant: EncounterParticipantView;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onAction: (
    action: EncounterSpellAction,
    expectedRevision?: bigint,
  ) => Promise<boolean>;
};

export function EncounterSpellControls({
  participant,
  detail,
  disabled,
  pending,
  recovery,
  error,
  onAction,
  encounterRef,
}: Actions & {
  detail: RecordDetailView;
  encounterRef?: string;
}) {
  const { message } = App.useApp();
  const inFlight = useRef(false);
  const displayedSelection = detail.selected;
  const catalog = participant.spellcasting;
  const spell = catalog?.spells.find(
    (s) =>
      s.navigation.record_key === displayedSelection.record_key &&
      JSON.stringify(s.navigation.embedded_path) ===
        JSON.stringify(displayedSelection.embedded_path),
  );
  if (!spell || detail.presentation.body.kind !== "spell") return null;
  const rank = detail.presentation.body.value.rank;
  const selectedRank =
    displayedSelection.spell_rank ??
    (rank.state === "value" ? Number(rank.value) : undefined);
  const selection = {
    ...spell.navigation,
    source_fingerprint:
      displayedSelection.source_fingerprint ?? spell.navigation.source_fingerprint,
    spell_rank: selectedRank,
  };
  const option = spell.ranks.find((r) => r.rank === selectedRank);
  const resource = catalog?.resources.find((r) => r.key === option?.resource_key);
  const unavailable =
    (spell.expended ? "Expended" : null) ??
    option?.unavailable_reason ??
    (!option
      ? "Casting unavailable"
      : resource && spellResourceExhausted(resource)
        ? "No uses remaining"
        : undefined);
  const cast = async () => {
    if (inFlight.current || disabled || pending || unavailable) return;
    inFlight.current = true;
    try {
      if (await onAction({ kind: "cast", selection }))
        void message.success("Use recorded", 2);
    } catch {
      /* The mutation owner reports the failure. */
    } finally {
      inFlight.current = false;
    }
  };
  return (
    <div className="encounter-casting__controls">
      <Space wrap>
        <Button
          type="primary"
          disabled={disabled || pending || !!unavailable}
          loading={pending}
          onClick={() => void cast()}
        >
          Cast
        </Button>
        {spell.expended === true && (
          <Button
            disabled={disabled || pending}
            onClick={() =>
              void onAction({
                kind: "set_expended",
                selection,
                expended: !spell.expended,
              })
            }
          >
            {spell.expended ? "Restore" : "Expend"}
          </Button>
        )}
        {resource ? (
          <ResourceCount resource={resource} />
        ) : (
          <Typography.Text type="secondary">
            {unavailable ?? "Unlimited"}
          </Typography.Text>
        )}
        {resource?.can_undo && (
          <Button
            disabled={disabled}
            onClick={() => void onAction({ kind: "undo", resource_key: resource.key })}
          >
            Undo
          </Button>
        )}
      </Space>
      {encounterRef && (
        <SpellCastingReminders
          encounterRef={encounterRef}
          generation={detail.artifact_generation}
          participant={participant}
          selection={selection}
        />
      )}
      {error && <Alert type="error" title="Could not cast" description={error} />}
      {recovery}
    </div>
  );
}

function SpellCastingReminders({
  encounterRef,
  generation,
  participant,
  selection,
}: {
  encounterRef: string;
  generation: string;
  participant: EncounterParticipantView;
  selection: import("../../generated/atlas").RecordNavigationView;
}) {
  const readingIdentity = JSON.stringify(
    [encounterRef, participant.participant_key, generation, selection],
    (_key, value) => (typeof value === "bigint" ? value.toString() : value),
  );
  const preview = useQuery({
    queryKey: [
      "spell-casting-reminders",
      generation,
      encounterRef,
      participant.participant_key,
      participant.revision.toString(),
      JSON.stringify(selection, (_key, value) =>
        typeof value === "bigint" ? value.toString() : value,
      ),
    ],
    queryFn: () =>
      previewEncounterCastingAttempt(encounterRef, {
        participant_key: participant.participant_key,
        selection,
        judgments: [],
        target: null,
        confusion: null,
      }),
    retry: false,
  });
  const reminders = useRetainedRecord(readingIdentity, preview.data, preview.error);
  return (
    <>
      {preview.error && (
        <Space>
          <Typography.Text type="secondary">Reminders unavailable</Typography.Text>
          <Button type="link" onClick={() => void preview.refetch()}>
            Retry
          </Button>
        </Space>
      )}
      {reminders?.advisories.map((advisory, index) => (
        <p key={index}>{statusReasonLabel(advisory.label, advisory.reason)}</p>
      ))}
    </>
  );
}

type ResourceDraft = {
  revision: bigint;
  resources: EncounterSpellResourceView[];
  remaining: Record<string, number | null>;
};

export function EncounterSpellResources({
  participant,
  disabled,
  pending,
  error,
  recovery,
  onAction,
  entryId,
  title,
}: Actions & { entryId: string; title: string }) {
  const [draft, setDraft] = useState<ResourceDraft | null>(null);
  const resources = (participant.spellcasting?.resources ?? []).filter(
    (r) => r.entry_id === entryId,
  );
  if (!resources.length && !draft) return null;
  const changed = draft && String(draft.revision) !== String(participant.revision);
  const invalid =
    !draft ||
    draft.resources.some((r) => {
      const n = draft.remaining[r.key];
      return n === null || !Number.isInteger(n) || n < 0 || n > Number(r.capacity);
    });
  const cancel = () => {
    if (!pending) setDraft(null);
  };
  const done = async () => {
    if (!draft || invalid || changed || disabled) return;
    if (draft.resources.every((r) => draft.remaining[r.key] === Number(r.remaining))) {
      setDraft(null);
      return;
    }
    const remaining = Object.fromEntries(
      Object.entries(draft.remaining).map(([key, value]) => [key, BigInt(value!)]),
    );
    if (
      await onAction(
        { kind: "set_entry_resources", entry_id: entryId, remaining },
        draft.revision,
      )
    )
      setDraft(null);
  };
  return (
    <>
      <Tooltip title={`Manage ${title} resources`}>
        <Button
          type="text"
          size="small"
          aria-label={`Manage ${title} resources`}
          icon={<Settings2 size={14} />}
          onClick={() =>
            setDraft({
              revision: participant.revision,
              resources,
              remaining: Object.fromEntries(
                resources.map((r) => [r.key, Number(r.remaining)]),
              ),
            })
          }
        />
      </Tooltip>
      <EncounterModal
        title={title}
        open={!!draft}
        onCancel={cancel}
        closable={!pending}
        keyboard={!pending}
        destroyOnHidden
        footer={
          <div className="encounter-casting__editor-footer">
            <Button
              disabled={disabled || !!changed}
              onClick={() =>
                setDraft(
                  (d) =>
                    d && {
                      ...d,
                      remaining: Object.fromEntries(
                        d.resources.map((r) => [r.key, Number(r.capacity)]),
                      ),
                    },
                )
              }
            >
              Restore all
            </Button>
            <Space>
              <Button disabled={pending} onClick={cancel}>
                Cancel
              </Button>
              <Button
                type="primary"
                disabled={disabled || invalid || !!changed}
                loading={pending}
                onClick={() => void done()}
              >
                Done
              </Button>
            </Space>
          </div>
        }
      >
        {draft?.resources.map((resource) => (
          <div className="encounter-casting__resource-row" key={resource.key}>
            <span>{resource.label}</span>
            <Space>
              <InputNumber
                aria-label={`${resource.label} remaining`}
                min={0}
                max={Number(resource.capacity)}
                precision={0}
                value={draft.remaining[resource.key]}
                disabled={disabled || !!changed}
                onChange={(value) =>
                  setDraft(
                    (d) =>
                      d && {
                        ...d,
                        remaining: { ...d.remaining, [resource.key]: value },
                      },
                  )
                }
              />
              <span>/ {String(resource.capacity)}</span>
            </Space>
          </div>
        ))}
        {changed ? (
          <Typography.Text type="danger" role="alert">
            This participant changed. Your edits have not been saved. Cancel and reopen
            to use the current values.
          </Typography.Text>
        ) : (
          error && (
            <Typography.Text type="danger" role="alert">
              {error}
            </Typography.Text>
          )
        )}
        {recovery}
      </EncounterModal>
    </>
  );
}

function ResourceCount({ resource }: { resource: EncounterSpellResourceView }) {
  const description = `${resource.label}: ${resource.remaining} of ${resource.capacity} remaining`;
  return (
    <Tooltip title={description}>
      <span className="encounter-casting__count" aria-label={description}>
        {spellResourceCount(resource)}
      </span>
    </Tooltip>
  );
}

export function SharedResourceControl({
  resource,
  participant,
  disabled,
  pending,
  onAction,
}: Actions & { resource: EncounterSpellResourceView }) {
  const { message } = App.useApp();
  const remaining = Number(resource.remaining);
  const capacity = Number(resource.capacity);
  const update = async (value: number) => {
    if (disabled || pending || value < 0 || value > capacity || value === remaining)
      return;
    const pools =
      participant.spellcasting?.resources.filter(
        (r) => r.entry_id === resource.entry_id,
      ) ?? [];
    const saved = await onAction(
      {
        kind: "set_entry_resources",
        entry_id: resource.entry_id,
        remaining: Object.fromEntries(
          pools.map((r) => [
            r.key,
            r.key === resource.key ? BigInt(value) : r.remaining,
          ]),
        ),
      },
      participant.revision,
    );
    if (!saved)
      void message.error(
        "Couldn’t update resources. Check the current count and try again.",
        4,
      );
  };
  return (
    <Popover
      trigger="click"
      title={resource.label}
      content={
        <Space direction="vertical">
          <Space>
            <Button
              aria-label={`Expend one ${resource.label}`}
              icon={<Minus size={14} />}
              disabled={disabled || pending || remaining === 0}
              onClick={() => void update(remaining - 1)}
            />
            <span aria-live="polite">{spellResourceCount(resource)}</span>
            <Button
              aria-label={`Restore one ${resource.label}`}
              icon={<Plus size={14} />}
              disabled={disabled || pending || remaining === capacity}
              onClick={() => void update(remaining + 1)}
            />
          </Space>
          <Button
            disabled={disabled || pending || remaining === capacity}
            onClick={() => void update(capacity)}
          >
            Restore all
          </Button>
        </Space>
      }
    >
      <Button
        type="link"
        size="small"
        aria-label={`${resource.label}: ${remaining} of ${capacity} remaining`}
      >
        {spellResourceCount(resource)}
      </Button>
    </Popover>
  );
}

export function encounterCastingAnnotations(props: Actions): CastingAnnotations {
  const catalog = props.participant.spellcasting;
  const resources = catalog?.resources ?? [];
  const spellView = (activity: import("../../generated/atlas").ActorActivityView) =>
    catalog?.spells.find(
      (s) =>
        s.navigation.record_key === activity.navigation.record_key &&
        JSON.stringify(s.navigation.embedded_path) ===
          JSON.stringify(activity.navigation.embedded_path),
    );
  const entryId = (entry: import("../../generated/atlas").ActorActivityView) =>
    entry.navigation.embedded_path.map((o) =>
      "Stable" in o.identity ? o.identity.Stable : undefined,
    )[entry.navigation.embedded_path.length - 1];
  const entryResources = (entry: import("../../generated/atlas").ActorActivityView) => {
    const id = entryId(entry);
    const sharedKeys = new Set(
      catalog?.spells
        .filter((s) => s.entry_id === id)
        .flatMap((s) => s.ranks.map((r) => r.resource_key)),
    );
    return resources.filter(
      (r) => r.entry_id === id || (r.scope.kind === "entry" && sharedKeys.has(r.key)),
    );
  };
  return {
    entry: (entry) => {
      const pools = entryResources(entry);
      if (!pools.length) return null;
      const ids = [...new Set(pools.map((r) => r.entry_id))];
      return (
        <span className="encounter-casting__entry-controls">
          {pools
            .filter((r) => r.scope.kind === "entry")
            .map((r) => (
              <SharedResourceControl key={r.key} resource={r} {...props} />
            ))}
          {ids.map((id) => (
            <EncounterSpellResources
              key={id}
              {...props}
              entryId={id}
              title={id === "focus" ? "Focus" : entry.title}
            />
          ))}
        </span>
      );
    },
    rank: (entry, rank) =>
      entryResources(entry)
        .filter((r) => r.scope.kind === "rank" && r.scope.rank === rank)
        .map((r) => <SharedResourceControl key={r.key} resource={r} {...props} />),
    spell: (activity) => {
      const spell = spellView(activity);
      const option = spell?.ranks.find(
        (r) => r.rank === activity.navigation.spell_rank,
      );
      const resource = resources.find((r) => r.key === option?.resource_key);
      const depleted = !!resource && spellResourceExhausted(resource);
      return {
        muted: spell?.expended === true || depleted,
        status: spell?.expended
          ? "Expended"
          : depleted
            ? "No uses remaining"
            : undefined,
        suffix:
          resource?.scope.kind === "spell" ? (
            <ResourceCount resource={resource} />
          ) : undefined,
      };
    },
  };
}
