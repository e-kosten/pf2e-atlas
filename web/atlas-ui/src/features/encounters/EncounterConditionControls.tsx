import { useQuery } from "@tanstack/react-query";
import { getEncounterConditionConsequences } from "../../api/atlasApi";
import { useConditionRecoveryControls } from "./EncounterConditionLifecycle";
import { conditionUpdate } from "./conditionDraft";
import { EncounterModal } from "./EncounterModal";
import { EncounterConditionConsequences } from "./EncounterConditionConsequences";

import { EncounterPersistentRecovery } from "./EncounterPersistentRecovery";
import { useConditionEventPreview } from "./useConditionEventPreview";
import { conditionDurationLabel } from "./conditionDurationLabel";
import { Pencil, Plus, X } from "lucide-react";
import { recordPath, shouldHandleAtlasRouteClick } from "../../app/routes";
import { Alert, Button, ConfigProvider, Popover, Space, Tooltip } from "antd";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
import { EncounterConditionEditor } from "./EncounterConditionEditor";
import { EncounterConditionValue } from "./EncounterConditionValue";

export function EncounterConditionControls({
  disabled,
  encounterRef,
  encounterRevision,
  pending,
  addDisabled,
  error,
  recovery,
  current,
  conditionDefinitions,
  damageTypes,
  standardRecoveryDc,
  onAddCondition,
  onRemoveCondition,
  onRecoverPersistent,
  onResolveDrained,
  onPreviewCondition,
  onReference,
  onUpdateCondition,
  participants,
  requestedEdit,
}: {
  disabled: boolean;
  encounterRef: string;
  encounterRevision: bigint;
  pending: boolean;
  addDisabled: boolean;
  error?: string;
  recovery?: ReactNode;
  current: EncounterParticipantView;
  conditionDefinitions: EncounterConditionDefinitionView[];
  damageTypes?: Array<{ value: string; label: string }>;
  standardRecoveryDc?: bigint;
  onAddCondition: (
    condition: Omit<AddEncounterParticipantConditionRequest, "request_id">,
  ) => Promise<boolean>;
  onPreviewCondition: (
    participantKey: string,
    request: import("./useConditionEventPreview").ConditionEventInput,
  ) => Promise<import("../../generated/atlas").ConditionEventPreviewView>;
  onResolveDrained?: (
    request: Omit<
      import("../../generated/atlas").ResolveDrainedConsequenceRequest,
      "request_id"
    >,
  ) => Promise<boolean>;
  onRecoverPersistent?: (
    request: Omit<
      import("../../generated/atlas").RecoverPersistentDamageRequest,
      "request_id"
    >,
  ) => Promise<import("../../generated/atlas").PersistentRecoveryResultView>;
  onRemoveCondition: (
    participantKey: string,
    conditionId: bigint,
    expectedRevision: bigint,
  ) => Promise<boolean>;
  onUpdateCondition: (
    participantKey: string,
    condition: Omit<UpdateEncounterParticipantConditionRequest, "request_id">,
  ) => Promise<boolean>;
  onReference: import("../../shared/records/PreparedContent").RecordReferenceHandler;
  participants: EncounterParticipantView[];
  requestedEdit?: import("./conditionEditRequest").ConditionEditRequest;
}) {
  const [actionsContainer, setActionsContainer] = useState<HTMLDivElement | null>(null);
  const editorFormId = useId();
  const preview = useConditionEventPreview(participants, (request) =>
    onPreviewCondition(current.participant_key, request),
  );
  const [editor, setEditor] = useState<
    "add" | { conditionId: bigint; revision: bigint } | null
  >(null);
  const recoveryControls = useConditionRecoveryControls();
  const conditions = recoveryControls.retainedCondition
    ? [...current.conditions, recoveryControls.retainedCondition]
    : current.conditions;
  const [recoveryContext, setRecoveryContext] = useState<{
    group: import("../../generated/atlas").PersistentDamageGroupView;
    condition?: EncounterParticipantConditionView;
  } | null>(null);
  const recoveryGroup = recoveryContext?.group;
  const visibleGroups =
    current.persistent_damage_groups.some(
      (g) => g.episode_id === recoveryGroup?.episode_id,
    ) || !recoveryGroup
      ? current.persistent_damage_groups
      : [...current.persistent_damage_groups, recoveryGroup];
  const [committed, setCommitted] = useState(false);
  const [addRevision, setAddRevision] = useState(encounterRevision);
  const handledEdit = useRef<string | undefined>(undefined);
  useEffect(() => {
    if (
      !requestedEdit ||
      requestedEdit.participantKey !== current.participant_key ||
      handledEdit.current === requestedEdit.token ||
      !current.conditions.some(
        (condition) => condition.condition_id === requestedEdit.conditionId,
      )
    )
      return;
    handledEdit.current = requestedEdit.token;
    setCommitted(false);
    setEditor({ conditionId: requestedEdit.conditionId, revision: encounterRevision });
  }, [requestedEdit, current.participant_key, current.conditions, encounterRevision]);
  const consequences = useQuery({
    queryKey: ["condition-consequences", encounterRef, encounterRevision.toString()],
    queryFn: () => getEncounterConditionConsequences(encounterRef),
    enabled: !!onResolveDrained,
  });
  const body = current.presentation?.body;
  const actor =
    body?.kind === "creature"
      ? body.value
      : body?.kind === "hazard"
        ? body.value.actor
        : undefined;
  const manualNotes = actor?.runtime?.unapplied_effects ?? [];
  const selected =
    editor !== null && editor !== "add"
      ? current.conditions.find((c) => c.condition_id === editor.conditionId)
      : undefined;
  const open = editor !== null && !(committed && !disabled);
  const [inlineRequest, setInlineRequest] =
    useState<Omit<UpdateEncounterParticipantConditionRequest, "request_id">>();
  const [valueResetEpoch, setValueResetEpoch] = useState(0);
  const cancelInlineEdit = () => {
    setInlineRequest(undefined);
    setValueResetEpoch((epoch) => epoch + 1);
    preview.reset();
  };
  const close = () => {
    preview.reset();
    setEditor(null);
  };
  const finished = (refreshed: boolean) => {
    if (refreshed) close();
    else setCommitted(true);
  };
  const update = (
    condition: EncounterParticipantConditionView,
    changes: Partial<Omit<UpdateEncounterParticipantConditionRequest, "request_id">>,
    revision = encounterRevision,
  ) => {
    const request = conditionUpdate(condition, revision, changes);
    return onPreviewCondition(current.participant_key, request).then((result) => {
      if (!result.consequences.length)
        return onUpdateCondition(current.participant_key, request);
      setInlineRequest(request);
      preview.setDraft(request);
      return false;
    });
  };
  const renderConditionActions = (
    condition: EncounterParticipantConditionView,
    missingPersistentSetup = false,
    recoveryAction?: ReactNode,
  ) => {
    const hpAction =
      onResolveDrained &&
      consequences.data
        ?.filter(
          (row) =>
            row.participant_key === current.participant_key &&
            row.condition_ref === condition.condition_key &&
            current.conditions.find((c) => c.condition_key === row.condition_ref)
              ?.condition_id === condition.condition_id,
        )
        .slice(0, 1)
        .map((row) => (
          <EncounterConditionConsequences
            key={row.consequence_key}
            row={row}
            encounterRef={encounterRef}
            encounterRevision={encounterRevision}
            level={current.level_override}
            disabled={disabled}
            pending={pending}
            onResolve={onResolveDrained}
          />
        ));
    return (
      <span className="encounter-condition-actions">
        {hpAction && hpAction.length > 0
          ? hpAction
          : (recoveryAction ??
            recoveryControls.conditionActions(condition.condition_id))}{" "}
        <Tooltip title="Edit" trigger={["hover", "focus"]}>
          <Button
            className="encounter-condition-edit"
            type="text"
            icon={<Pencil size={14} aria-hidden="true" />}
            size="small"
            disabled={
              disabled ||
              !current.conditions.some(
                (entry) => entry.condition_id === condition.condition_id,
              )
            }
            aria-label={
              missingPersistentSetup
                ? `Edit setup for ${condition.name}`
                : `Edit ${condition.name}`
            }
            onClick={() => {
              cancelInlineEdit();
              setCommitted(false);
              setEditor({
                conditionId: condition.condition_id,
                revision: encounterRevision,
              });
            }}
          >
            {missingPersistentSetup ? "Edit setup" : null}
          </Button>
        </Tooltip>
        <Tooltip
          title="Remove"
          trigger={["hover", "focus"]}
          styles={{ root: { pointerEvents: "none" } }}
        >
          <Button
            className="encounter-condition-remove"
            type="text"
            size="small"
            disabled={
              disabled ||
              !current.conditions.some(
                (entry) => entry.condition_id === condition.condition_id,
              )
            }
            aria-label={`Remove ${condition.name}`}
            icon={<X size={14} aria-hidden="true" />}
            onClick={() => {
              void onRemoveCondition(
                current.participant_key,
                condition.condition_id,
                encounterRevision,
              ).catch(() => {});
            }}
          />
        </Tooltip>
      </span>
    );
  };
  const renderConditionContext = (
    condition: EncounterParticipantConditionView,
    definition = conditionDefinitions.find(
      (entry) => entry.condition_ref === condition.condition_key,
    ),
  ) => (
    <>
      {" "}
      {condition.scope.kind === "selected_participants" && (
        <span className="encounter-condition-meta">
          {definition?.scope_label}{" "}
          {condition.scope.participant_keys
            .map(
              (key) =>
                participants.find((p) => p.participant_key === key)?.display_name ??
                key,
            )
            .join(", ")}
        </span>
      )}
      {condition.source_participant_key && (
        <span className="encounter-condition-meta">
          Source:{" "}
          {participants.find(
            (p) => p.participant_key === condition.source_participant_key,
          )?.display_name ?? "Unknown participant"}
        </span>
      )}
      {condition.immunity_override && (
        <span className="encounter-condition-meta">Immunity overridden</span>
      )}
      {condition.duration && (
        <span className="encounter-condition-meta">
          {condition.duration.unit === "rounds" && condition.duration.amount === 0n
            ? "Duration ended"
            : conditionDurationLabel(condition.duration)}
          {condition.duration.unit === "rounds" && condition.duration.timing === null
            ? " · timing not configured"
            : ""}
        </span>
      )}
      {condition.note && (
        <span className="encounter-condition-meta" style={{ whiteSpace: "pre-wrap" }}>
          {condition.note}
        </span>
      )}
      {manualNotes
        .filter((note) => note.condition_id === condition.condition_id)
        .map((note, index) => (
          <p className="encounter-condition-manual-note" key={index}>
            {note.reason}
          </p>
        ))}
    </>
  );
  const renderCondition = (
    condition: EncounterParticipantConditionView,
    presentation?: {
      label: string;
      superseded: boolean;
      recovery?: ReactNode;
      immunityNote?: string;
    },
  ) => {
    const parameters = condition.parameters;
    const persistentDetail =
      parameters.kind === "persistent_damage" ||
      parameters.kind === "persistent_damage_reminder"
        ? [
            [
              parameters.formula,
              damageTypes?.find((type) => type.value === parameters.damage_type)
                ?.label ?? parameters.damage_type,
            ]
              .filter(Boolean)
              .join(" "),
            parameters.multiplier === "normal"
              ? undefined
              : parameters.multiplier === "double"
                ? "Double"
                : "Half",
          ]
            .filter(Boolean)
            .join(" · ")
        : undefined;
    const cleared = !current.conditions.some(
      (entry) => entry.condition_id === condition.condition_id,
    );
    const definition = conditionDefinitions.find(
      (d) => d.condition_ref === condition.condition_key,
    );
    const missingPersistentSetup =
      definition?.editor_kind === "persistent_damage" &&
      parameters.kind !== "persistent_damage" &&
      parameters.kind !== "persistent_damage_reminder";
    const hasValue =
      condition.value !== undefined ||
      (definition?.has_value && !definition.supports_duration_instead_of_value);
    return (
      <div
        className={[
          "encounter-condition-row",
          hasValue && "encounter-condition-row--valued",
          (presentation?.superseded || cleared) &&
            "encounter-condition-row--superseded",
        ]
          .filter(Boolean)
          .join(" ")}
        key={condition.condition_id.toString()}
      >
        {condition.condition_key ? (
          <Button
            type="link"
            className="encounter-condition-row__name"
            style={{ padding: 0, height: "auto", justifyContent: "start" }}
            aria-label={presentation?.label}
            href={recordPath(condition.condition_key)}
            onClick={(event) => {
              if (!shouldHandleAtlasRouteClick(event)) return;
              event.preventDefault();
              onReference(condition.condition_key!, event.currentTarget);
            }}
          >
            {presentation?.label ?? condition.name}
          </Button>
        ) : (
          <span className="encounter-condition-row__name">
            {presentation?.label ?? condition.name}
          </span>
        )}
        {hasValue && (
          <Popover
            trigger="click"
            open={!open && inlineRequest?.condition_id === condition.condition_id}
            placement="bottom"
            title={condition.name}
            onOpenChange={(next) => {
              if (!next) cancelInlineEdit();
            }}
            content={
              <div className="encounter-condition-change">
                {preview.review}
                {preview.errorNotice}
                <Space>
                  <Button
                    size="small"
                    type="primary"
                    disabled={!preview.ready || disabled}
                    onClick={() => {
                      void onUpdateCondition(
                        current.participant_key,
                        preview.apply(
                          inlineRequest! as Omit<
                            UpdateEncounterParticipantConditionRequest,
                            "request_id"
                          >,
                        ),
                      )
                        .then((ok) => {
                          if (ok) cancelInlineEdit();
                        })
                        .catch(() => {});
                    }}
                  >
                    Apply
                  </Button>
                  <Button
                    size="small"
                    type="text"
                    icon={<X size={14} />}
                    aria-label="Cancel condition change"
                    onClick={cancelInlineEdit}
                  />
                </Space>
              </div>
            }
          >
            <span className="encounter-condition-row__value">
              <EncounterConditionValue
                key={valueResetEpoch}
                name={condition.name}
                value={condition.value}
                disabled={disabled || cleared}
                onCommit={(value) => update(condition, { value })}
                onClear={() =>
                  onRemoveCondition(
                    current.participant_key,
                    condition.condition_id,
                    encounterRevision,
                  )
                }
              />
            </span>
          </Popover>
        )}
        {renderConditionActions(
          condition,
          missingPersistentSetup,
          presentation?.recovery,
        )}
        {cleared && <span className="encounter-condition-meta">Cleared</span>}
        {!cleared && presentation?.superseded && (
          <span className="encounter-condition-meta">Superseded</span>
        )}
        {!presentation && persistentDetail && (
          <span className="encounter-condition-meta">{persistentDetail}</span>
        )}
        {presentation?.immunityNote && (
          <p className="encounter-condition-manual-note">{presentation.immunityNote}</p>
        )}
        {renderConditionContext(condition, definition)}
      </div>
    );
  };
  const removeSelected = selected
    ? () => {
        void onRemoveCondition(
          current.participant_key,
          selected.condition_id,
          editor !== null && editor !== "add" ? editor.revision : encounterRevision,
        )
          .then(finished)
          .catch(() => {});
      }
    : undefined;
  const groupedIds = new Set(
    current.persistent_damage_groups.flatMap((group) =>
      group.sources.map((source) => source.condition_id),
    ),
  );
  return (
    <section className="encounter-conditions">
      {!open && preview.errorNotice}
      {consequences.error && (
        <Alert
          type="error"
          title="HP changes unavailable"
          action={<Button onClick={() => void consequences.refetch()}>Retry</Button>}
        />
      )}
      <div className="encounter-conditions__header">
        <h3>Conditions</h3>
        <Button
          size="small"
          type="text"
          aria-label="Add Condition"
          title="Add Condition"
          icon={<Plus size={16} aria-hidden="true" />}
          disabled={disabled || addDisabled}
          onClick={() => {
            cancelInlineEdit();
            setCommitted(false);
            setAddRevision(encounterRevision);
            setEditor("add");
          }}
        />
      </div>
      {recoveryControls.notice}
      {error && !open && (
        <Alert type="error" title="Condition action failed" description={error} />
      )}
      {current.conditions.length === 0 && !recoveryGroup ? (
        <p className="encounter-empty-note">No conditions</p>
      ) : (
        <div className="encounter-condition-list">
          {visibleGroups.map((group) => {
            const damageType =
              damageTypes?.find((type) => type.value === group.damage_type)?.label ??
              group.damage_type;
            const recovery = onRecoverPersistent ? (
              <EncounterPersistentRecovery
                mode="explicit"
                appearance="icon"
                available={true}
                group={group}
                participantKey={current.participant_key}
                encounterRevision={encounterRevision}
                disabled={disabled}
                pending={pending}
                damageType={damageType}
                onRecover={onRecoverPersistent}
                onOpenChange={(open) =>
                  setRecoveryContext(
                    open
                      ? {
                          group,
                          condition: current.conditions.find(
                            (condition) =>
                              condition.condition_id === group.selected_source_id,
                          ),
                        }
                      : null,
                  )
                }
              />
            ) : undefined;
            const anchor =
              recoveryContext?.group.episode_id === group.episode_id
                ? recoveryContext.condition
                : undefined;
            const sources = [...group.sources];
            if (
              anchor &&
              !sources.some((source) => source.condition_id === anchor.condition_id)
            ) {
              const source = recoveryContext?.group.sources.find(
                (source) => source.condition_id === anchor.condition_id,
              );
              if (source) sources.push(source);
            }
            sources.sort(
              (a, b) =>
                Number(b.condition_id === group.selected_source_id) -
                Number(a.condition_id === group.selected_source_id),
            );
            const conditions = sources.flatMap((source) => {
              const condition =
                current.conditions.find(
                  (entry) => entry.condition_id === source.condition_id,
                ) ??
                (anchor?.condition_id === source.condition_id ? anchor : undefined);
              return condition ? [{ condition, source }] : [];
            });
            return (
              <div key={group.episode_id}>
                {conditions.map(({ condition, source }) =>
                  renderCondition(condition, {
                    immunityNote: group.immunity_note,
                    label: `${condition.name} · ${source.formula}${source.multiplier === "normal" ? "" : source.multiplier === "double" ? " ×2" : " ×½"} ${damageType}`,
                    recovery:
                      source.condition_id ===
                      (anchor?.condition_id ?? group.selected_source_id)
                        ? recovery
                        : undefined,
                    superseded: source.condition_id !== group.selected_source_id,
                  }),
                )}
              </div>
            );
          })}
          {conditions
            .filter((condition) => !groupedIds.has(condition.condition_id))
            .map((condition) => renderCondition(condition))}
        </div>
      )}
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EncounterModal
          title={selected ? `Edit ${selected.name}` : "Add condition"}
          open={open}
          destroyOnHidden
          footer={<div ref={setActionsContainer} />}
          onCancel={close}
          closable={!pending}
          keyboard={!pending}
          mask={{ closable: !pending }}
        >
          <EncounterConditionEditor
            key={
              editor === null
                ? "closed"
                : editor === "add"
                  ? `add:${addRevision}`
                  : `${editor.conditionId}:${editor.revision}`
            }
            formId={editorFormId}
            actionsContainer={actionsContainer}
            condition={selected}
            participantKind={current.participant_kind}
            definitions={conditionDefinitions}
            damageTypes={damageTypes}
            standardRecoveryDc={standardRecoveryDc}
            participants={participants}
            participantKey={current.participant_key}
            disabled={disabled || (editor === "add" && addDisabled)}
            pending={pending}
            error={error}
            recovery={recovery}
            revision={
              editor !== null && editor !== "add" ? editor.revision : addRevision
            }
            onPreview={(request) =>
              onPreviewCondition(current.participant_key, request)
            }
            onReload={
              editor === "add" && addRevision !== encounterRevision
                ? () => setAddRevision(encounterRevision)
                : editor !== null &&
                    editor !== "add" &&
                    editor.revision !== encounterRevision
                  ? () => setEditor({ ...editor, revision: encounterRevision })
                  : undefined
            }
            onSave={async (request) =>
              finished(
                await ("condition_id" in request
                  ? onUpdateCondition(current.participant_key, request)
                  : onAddCondition(request)),
              )
            }
            onRemove={removeSelected}
          />
        </EncounterModal>
      </ConfigProvider>
    </section>
  );
}
