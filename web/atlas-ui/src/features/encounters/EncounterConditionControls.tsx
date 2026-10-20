import { EncounterModal } from "./EncounterModal";
import { EncounterConditionConsequences } from "./EncounterConditionConsequences";

import { EncounterPersistentRecovery } from "./EncounterPersistentRecovery";
import { useConditionEventPreview } from "./useConditionEventPreview";
import { conditionDurationLabel } from "./conditionDurationLabel";
import { Pencil, Plus, X } from "lucide-react";
import { recordPath, shouldHandleAtlasRouteClick } from "../../app/routes";
import { Alert, Button, Collapse, ConfigProvider, Tooltip } from "antd";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import type {
  AddEncounterParticipantConditionRequest,
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
  EncounterParticipantView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
import {
  EncounterConditionEditor,
  conditionDuration,
  type ConditionFormValues,
} from "./EncounterConditionEditor";
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
  const [recoveryGroup, setRecoveryGroup] = useState<
    import("../../generated/atlas").PersistentDamageGroupView | null
  >(null);
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
    useState<import("./useConditionEventPreview").ConditionEventInput>();
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
    mode: "inline" | "draft" | "commit" = "inline",
  ) => {
    const request: Omit<UpdateEncounterParticipantConditionRequest, "request_id"> = {
      expected_encounter_revision: revision,
      event_intent: "apply",
      decisions: { keep_upright: false },
      condition_id: condition.condition_id,
      name: condition.name,
      value: condition.value,
      duration: condition.duration,
      source_participant_key: condition.source_participant_key,
      note: condition.note,
      scope: condition.scope,
      extra_action: condition.extra_action,
      immunity_override: condition.immunity_override,
      parameters: condition.parameters,
      origin: condition.origin,
      ...changes,
    };
    if (mode === "draft") {
      preview.setDraft(request);
      return Promise.resolve(false);
    }
    if (mode === "commit")
      return onUpdateCondition(current.participant_key, preview.apply(request));
    return onPreviewCondition(current.participant_key, request).then((result) => {
      if (!result.consequences.length)
        return onUpdateCondition(current.participant_key, request);
      setInlineRequest(request);
      preview.setDraft(request);
      return false;
    });
  };
  const save = async (
    values: ConditionFormValues,
    durationEdited: boolean,
    previewOnly = false,
  ) => {
    if (disabled) return;
    const definition = conditionDefinitions.find(
      (d) => d.condition_ref === (selected?.condition_key ?? values.conditionRef),
    );
    const parameters =
      definition?.editor_kind === "persistent_damage" ||
      selected?.parameters.kind === "persistent_damage" ||
      selected?.parameters.kind === "persistent_damage_reminder"
        ? {
            ...(values.persistentFormula?.trim() && values.persistentDamageType
              ? {
                  kind: "persistent_damage" as const,
                  formula: values.persistentFormula.trim(),
                  damage_type: values.persistentDamageType,
                  episode_id:
                    selected?.parameters.kind === "persistent_damage"
                      ? selected.parameters.episode_id
                      : undefined,
                }
              : {
                  kind: "persistent_damage_reminder" as const,
                  formula: values.persistentFormula?.trim() || null,
                  damage_type: values.persistentDamageType || null,
                }),
            multiplier: values.persistentMultiplier ?? "normal",
            recovery:
              (selected?.parameters.kind === "persistent_damage" ||
                selected?.parameters.kind === "persistent_damage_reminder") &&
              !values.persistentRecoveryDcEdited
                ? selected.parameters.recovery
                : values.persistentRecoveryDc == null
                  ? { kind: "source_dependent" as const }
                  : {
                      kind: "specified" as const,
                      dc: BigInt(values.persistentRecoveryDc),
                    },
          }
        : definition?.editor_kind === "sickened" ||
            selected?.parameters.kind === "sickened"
          ? {
              kind: "sickened" as const,
              recovery_dc:
                values.sickenedRecoveryDc === undefined ||
                values.sickenedRecoveryDc === null
                  ? undefined
                  : BigInt(values.sickenedRecoveryDc),
            }
          : definition?.editor_kind === "unconscious" ||
              selected?.parameters.kind === "unconscious"
            ? { kind: "unconscious" as const, waking: values.wakingRule ?? "normal" }
            : definition?.editor_kind === "fascinated" ||
                selected?.parameters.kind === "fascinated"
              ? {
                  kind: "fascinated" as const,
                  subject:
                    values.conditionSubject === "__other"
                      ? {
                          kind: "other" as const,
                          description: values.conditionSubjectDescription!.trim(),
                        }
                      : values.conditionSubject &&
                          values.conditionSubject !== "__unknown"
                        ? {
                            kind: "participant" as const,
                            participant_key: values.conditionSubject,
                          }
                        : null,
                }
              : definition?.editor_kind === "fleeing" ||
                  selected?.parameters.kind === "fleeing"
                ? {
                    kind: "fleeing" as const,
                    subject:
                      values.conditionSubject === "__other"
                        ? {
                            kind: "other" as const,
                            description: values.conditionSubjectDescription!.trim(),
                          }
                        : values.conditionSubject &&
                            values.conditionSubject !== "__unknown"
                          ? {
                              kind: "participant" as const,
                              participant_key: values.conditionSubject,
                            }
                          : null,
                  }
                : definition?.editor_kind === "attitude" ||
                    selected?.parameters.kind === "attitude"
                  ? {
                      kind: "attitude" as const,
                      supernatural: values.attitudeSupernatural ?? false,
                    }
                  : definition?.editor_kind === "frightened" ||
                      selected?.parameters.kind === "frightened"
                    ? {
                        kind: "frightened" as const,
                        recovery:
                          values.frightenedRecovery === "held"
                            ? { kind: "held" as const }
                            : values.frightenedRecovery === "source_dependent"
                              ? { kind: "source_dependent" as const }
                              : {
                                  kind: "end_turn" as const,
                                  minimum: BigInt(values.frightenedMinimum ?? 0),
                                },
                      }
                    : definition?.editor_kind === "broken" ||
                        selected?.parameters.kind === "broken"
                      ? {
                          kind: "broken" as const,
                          threshold:
                            values.brokenThreshold == null
                              ? null
                              : BigInt(values.brokenThreshold),
                        }
                      : (selected?.parameters ?? { kind: "none" as const });
    const context = {
      parameters,
      scope:
        values.againstKeys && !values.againstKeys.includes("everyone")
          ? {
              kind: "selected_participants" as const,
              participant_keys: values.againstKeys,
            }
          : { kind: "everyone" as const },
      extra_action:
        values.extraActionMode === "restricted"
          ? {
              kind: "restricted" as const,
              actions: values.extraActions ?? [],
              guidance: values.extraActionGuidance?.trim() || null,
            }
          : {
              kind:
                values.extraActionMode === "unrestricted"
                  ? ("unrestricted" as const)
                  : ("unspecified" as const),
            },
      immunity_override: values.immunityOverride ?? false,
    };
    if (selected) {
      const refreshed = await update(
        selected,
        {
          ...context,
          value:
            values.value === null || values.value === undefined
              ? undefined
              : BigInt(values.value),
          duration: durationEdited ? conditionDuration(values) : selected.duration,
          source_participant_key: values.sourceParticipantKey,
          note: values.note || undefined,
        },
        editor !== null && editor !== "add" ? editor.revision : encounterRevision,
        previewOnly ? "draft" : "commit",
      );
      if (!previewOnly) finished(refreshed);
    } else {
      const definition = conditionDefinitions.find(
        (d) => d.condition_ref === values.conditionRef,
      );
      if (!definition) return;
      const request: Omit<AddEncounterParticipantConditionRequest, "request_id"> = {
        participant_key: current.participant_key,
        expected_encounter_revision: addRevision,
        event_intent: "apply",
        decisions: { keep_upright: false },
        condition_ref: definition.condition_ref,
        ...context,
        value:
          definition.has_value && values.value !== null && values.value !== undefined
            ? BigInt(values.value)
            : undefined,
        duration: conditionDuration(values),
        source_participant_key: values.sourceParticipantKey,
        note: values.note || undefined,
      };
      if (previewOnly) preview.setDraft(request);
      else finished(await onAddCondition(preview.apply(request)));
    }
  };
  const renderConditionActions = (
    condition: EncounterParticipantConditionView,
    hasValue: boolean,
    missingPersistentSetup = false,
  ) => (
    <>
      {" "}
      <Tooltip title="Edit" trigger={["hover", "focus"]}>
        <Button
          className="encounter-condition-edit"
          type="text"
          icon={<Pencil size={14} aria-hidden="true" />}
          size="small"
          disabled={disabled}
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
      {!(
        hasValue &&
        condition.value !== undefined &&
        Number(condition.value) === 1
      ) && (
        <Tooltip title="Remove" trigger={["hover", "focus"]}>
          <Button
            className="encounter-condition-remove"
            type="text"
            size="small"
            disabled={disabled}
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
      )}
    </>
  );
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
  const renderCondition = (condition: EncounterParticipantConditionView) => {
    const parameters = condition.parameters;
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
      <div className="encounter-condition-row" key={condition.condition_id.toString()}>
        {condition.condition_key ? (
          <Button
            type="link"
            className="encounter-condition-row__name"
            style={{ padding: 0, height: "auto", justifyContent: "start" }}
            href={recordPath(condition.condition_key)}
            onClick={(event) => {
              if (!shouldHandleAtlasRouteClick(event)) return;
              event.preventDefault();
              onReference(condition.condition_key!, event.currentTarget);
            }}
          >
            {condition.name}
          </Button>
        ) : (
          <span className="encounter-condition-row__name">{condition.name}</span>
        )}
        {hasValue && (
          <EncounterConditionValue
            key={valueResetEpoch}
            name={condition.name}
            value={condition.value}
            disabled={disabled}
            onCommit={(value) => update(condition, { value })}
            onClear={() =>
              onRemoveCondition(
                current.participant_key,
                condition.condition_id,
                encounterRevision,
              )
            }
          />
        )}
        {renderConditionActions(condition, !!hasValue, missingPersistentSetup)}
        {(parameters.kind === "persistent_damage" ||
          parameters.kind === "persistent_damage_reminder") && (
          <span className="encounter-condition-meta">
            {parameters.kind === "persistent_damage_reminder" && "Reminder · "}
            {parameters.formula}{" "}
            {damageTypes?.find((type) => type.value === parameters.damage_type)
              ?.label ?? parameters.damage_type}
            {parameters.multiplier !== "normal"
              ? ` · ${parameters.multiplier === "double" ? "Double" : "Half"}`
              : ""}
          </span>
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
      {!open && inlineRequest && (
        <>
          {preview.review}
          {preview.errorNotice}
          <Button
            type="primary"
            disabled={!preview.ready || disabled}
            onClick={() => {
              void onUpdateCondition(
                current.participant_key,
                preview.apply(
                  inlineRequest as Omit<
                    UpdateEncounterParticipantConditionRequest,
                    "request_id"
                  >,
                ),
              ).then((ok) => {
                if (ok) {
                  cancelInlineEdit();
                }
              });
            }}
          >
            Apply
          </Button>
          <Button
            onClick={() => {
              cancelInlineEdit();
            }}
          >
            Cancel
          </Button>
        </>
      )}
      {!open && preview.errorNotice}
      {onResolveDrained && (
        <EncounterConditionConsequences
          encounterRef={encounterRef}
          participantKey={current.participant_key}
          encounterRevision={encounterRevision}
          disabled={disabled}
          pending={pending}
          onResolve={onResolveDrained}
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
      {error && !open && (
        <Alert type="error" title="Condition action failed" description={error} />
      )}
      {current.conditions.length === 0 && !recoveryGroup ? (
        <p className="encounter-empty-note">No conditions</p>
      ) : (
        <div className="encounter-condition-list">
          {visibleGroups.map((group) => {
            const selectedSource = group.sources.find(
              (source) => source.condition_id === group.selected_source_id,
            );
            const damageType =
              damageTypes?.find((type) => type.value === group.damage_type)?.label ??
              group.damage_type;
            const summary = selectedSource
              ? `${selectedSource.formula}${selectedSource.multiplier === "normal" ? "" : selectedSource.multiplier === "double" ? " ×2" : " ×½"} ${damageType}`
              : `${damageType} · Choose source`;
            const soleCondition =
              group.sources.length === 1
                ? current.conditions.find(
                    (condition) =>
                      condition.condition_id === group.sources[0].condition_id,
                  )
                : undefined;
            const recovery = onRecoverPersistent ? (
              <EncounterPersistentRecovery
                mode="assisted"
                available={true}
                group={group}
                participantKey={current.participant_key}
                encounterRevision={encounterRevision}
                disabled={disabled}
                pending={pending}
                damageType={damageType}
                onRecover={onRecoverPersistent}
                onOpenChange={(open) => setRecoveryGroup(open ? group : null)}
              />
            ) : undefined;
            const label = `Persistent Damage · ${summary}${selectedSource?.recovery_dc != null ? ` · DC ${selectedSource.recovery_dc}` : ""}`;
            if (group.sources.length === 1)
              return (
                <div key={group.episode_id} className="encounter-condition-row">
                  <span>{label}</span>
                  {soleCondition && renderConditionActions(soleCondition, false)}
                  {recovery}
                  {soleCondition && renderConditionContext(soleCondition)}
                </div>
              );
            return (
              <Collapse
                key={group.episode_id}
                ghost
                items={[
                  {
                    key: "sources",
                    label,
                    extra: recovery,
                    children: group.sources.map((source) => {
                      const condition = current.conditions.find(
                        (condition) => condition.condition_id === source.condition_id,
                      );
                      return condition ? renderCondition(condition) : null;
                    }),
                  },
                ]}
              />
            );
          })}
          {current.conditions
            .filter((condition) => !groupedIds.has(condition.condition_id))
            .map(renderCondition)}
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
            review={
              <>
                {preview.review}
                {preview.errorNotice}
              </>
            }
            previewReady={preview.ready}
            onDraftChange={(values, durationEdited) => {
              if (values)
                void save(values, durationEdited, true).catch(() => preview.reset());
              else preview.reset();
            }}
            onReload={
              editor === "add" && addRevision !== encounterRevision
                ? () => setAddRevision(encounterRevision)
                : editor !== null &&
                    editor !== "add" &&
                    editor.revision !== encounterRevision
                  ? () => setEditor({ ...editor, revision: encounterRevision })
                  : undefined
            }
            onSave={save}
            onCancel={close}
            onRemove={removeSelected}
          />
        </EncounterModal>
      </ConfigProvider>
    </section>
  );
}
