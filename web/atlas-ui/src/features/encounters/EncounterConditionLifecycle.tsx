import { EncounterModal } from "./EncounterModal";
import { actionIssueQuestionLabel, actionIssueSourceLabel } from "./actionIssueLabels";
import { ConfusionContextFields } from "./ConfusionContextFields";
import { useQuery } from "@tanstack/react-query";
import {
  Alert,
  Button,
  Checkbox,
  Collapse,
  Form,
  InputNumber,
  Select,
  Space,
  Tooltip,
} from "antd";
import { useState } from "react";
import {
  getEncounterConditionLifecycleActions,
  previewEncounterRetch,
} from "../../api/atlasApi";
import type {
  ApplyConditionLifecycleRequest,
  ConditionLifecycleActionView,
  ConditionLifecycleOptionView,
  ConditionLifecycleResultView,
  EncounterParticipantView,
  EncounterActionJudgmentView,
  EncounterConfusionContextView,
  EncounterActionIssueView,
} from "../../generated/atlas";
import { ActionIssueContext } from "./ActionIssueContext";
import { integerRule } from "./EncounterConditionEditor";
import {
  LifecycleCheckFields,
  lifecycleCheck,
  type LifecycleCheckValues,
} from "./LifecycleCheckFields";
type Values = LifecycleCheckValues & {
  dc?: number;
  modifier?: number;
  actionsSpent?: boolean;
  minutes?: number;
  healing?: number;
  actionsLost?: number;
  threshold?: number;
  qualifying?: string;
  cause?: "interact" | "rest_interrupted" | "rest_completed";
  sources?: string[];
  frightened?: Record<string, { reduce?: string; minimum?: number }>;
  judgments?: Record<string, string>;
};
const signedInteger = {
  validator: (_rule: unknown, value: number | undefined) =>
    Number.isSafeInteger(value)
      ? Promise.resolve()
      : Promise.reject(new Error("Enter a whole number.")),
};
export function EncounterConditionLifecycle({
  encounterRef,
  boundaryKey = null,
  revision,
  participant,
  participants,
  disabled,
  onApply,
}: {
  encounterRef: string;
  boundaryKey?: string | null;
  revision: bigint;
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  disabled: boolean;
  onApply: (
    request: Omit<ApplyConditionLifecycleRequest, "request_id">,
  ) => Promise<ConditionLifecycleResultView>;
}) {
  const query = useQuery({
    queryKey: [
      "condition-lifecycle-actions",
      encounterRef,
      participant.participant_key,
      revision.toString(),
      boundaryKey,
    ],
    queryFn: () =>
      getEncounterConditionLifecycleActions(
        encounterRef,
        participant.participant_key,
        boundaryKey,
      ),
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef &&
      query.queryKey[2] === participant.participant_key &&
      query.queryKey[4] === boundaryKey
        ? previous
        : undefined,
  });
  const [selected, setSelected] = useState<{
    option: ConditionLifecycleOptionView;
    revision: bigint;
  } | null>(null);
  const [result, setResult] = useState<ConditionLifecycleResultView | null>(null);
  if (query.error)
    return (
      <Alert
        type="error"
        title="Could not load recovery actions"
        action={<Button onClick={() => void query.refetch()}>Retry</Button>}
      />
    );
  if (!query.data?.actions.length && !selected && !result) return null;
  return (
    <>
      <Collapse
        ghost
        items={[
          {
            key: "recovery",
            label: "Recovery and rest",
            children: (
              <Space wrap>
                {query.data?.actions.map((option, index) => (
                  <Tooltip
                    key={`${option.kind}:${option.condition_id ?? index}`}
                    title={option.unavailable_reason}
                    trigger={["hover", "focus", "click"]}
                  >
                    <span tabIndex={option.available ? undefined : 0}>
                      <Button
                        size="small"
                        disabled={disabled || query.isFetching || !option.available}
                        onClick={() =>
                          setSelected({
                            option,
                            revision: query.data!.encounter_revision,
                          })
                        }
                      >
                        {option.label}
                      </Button>
                    </span>
                  </Tooltip>
                ))}
              </Space>
            ),
          },
        ]}
      />
      {result && (
        <p role="status">
          {result.check
            ? [
                result.check.degree.replace(/_/g, " "),
                result.check.table_result !== null
                  ? `Result ${result.check.table_result}`
                  : result.check.roll
                    ? `Result ${result.check.roll.total}`
                    : null,
                result.check.dc === null ? null : `DC ${result.check.dc}`,
              ]
                .filter(Boolean)
                .join(" · ")
            : "Recovery saved"}
        </p>
      )}
      <EncounterModal
        title={selected?.option.label ?? "Recovery"}
        open={!!selected}
        footer={null}
        destroyOnHidden
        closable={!disabled}
        keyboard={!disabled}
        mask={{ closable: !disabled }}
        onCancel={() => {
          if (!disabled) setSelected(null);
        }}
      >
        {selected && (
          <LifecycleForm
            key={`${selected.option.kind}:${selected.option.condition_id}:${selected.revision}`}
            encounterRef={encounterRef}
            option={selected.option}
            participant={participant}
            participants={participants}
            revision={selected.revision}
            changed={selected.revision !== revision}
            disabled={disabled || query.isFetching}
            onReload={() => {
              const option = query.data?.actions.find(
                (option) =>
                  option.kind === selected.option.kind &&
                  option.condition_id === selected.option.condition_id &&
                  option.consequence_key === selected.option.consequence_key,
              );
              if (option) setSelected({ option, revision });
              else setSelected(null);
            }}
            onApply={async (action) => {
              const value = await onApply({
                boundary_key: boundaryKey,
                expected_encounter_revision: selected.revision,
                participant_key: participant.participant_key,
                action,
              });
              setResult(value);
              setSelected(null);
              void query.refetch();
            }}
          />
        )}
      </EncounterModal>
    </>
  );
}
function LifecycleForm({
  encounterRef,
  revision,
  option,
  participant,
  participants,
  changed,
  disabled,
  onReload,
  onApply,
}: {
  encounterRef: string;
  option: ConditionLifecycleOptionView;
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  revision: bigint;
  changed: boolean;
  disabled: boolean;
  onReload: () => void;
  onApply: (action: ConditionLifecycleActionView) => Promise<void>;
}) {
  const [form] = Form.useForm<Values>();
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();
  const [confusion, setConfusion] = useState<EncounterConfusionContextView | null>(
    null,
  );
  const [judgments, setJudgments] = useState<EncounterActionJudgmentView[]>([]);
  const [questions, setQuestions] = useState<EncounterActionIssueView[]>(
    option.issues.filter((issue) => issue.kind === "needs_fact" && issue.fact !== null),
  );
  const actionPreview = useQuery({
    queryKey: [
      "lifecycle-action-preview",
      encounterRef,
      participant.participant_key,
      revision.toString(),
      confusion,
      JSON.stringify(judgments, (_key, value) =>
        typeof value === "bigint" ? value.toString() : value,
      ),
    ],
    queryFn: () =>
      previewEncounterRetch(encounterRef, {
        participant_key: participant.participant_key,
        confusion,
        judgments,
      }),
    enabled: option.kind === "retch" && !changed,
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef &&
      query.queryKey[2] === participant.participant_key
        ? previous
        : undefined,
  });
  const issues =
    option.kind === "retch"
      ? (actionPreview.data?.issues ?? option.issues)
      : option.issues;
  const newQuestions = issues.filter(
    (issue) =>
      issue.kind === "needs_fact" &&
      issue.fact !== null &&
      !questions.some(
        (question) =>
          question.source_condition_id === issue.source_condition_id &&
          question.fact === issue.fact,
      ),
  );
  if (newQuestions.length) setQuestions([...questions, ...newQuestions]);
  const actionBlocked =
    option.kind === "retch" &&
    (actionPreview.isFetching ||
      !!actionPreview.error ||
      !actionPreview.data ||
      issues.length > 0 ||
      !!actionPreview.data.confusion?.required_facts.length ||
      !!actionPreview.data.confusion?.wastes_action);
  const qualifyingValue = Form.useWatch("qualifying", form);
  const checkMode = Form.useWatch("checkMode", form);
  const qualifies =
    qualifyingValue === undefined
      ? option.qualifying_damage
      : qualifyingValue === "yes";
  const has = (input: ConditionLifecycleOptionView["required_inputs"][number]) =>
    option.required_inputs.includes(input);
  const sourceLabel = (id: bigint) => {
    const source = participant.conditions.find((source) => source.condition_id === id);
    return source
      ? `${source.name}${source.value === undefined ? "" : ` ${source.value}`}`
      : `Source ${id}`;
  };
  function requiredId() {
    if (option.condition_id === null) throw new Error("Choose a condition source.");
    return option.condition_id;
  }
  function consequence() {
    if (option.consequence_key === null) throw new Error("Recovery event unavailable.");
    return option.consequence_key;
  }
  function build(values: Values): ConditionLifecycleActionView {
    const dc = values.dc === undefined ? option.dc : BigInt(values.dc);
    const modifier =
      values.modifier === undefined ? option.modifier : BigInt(values.modifier);
    switch (option.kind) {
      case "dying_recovery":
        return { kind: option.kind, check: lifecycleCheck(values) };
      case "retch":
        return {
          kind: option.kind,
          condition_id: requiredId(),
          dc,
          fortitude_modifier: modifier,
          actions_spent: values.actionsSpent === true,
          judgments,
          confusion,
          check: lifecycleCheck(values),
        };
      case "frightened_end_turn":
        return {
          kind: option.kind,
          decisions: option.source_condition_ids.flatMap((condition_id) => {
            const source = participant.conditions.find(
              (source) => source.condition_id === condition_id,
            );
            if (
              source?.parameters.kind !== "frightened" ||
              source.parameters.recovery.kind !== "source_dependent"
            )
              return [];
            const decision = values.frightened?.[condition_id.toString()];
            if (!decision?.reduce)
              throw new Error("Choose whether each source reduces.");
            return [
              {
                condition_id,
                reduce: decision.reduce === "yes",
                minimum:
                  decision.reduce === "no"
                    ? null
                    : decision.minimum === undefined
                      ? null
                      : BigInt(decision.minimum),
              },
            ];
          }),
        };
      case "treat_wounds":
        return { kind: option.kind, healing_received: BigInt(values.healing!) };
      case "stable_recovery":
      case "full_hp_rest":
        return { kind: option.kind, minutes: values.minutes! };
      case "overnight_rest":
        return { kind: option.kind };
      case "stunned_actions_lost":
        if (option.actions_lost === null)
          throw new Error("Action forecast unavailable.");
        return {
          kind: option.kind,
          condition_id: requiredId(),
          actions_lost: BigInt(option.actions_lost),
        };
      case "wake":
        if (!values.cause) throw new Error("Choose what wakes the participant.");
        return { kind: option.kind, cause: values.cause };
      case "wake_from_noise":
        if (dc === null) throw new Error("Enter the noise DC.");
        return {
          kind: option.kind,
          dc,
          perception_modifier: modifier,
          check: lifecycleCheck(values),
        };
      case "confused_damage_recovery":
        return {
          kind: option.kind,
          consequence_key: consequence(),
          condition_ids:
            qualifies === false || values.checkMode === "decline"
              ? []
              : (values.sources ?? []).map(BigInt),
          qualifying_damage: qualifies === true,
          check:
            qualifies === false || values.checkMode === "decline"
              ? null
              : lifecycleCheck(values),
        };
      case "resolve_broken":
        return {
          kind: option.kind,
          consequence_key: consequence(),
          threshold: BigInt(values.threshold!),
        };
    }
  }
  return (
    <>
      {changed && (
        <Alert
          type="warning"
          title="Encounter changed"
          action={
            <Button
              disabled={disabled}
              onClick={() => {
                form.resetFields();
                onReload();
              }}
            >
              Reload recovery
            </Button>
          }
        />
      )}
      {error && <Alert type="error" title="Recovery failed" description={error} />}
      <Form
        form={form}
        layout="vertical"
        disabled={disabled || saving || changed}
        initialValues={{ checkMode: "roll" }}
        onFinish={async (values) => {
          setError(undefined);
          setSaving(true);
          try {
            await onApply(build(values));
          } catch (value) {
            setError(value instanceof Error ? value.message : "Recovery failed.");
          } finally {
            setSaving(false);
          }
        }}
      >
        {option.kind === "retch" && (
          <ConfusionContextFields
            requirements={actionPreview.data?.confusion}
            value={confusion}
            participants={participants}
            onChange={setConfusion}
          />
        )}
        {(option.dc !== null || option.modifier !== null) && (
          <p>
            {option.dc === null ? "" : `DC ${option.dc}`}
            {option.dc !== null && option.modifier !== null ? " · " : ""}
            {option.modifier === null
              ? ""
              : `Modifier ${option.modifier >= 0n ? "+" : ""}${option.modifier}`}
          </p>
        )}
        {actionPreview.error && (
          <Alert
            type="error"
            title="Could not review action"
            action={<Button onClick={() => void actionPreview.refetch()}>Retry</Button>}
          />
        )}
        {questions.map((question) => (
          <Form.Item
            key={`${question.source_condition_id}:${question.fact}`}
            label={
              <>
                {question.reason}
                <ActionIssueContext source={question} participants={participants} />
              </>
            }
          >
            <Select
              aria-label={actionIssueQuestionLabel(
                question.reason,
                question,
                participants,
                question.source_condition_id,
              )}
              value={
                judgments.find(
                  (judgment) =>
                    judgment.source_condition_id === question.source_condition_id &&
                    judgment.fact === question.fact,
                )?.value === true
                  ? "yes"
                  : judgments.find(
                        (judgment) =>
                          judgment.source_condition_id ===
                            question.source_condition_id &&
                          judgment.fact === question.fact,
                      )?.value === false
                    ? "no"
                    : undefined
              }
              options={[
                { value: "yes", label: "Yes" },
                { value: "no", label: "No" },
              ]}
              onChange={(value) =>
                setJudgments((previous) => [
                  ...previous.filter(
                    (judgment) =>
                      judgment.source_condition_id !== question.source_condition_id ||
                      judgment.fact !== question.fact,
                  ),
                  {
                    source_condition_id: question.source_condition_id,
                    fact: question.fact!,
                    value: value === "yes",
                  },
                ])
              }
            />
          </Form.Item>
        ))}
        {issues
          .filter((issue) => issue.kind !== "needs_fact")
          .map((issue, index) => (
            <p key={index}>
              {actionIssueSourceLabel(issue, participants)}: {issue.reason}
            </p>
          ))}
        {has("source_dc") && (
          <Form.Item name="dc" label="DC" rules={[integerRule(true)]}>
            <InputNumber aria-label="Recovery DC" />
          </Form.Item>
        )}
        {(has("current_fortitude") || has("current_perception")) && (
          <Form.Item
            name="modifier"
            label={
              has("current_fortitude")
                ? "Current Fortitude modifier"
                : "Current Perception modifier"
            }
            rules={[signedInteger]}
          >
            <InputNumber aria-label="Recovery modifier" />
          </Form.Item>
        )}
        {has("actions_spent") && (
          <Form.Item
            name="actionsSpent"
            valuePropName="checked"
            rules={[
              {
                validator: (_rule, value) =>
                  value
                    ? Promise.resolve()
                    : Promise.reject(new Error("Confirm the actions were spent.")),
              },
            ]}
          >
            <Checkbox>Actions spent</Checkbox>
          </Form.Item>
        )}
        {has("rest_minutes") && (
          <Form.Item name="minutes" label="Minutes rested" rules={[integerRule(true)]}>
            <InputNumber aria-label="Minutes rested" min={0} />
          </Form.Item>
        )}
        {has("healing_received") && (
          <Form.Item
            name="healing"
            label="Healing received"
            rules={[integerRule(true)]}
          >
            <InputNumber aria-label="Healing received" min={0} />
          </Form.Item>
        )}
        {has("actions_lost") && option.actions_lost !== null && (
          <p>Actions lost: {option.actions_lost}</p>
        )}
        {has("threshold") && (
          <Form.Item
            name="threshold"
            label="Broken threshold"
            rules={[integerRule(true)]}
          >
            <InputNumber aria-label="Broken threshold" min={0} />
          </Form.Item>
        )}
        {option.kind === "wake" && (
          <Form.Item
            name="cause"
            label="Wake from"
            rules={[{ required: true, message: "Choose what wakes the participant." }]}
          >
            <Select
              aria-label="Wake cause"
              options={[
                { value: "interact", label: "Interact" },
                { value: "rest_interrupted", label: "Rest interrupted" },
                { value: "rest_completed", label: "Rest completed" },
              ]}
            />
          </Form.Item>
        )}
        {has("qualifying_damage") && (
          <Form.Item
            name="qualifying"
            label="Attack or spell damage?"
            rules={[
              { required: true, message: "Choose whether the damage qualifies." },
            ]}
          >
            <Select
              aria-label="Qualifying damage"
              options={[
                { value: "yes", label: "Yes" },
                { value: "no", label: "No" },
              ]}
            />
          </Form.Item>
        )}
        {has("source_selection") && qualifies !== false && checkMode !== "decline" && (
          <Form.Item
            name="sources"
            label="Sources"
            rules={[{ required: true, message: "Choose the affected sources." }]}
          >
            <Select
              mode="multiple"
              aria-label="Recovery sources"
              options={option.source_condition_ids.map((id) => ({
                value: id.toString(),
                label: sourceLabel(id),
              }))}
            />
          </Form.Item>
        )}
        {has("frightened_decisions") &&
          option.source_condition_ids.map((id) => {
            const source = participant.conditions.find(
              (source) => source.condition_id === id,
            );
            return source?.parameters.kind === "frightened" &&
              source.parameters.recovery.kind === "source_dependent" ? (
              <Space key={id.toString()} wrap>
                <Form.Item
                  name={["frightened", id.toString(), "reduce"]}
                  label={sourceLabel(id)}
                  rules={[
                    { required: true, message: "Choose whether this source reduces." },
                  ]}
                >
                  <Select
                    aria-label={`Reduce ${sourceLabel(id)}`}
                    options={[
                      { value: "yes", label: "Reduce" },
                      { value: "no", label: "Keep" },
                    ]}
                  />
                </Form.Item>
                <Form.Item
                  name={["frightened", id.toString(), "minimum"]}
                  label="Minimum"
                  dependencies={[["frightened", id.toString(), "reduce"]]}
                  rules={[
                    {
                      validator: (_rule, value) =>
                        form.getFieldValue(["frightened", id.toString(), "reduce"]) ===
                          "yes" && value == null
                          ? Promise.reject(new Error("Enter the source minimum."))
                          : Promise.resolve(),
                    },
                    integerRule(false),
                  ]}
                >
                  <InputNumber aria-label={`Minimum for ${sourceLabel(id)}`} min={0} />
                </Form.Item>
              </Space>
            ) : null;
          })}
        {has("check") &&
          (option.kind !== "confused_damage_recovery" || qualifies !== false) && (
            <LifecycleCheckFields
              form={form}
              allowDecline={option.kind === "confused_damage_recovery"}
            />
          )}
        <Button
          htmlType="submit"
          type="primary"
          loading={saving}
          disabled={disabled || changed || actionBlocked}
        >
          Record
        </Button>
      </Form>
    </>
  );
}
