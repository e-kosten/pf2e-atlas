import { nonNegativeIntegerRule } from "../../shared/ui/forms/nonNegativeIntegerRule";
import { conditionDraft, type ConditionFormValues } from "./conditionDraft";
import {
  useConditionEventPreview,
  type ConditionEventInput,
} from "./useConditionEventPreview";
import {
  Alert,
  Button,
  Checkbox,
  Form,
  Input,
  InputNumber,
  Select,
  Segmented,
  Space,
  Tooltip,
  Typography,
} from "antd";
import { createPortal } from "react-dom";
import { EncounterPersistentPrecedence } from "./EncounterPersistentPrecedence";
import { ConditionValueInput } from "./ConditionValueInput";
import { Info, X } from "lucide-react";
import { useEffect, useEffectEvent, useId, useState, type ReactNode } from "react";
import type {
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
  EncounterParticipantView,
} from "../../generated/atlas";

const durationFields: Array<keyof ConditionFormValues> = [
  "durationAmount",
  "durationUnit",
  "durationEvent",
  "durationParticipant",
];

export function EncounterConditionEditor({
  condition,
  formId,
  actionsContainer,
  definitions,
  damageTypes,
  standardRecoveryDc,
  participants,
  participantKey,
  participantKind,
  disabled,
  pending,
  error,
  recovery,
  revision,
  onPreview,
  onSave,
  onRemove,
  onReload,
}: {
  condition?: EncounterParticipantConditionView;
  formId?: string;
  actionsContainer?: HTMLElement | null;
  definitions: EncounterConditionDefinitionView[];
  damageTypes?: Array<{ value: string; label: string }>;
  standardRecoveryDc?: bigint;
  participants: EncounterParticipantView[];
  participantKey: string;
  participantKind?: EncounterParticipantView["participant_kind"];
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  revision: bigint;
  onPreview: (
    input: ConditionEventInput,
  ) => Promise<import("../../generated/atlas").ConditionEventPreviewView>;
  onSave: (input: ConditionEventInput) => Promise<void>;
  onRemove?: () => void;
  onReload?: () => void;
}) {
  const preview = useConditionEventPreview(participants, onPreview);
  const review = (
    <>
      {preview.review}
      {preview.errorNotice}
    </>
  );
  const name = useId();
  const [form] = Form.useForm<ConditionFormValues>();
  const [detailsOpen, setDetailsOpen] = useState(
    () => !!(condition?.source_participant_key || condition?.note),
  );
  const conditionRef = Form.useWatch("conditionRef", form);
  const definition = definitions.find(
    (d) => d.condition_ref === (condition?.condition_key ?? conditionRef),
  );
  const extraActionKeys: string[] = Form.useWatch("extraActionKeys", form) ?? [
    "unspecified",
  ];
  const eligibility = definition?.eligibility;
  const persistent =
    condition?.parameters.kind === "persistent_damage" ||
    condition?.parameters.kind === "persistent_damage_reminder"
      ? condition.parameters
      : undefined;
  const sickened =
    condition?.parameters.kind === "sickened" ? condition.parameters : undefined;
  const subjectParameters =
    condition?.parameters.kind === "fascinated" ||
    condition?.parameters.kind === "fleeing"
      ? condition.parameters
      : undefined;
  const conditionSubjectKey =
    subjectParameters?.subject?.kind === "participant"
      ? subjectParameters.subject.participant_key
      : subjectParameters?.subject?.kind === "other"
        ? "__other"
        : "__unknown";
  const subjectChoice = Form.useWatch("conditionSubject", form) ?? conditionSubjectKey;
  const attitude =
    condition?.parameters.kind === "attitude" ? condition.parameters : undefined;
  const frightened =
    condition?.parameters.kind === "frightened" ? condition.parameters : undefined;
  const broken =
    condition?.parameters.kind === "broken" ? condition.parameters : undefined;
  const unconscious =
    condition?.parameters.kind === "unconscious" ? condition.parameters : undefined;
  const persistentEditor =
    definition?.editor_kind === "persistent_damage" || !!persistent;
  const frightenedRecovery = Form.useWatch("frightenedRecovery", form) ?? "end_turn";
  const stunMode = Form.useWatch("stunMode", form) ?? "value";
  const durationInstead =
    definition?.supports_duration_instead_of_value && stunMode === "duration";
  const hasValue = definition?.has_value ?? condition?.value !== undefined;

  const durationAmount = Form.useWatch("durationAmount", form);
  const durationUnit = Form.useWatch("durationUnit", form) ?? "none";
  const isRounds =
    durationAmount !== undefined &&
    durationAmount !== null &&
    durationUnit === "rounds";
  useEffect(() => {
    if (!condition) return;
    const values: ConditionFormValues = {
      conditionRef: condition.condition_key,
      stunMode:
        condition.value === undefined && condition.duration !== undefined
          ? "duration"
          : "value",
      value: condition.value === undefined ? undefined : Number(condition.value),
      durationAmount:
        condition.duration === undefined
          ? undefined
          : Number(condition.duration.amount),
      durationUnit: condition.duration?.unit ?? "none",
      durationEvent: condition.duration?.timing?.event,
      durationParticipant: condition.duration?.timing?.participant_key,
      sourceParticipantKey: condition.source_participant_key,
      note: condition.note,
      againstKeys:
        condition.scope.kind === "selected_participants"
          ? condition.scope.participant_keys
          : ["everyone"],
      extraActionGuidance:
        condition.extra_action.kind === "restricted"
          ? (condition.extra_action.guidance ?? undefined)
          : undefined,
      immunityOverride: condition.immunity_override,
      sickenedRecoveryDc:
        sickened?.recovery_dc === undefined ? undefined : Number(sickened.recovery_dc),
      wakingRule: unconscious?.waking ?? "normal",
      conditionSubject: conditionSubjectKey,
      conditionSubjectDescription:
        subjectParameters?.subject?.kind === "other"
          ? subjectParameters.subject.description
          : undefined,
      frightenedRecovery: frightened?.recovery.kind ?? "end_turn",
      frightenedMinimum:
        frightened?.recovery.kind === "end_turn"
          ? Number(frightened.recovery.minimum)
          : 0,
      brokenThreshold: broken?.threshold == null ? undefined : Number(broken.threshold),
      persistentFormula: persistent?.formula ?? undefined,
      persistentDamageType: persistent?.damage_type ?? undefined,
      persistentMultiplier: persistent?.multiplier ?? "normal",
      persistentRecoveryDc:
        persistent?.recovery.kind === "specified"
          ? Number(persistent.recovery.dc)
          : persistent?.recovery.kind === "source_dependent"
            ? undefined
            : standardRecoveryDc == null
              ? undefined
              : Number(standardRecoveryDc),
      extraActionKeys:
        condition.extra_action.kind === "restricted"
          ? [
              ...condition.extra_action.actions,
              ...(condition.extra_action.guidance ? ["custom" as const] : []),
            ]
          : [condition.extra_action.kind],
    };
    for (const field of Object.keys(values) as Array<keyof ConditionFormValues>) {
      if (!form.isFieldTouched(field))
        form.setFields([{ name: field, value: values[field], touched: false }]);
    }
  }, [
    condition,
    standardRecoveryDc,
    form,
    persistent,
    sickened,
    unconscious,
    frightened,
    broken,
    attitude,
    subjectParameters,
    conditionSubjectKey,
  ]);
  const timingRule = {
    validator: (_rule: unknown, value: string | undefined) => {
      const unchangedUnresolved =
        condition?.duration?.unit === "rounds" &&
        condition.duration.timing === null &&
        !form.isFieldsTouched(durationFields);
      return !isRounds || value || unchangedUnresolved
        ? Promise.resolve()
        : Promise.reject(new Error("Choose when these rounds count down."));
    },
  };
  const normalizeDraft = (values: ConditionFormValues): ConditionEventInput => {
    return conditionDraft(
      {
        ...values,
        persistentRecoveryDcEdited: form.isFieldTouched("persistentRecoveryDc"),
        value: values.stunMode === "duration" ? undefined : values.value,
        immunityOverride:
          eligibility?.kind === "immune" || !!condition?.immunity_override,
      },
      form.isFieldsTouched(durationFields) || form.isFieldTouched("stunMode"),
      revision,
      participantKey,
      definition,
      condition,
    );
  };
  const watchedDraft = Form.useWatch((values) => values, form);
  const draftIdentity = JSON.stringify({ ...watchedDraft, note: undefined });
  const [preparing, setPreparing] = useState(false);
  const notifyDraft = useEffectEvent((values: ConditionFormValues | undefined) => {
    if (!values) {
      preview.reset();
      return;
    }
    try {
      preview.setDraft(normalizeDraft(values));
    } catch {
      preview.reset();
    }
  });
  useEffect(() => {
    notifyDraft(form.getFieldsValue());
  }, [draftIdentity, form]);
  const actions = (
    <div className="encounter-condition-footer">
      <Space wrap>
        {durationUnit === "none" && !durationInstead && (
          <Button
            onClick={() =>
              form.setFieldsValue({
                durationUnit: "rounds",
                durationAmount: 1,
                durationEvent: "end",
                durationParticipant: participantKey,
              })
            }
          >
            Add duration
          </Button>
        )}
        <Button
          aria-expanded={detailsOpen}
          onClick={() => setDetailsOpen(!detailsOpen)}
          disabled={pending}
        >
          {detailsOpen ? "Hide details" : "Add details"}
        </Button>
      </Space>
      <EncounterConditionEditorActions
        formId={formId ?? name}
        disabled={disabled}
        pending={pending || preparing}
        reloadRequired={!!onReload}
        onRemove={onRemove}
      />
    </div>
  );
  return (
    <>
      {onReload && (
        <Alert
          type="warning"
          title="Encounter changed"
          description="Reload this condition to review its current values before saving. This replaces your unsaved changes."
          action={
            <Button disabled={pending} onClick={onReload}>
              Reload condition
            </Button>
          }
        />
      )}
      {recovery}
      {error && (
        <Alert type="error" title="Condition action failed" description={error} />
      )}
      <Form
        name={name}
        id={formId ?? name}
        form={form}
        layout="vertical"
        disabled={disabled || preparing}
        initialValues={{
          durationUnit: condition?.duration?.unit ?? "none",
          durationAmount: condition?.duration
            ? Number(condition.duration.amount)
            : undefined,
          durationEvent: condition?.duration?.timing?.event,
          durationParticipant: condition?.duration?.timing?.participant_key,
          value: condition?.value === undefined ? undefined : Number(condition.value),
          conditionRef: condition?.condition_key,
          stunMode:
            condition?.value === undefined && condition?.duration !== undefined
              ? "duration"
              : "value",
          sourceParticipantKey: condition?.source_participant_key,
          note: condition?.note,
          againstKeys:
            condition?.scope.kind === "selected_participants"
              ? condition.scope.participant_keys
              : ["everyone"],
          extraActionGuidance:
            condition?.extra_action.kind === "restricted"
              ? (condition.extra_action.guidance ?? undefined)
              : undefined,
          immunityOverride: condition?.immunity_override,
          sickenedRecoveryDc:
            sickened?.recovery_dc === undefined
              ? undefined
              : Number(sickened.recovery_dc),
          wakingRule: unconscious?.waking ?? "normal",
          conditionSubject: conditionSubjectKey,
          conditionSubjectDescription:
            subjectParameters?.subject?.kind === "other"
              ? subjectParameters.subject.description
              : undefined,
          frightenedRecovery: frightened?.recovery.kind ?? "end_turn",
          frightenedMinimum:
            frightened?.recovery.kind === "end_turn"
              ? Number(frightened.recovery.minimum)
              : 0,
          brokenThreshold:
            broken?.threshold == null ? undefined : Number(broken.threshold),
          persistentFormula: persistent?.formula ?? undefined,
          persistentDamageType: persistent?.damage_type ?? undefined,
          persistentMultiplier: persistent?.multiplier ?? "normal",
          persistentRecoveryDc:
            persistent?.recovery.kind === "specified"
              ? Number(persistent.recovery.dc)
              : persistent?.recovery.kind === "source_dependent"
                ? undefined
                : standardRecoveryDc == null
                  ? undefined
                  : Number(standardRecoveryDc),
          extraActionKeys:
            condition?.extra_action.kind === "restricted"
              ? [
                  ...condition.extra_action.actions,
                  ...(condition.extra_action.guidance ? ["custom" as const] : []),
                ]
              : [condition?.extra_action.kind ?? "unspecified"],
        }}
        onFinishFailed={({ errorFields }) => {
          if (
            errorFields.some(({ name }) =>
              [
                "durationAmount",
                "durationUnit",
                "durationEvent",
                "durationParticipant",
                "sourceParticipantKey",
                "note",
              ].includes(String(name[0])),
            )
          )
            setDetailsOpen(true);
        }}
        onFinish={async (values) => {
          if (disabled || preparing || onReload) return;
          setPreparing(true);
          try {
            const request = await preview.prepare(normalizeDraft(values));
            if (request) await onSave(request);
          } catch {
            /* Retain draft; preview or mutation owner reports the error. */
          } finally {
            setPreparing(false);
          }
        }}
      >
        {!condition && (
          <Form.Item
            name="conditionRef"
            label="Condition"
            rules={[{ required: true, message: "Choose a condition." }]}
          >
            <Select
              aria-label="Add condition"
              showSearch={{ optionFilterProp: "label" }}
              options={definitions.map((d) => ({
                label: d.eligibility?.kind === "immune" ? `${d.name} · immune` : d.name,
                value: d.condition_ref,
              }))}
              optionRender={(option) => {
                const entry = definitions.find((d) => d.condition_ref === option.value);
                const eligibility = entry?.eligibility;
                const immune = eligibility?.kind === "immune";
                const label = (
                  <span style={{ opacity: immune ? 0.55 : 1 }}>{option.label}</span>
                );
                const reason =
                  eligibility && eligibility.kind !== "available"
                    ? eligibility.reason
                    : undefined;
                const redundant =
                  immune && reason === `Immune to ${entry?.name.toLowerCase()}.`;
                return reason && !redundant ? (
                  <Tooltip title={reason} trigger={["hover", "focus", "click"]}>
                    {label}
                  </Tooltip>
                ) : (
                  label
                );
              }}
              onChange={(ref) => {
                const selected = definitions.find((d) => d.condition_ref === ref);
                form.setFieldsValue({
                  againstKeys: ["everyone"],
                  extraActionGuidance: undefined,
                  immunityOverride: false,
                  extraActionKeys: ["unspecified"],
                  persistentFormula: undefined,
                  persistentDamageType: undefined,
                  persistentMultiplier: "normal",
                  persistentRecoveryDc:
                    standardRecoveryDc == null ? undefined : Number(standardRecoveryDc),
                  sickenedRecoveryDc: undefined,
                  wakingRule: "normal",
                  conditionSubject: "__unknown",
                  conditionSubjectDescription: undefined,
                  frightenedRecovery: "end_turn",
                  frightenedMinimum: 0,
                  brokenThreshold: undefined,
                });
                form.setFields([
                  {
                    name: "value",
                    value: selected?.has_value
                      ? Number(selected.default_value ?? 1n)
                      : undefined,
                    errors: [],
                  },
                ]);
              }}
            />
          </Form.Item>
        )}
        {persistentEditor && (
          <>
            <h3>Damage</h3>
            <div className="encounter-condition-damage-fields">
              <Form.Item name="persistentFormula">
                <Input aria-label="Damage formula" placeholder="Formula, e.g. 1d6" />
              </Form.Item>
              <Form.Item name="persistentDamageType">
                <Select
                  aria-label="Damage type"
                  allowClear
                  placeholder="Type"
                  showSearch={{ optionFilterProp: "label" }}
                  options={damageTypes ?? []}
                />
              </Form.Item>
            </div>
            {preview.persistentPrecedence && (
              <EncounterPersistentPrecedence
                precedence={preview.persistentPrecedence}
                order={preview.persistentOrder}
                disabled={pending || preview.loading}
                onChange={preview.setPersistentOrder}
              />
            )}
            <Space align="start" wrap>
              <Form.Item name="persistentMultiplier" label="Multiplier">
                <Select
                  aria-label="Damage multiplier"
                  style={{ minWidth: 110 }}
                  options={[
                    { value: "normal", label: "×1" },
                    { value: "double", label: "×2" },
                    { value: "half", label: "×½" },
                  ]}
                />
              </Form.Item>
              <Form.Item
                name="persistentRecoveryDc"
                label="Flat check DC"
                rules={[
                  nonNegativeIntegerRule(
                    persistent?.recovery.kind !== "source_dependent",
                  ),
                ]}
              >
                <InputNumber aria-label="Flat check DC" step={1} />
              </Form.Item>
            </Space>
          </>
        )}
        {(definition?.editor_kind === "fascinated" ||
          definition?.editor_kind === "fleeing" ||
          !!subjectParameters) && (
          <>
            <Form.Item
              name="conditionSubject"
              label={
                definition?.editor_kind === "fleeing" ||
                condition?.parameters.kind === "fleeing"
                  ? "Fleeing from"
                  : "Subject"
              }
            >
              <Select
                aria-label={
                  definition?.editor_kind === "fleeing" ||
                  condition?.parameters.kind === "fleeing"
                    ? "Fleeing from"
                    : "Subject"
                }
                options={[
                  { value: "__unknown", label: "Unspecified" },
                  ...participants.map((participant) => ({
                    value: participant.participant_key,
                    label: participant.display_name,
                  })),
                  { value: "__other", label: "Other" },
                ]}
              />
            </Form.Item>
            {subjectChoice === "__other" && (
              <Form.Item
                name="conditionSubjectDescription"
                label="Subject"
                rules={[
                  {
                    required: true,
                    whitespace: true,
                    message: "Name the subject.",
                  },
                  { max: 512, message: "Use 512 characters or fewer." },
                ]}
              >
                <Input aria-label="Subject description" />
              </Form.Item>
            )}
          </>
        )}
        {(definition?.editor_kind === "attitude" || !!attitude) &&
          participantKind === "pc" && (
            <Typography.Paragraph type="secondary">
              Only supernatural effects can impose an attitude on a PC.
            </Typography.Paragraph>
          )}
        {(definition?.editor_kind === "sickened" || !!sickened) && (
          <Form.Item
            name="sickenedRecoveryDc"
            label="Recovery DC"
            rules={[nonNegativeIntegerRule(false)]}
          >
            <InputNumber aria-label="Sickened recovery DC" />
          </Form.Item>
        )}
        {(definition?.editor_kind === "unconscious" || !!unconscious) && (
          <Form.Item>
            <Space>
              <Form.Item
                name="wakingRule"
                noStyle
                getValueProps={(value) => ({ checked: value !== "source_controlled" })}
                getValueFromEvent={(event) =>
                  event.target.checked ? "normal" : "source_controlled"
                }
              >
                <Checkbox>Wake automatically</Checkbox>
              </Form.Item>
              <Tooltip
                title="When healing, damage, or another normal waking trigger applies."
                trigger={["hover", "focus", "click"]}
              >
                <Button
                  type="text"
                  size="small"
                  icon={<Info size={14} />}
                  aria-label="About automatic waking"
                />
              </Tooltip>
            </Space>
          </Form.Item>
        )}
        {(definition?.editor_kind === "frightened" || !!frightened) && (
          <Space align="start" wrap>
            <Form.Item name="frightenedRecovery" label="Recovery">
              <Select
                aria-label="Frightened recovery"
                style={{ minWidth: 160 }}
                options={[
                  { value: "end_turn", label: "End of turn" },
                  { value: "held", label: "Held by source" },
                  { value: "source_dependent", label: "Source-dependent" },
                ]}
              />
            </Form.Item>
            {frightenedRecovery === "end_turn" && (
              <Form.Item
                name="frightenedMinimum"
                label="Minimum"
                rules={[nonNegativeIntegerRule(true)]}
              >
                <InputNumber aria-label="Frightened minimum" min={0} />
              </Form.Item>
            )}
          </Space>
        )}
        {(definition?.editor_kind === "broken" || !!broken) && (
          <Form.Item
            name="brokenThreshold"
            label="Broken threshold"
            rules={[nonNegativeIntegerRule(false)]}
          >
            <InputNumber aria-label="Broken threshold" min={0} />
          </Form.Item>
        )}
        {definition?.supports_duration_instead_of_value && (
          <Form.Item name="stunMode" label="Stunned">
            <Select
              aria-label="Stunned mode"
              options={[
                { value: "value", label: "Lose actions" },
                { value: "duration", label: "For a duration" },
              ]}
              onChange={(mode) => {
                if (mode === "duration" && durationUnit === "none") {
                  form.setFieldsValue({
                    durationUnit: "rounds",
                    durationAmount: 1,
                    durationEvent: "end",
                    durationParticipant: participantKey,
                  });
                }
                if (mode === "value" && form.getFieldValue("value") == null)
                  form.setFieldValue("value", 1);
              }}
            />
          </Form.Item>
        )}
        {hasValue && !durationInstead && !persistentEditor && (
          <Form.Item
            name="value"
            label="Value"
            rules={[
              {
                validator: (_rule, value) =>
                  Number.isSafeInteger(value) && value >= 1
                    ? Promise.resolve()
                    : Promise.reject(
                        new Error("Enter a whole number, one or greater."),
                      ),
              },
            ]}
          >
            <ConditionValueInput name="Condition value" onRemove={onRemove} />
          </Form.Item>
        )}
        {definition?.scope_label && (
          <Form.Item
            name="againstKeys"
            label={definition.scope_label}
            rules={[
              {
                required: true,
                type: "array",
                min: 1,
                message: "Choose Everyone or a participant.",
              },
            ]}
          >
            <Select
              aria-label={definition.scope_label}
              mode="multiple"
              onSelect={(key: string) => {
                form.setFieldValue(
                  "againstKeys",
                  key === "everyone"
                    ? ["everyone"]
                    : (form.getFieldValue("againstKeys") as string[]).filter(
                        (value) => value !== "everyone",
                      ),
                );
              }}
              options={[
                { value: "everyone", label: "Everyone" },
                ...participants
                  .filter((p) => p.participant_key !== participantKey)
                  .map((p) => ({ value: p.participant_key, label: p.display_name })),
              ]}
            />
          </Form.Item>
        )}
        {definition?.supports_extra_action && (
          <>
            <Form.Item name="extraActionKeys" label="Extra action">
              <Select
                aria-label="Extra action"
                mode="multiple"
                onChange={(keys: string[]) => {
                  if (keys.length === 0)
                    form.setFieldValue("extraActionKeys", ["unspecified"]);
                }}
                onSelect={(key: string) =>
                  form.setFieldValue(
                    "extraActionKeys",
                    ["unspecified", "unrestricted"].includes(key)
                      ? [key]
                      : (form.getFieldValue("extraActionKeys") as string[]).filter(
                          (value) => !["unspecified", "unrestricted"].includes(value),
                        ),
                  )
                }
                options={[
                  { value: "unspecified", label: "Unspecified" },
                  { value: "unrestricted", label: "Unrestricted" },
                  ...[
                    "Stride",
                    "Strike",
                    "Step",
                    "Leap",
                    "Stand",
                    "Interact",
                    "Cast spell",
                    "Sustain",
                    "Escape",
                    "Crawl",
                    "Fly",
                    "Swim",
                    "Climb",
                    "Burrow",
                  ].map((label) => ({
                    label,
                    value: label.toLowerCase().replace(/ /g, "_"),
                  })),
                  { value: "custom", label: "Custom restriction" },
                ]}
              />
            </Form.Item>
            {extraActionKeys.includes("custom") && (
              <Form.Item name="extraActionGuidance" label="Custom restriction">
                <Input.TextArea aria-label="Custom restriction" autoSize />
              </Form.Item>
            )}
          </>
        )}
        <>
          <div
            style={{
              display: durationUnit === "none" && !durationInstead ? "none" : undefined,
            }}
          >
            <Form.Item label="Duration">
              <Space.Compact>
                <Form.Item
                  name="durationAmount"
                  noStyle
                  rules={[nonNegativeIntegerRule(durationUnit !== "none")]}
                >
                  <InputNumber
                    aria-label="Duration amount"
                    min={0}
                    step={1}
                    onKeyDown={(event) => {
                      if (event.key === "-") event.preventDefault();
                    }}
                    onPaste={(event) => {
                      if (/^\s*[-−]/.test(event.clipboardData.getData("text")))
                        event.preventDefault();
                    }}
                    style={{ width: 88 }}
                  />
                </Form.Item>
                <Form.Item name="durationUnit" noStyle>
                  <Select
                    aria-label="Duration unit"
                    onChange={(unit) => {
                      if (unit === "rounds")
                        form.setFieldsValue({
                          durationEvent: form.getFieldValue("durationEvent") ?? "end",
                          durationParticipant:
                            form.getFieldValue("durationParticipant") ?? participantKey,
                        });
                    }}
                    style={{ width: 128 }}
                    options={["rounds", "minutes", "hours", "days"].map((value) => ({
                      value,
                      label: value,
                    }))}
                  />
                </Form.Item>
                {!durationInstead && (
                  <Tooltip title="Remove duration" trigger={["hover", "focus"]}>
                    <Button
                      aria-label="Remove duration"
                      icon={<X size={14} aria-hidden="true" />}
                      onClick={() =>
                        form.setFieldsValue({
                          durationUnit: "none",
                          durationAmount: undefined,
                          durationEvent: undefined,
                          durationParticipant: undefined,
                        })
                      }
                    />
                  </Tooltip>
                )}
              </Space.Compact>
            </Form.Item>
          </div>
          {isRounds && (
            <Space align="start" wrap>
              <Form.Item name="durationEvent" rules={[timingRule]}>
                <Segmented
                  aria-label="Duration boundary"
                  options={[
                    { label: "Starts on", value: "start" },
                    { label: "Ends on", value: "end" },
                  ]}
                />
              </Form.Item>
              <Form.Item name="durationParticipant" rules={[timingRule]}>
                <Select
                  aria-label="Whose turn"
                  placeholder="Whose turn"
                  style={{ minWidth: 180 }}
                  options={participants.map((p) => ({
                    value: p.participant_key,
                    label: `${p.display_name}’s turn`,
                  }))}
                />
              </Form.Item>
            </Space>
          )}
        </>
        {
          <section
            aria-label="Condition details"
            style={{ display: detailsOpen ? undefined : "none" }}
          >
            <Form.Item name="sourceParticipantKey" label="Source">
              <Select
                aria-label="Source"
                allowClear
                options={participants.map((p) => ({
                  value: p.participant_key,
                  label: p.display_name,
                }))}
              />
            </Form.Item>
            <Form.Item name="note" label="Note">
              <Input.TextArea
                aria-label="Condition note"
                autoSize={{ minRows: 2, maxRows: 8 }}
              />
            </Form.Item>
          </section>
        }
        {review}
        {actionsContainer ? createPortal(actions, actionsContainer) : actions}
      </Form>
    </>
  );
}

export function EncounterConditionEditorActions({
  formId,
  disabled,
  pending,
  reloadRequired,
  onRemove,
}: {
  formId: string;
  disabled: boolean;
  pending: boolean;
  reloadRequired: boolean;
  onRemove?: () => void;
}) {
  return (
    <Space wrap>
      {onRemove && (
        <Button danger disabled={disabled} onClick={onRemove}>
          Remove
        </Button>
      )}
      <Button
        type="primary"
        htmlType="submit"
        form={formId}
        disabled={disabled || reloadRequired}
        loading={pending}
      >
        Apply
      </Button>
    </Space>
  );
}
