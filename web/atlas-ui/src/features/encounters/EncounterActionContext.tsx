import { actionIssueQuestionLabel, actionIssueSourceLabel } from "./actionIssueLabels";
import { useQuery } from "@tanstack/react-query";
import {
  Alert,
  Button,
  Collapse,
  Form,
  InputNumber,
  Select,
  Space,
  Spin,
  Tag,
} from "antd";
import { useState } from "react";
import { previewEncounterAction } from "../../api/atlasApi";
import type {
  ConditionActionView,
  EncounterActionIssueView,
  EncounterActionJudgmentView,
  EncounterActionKindView,
  EncounterConfusionContextView,
  EncounterActionTargetView,
  EncounterParticipantView,
  PreviewEncounterActionRequest,
} from "../../generated/atlas";
import { ConfusionContextFields } from "./ConfusionContextFields";
import { TargetContextFields } from "./TargetContextFields";
import { ActionIssueContext } from "./ActionIssueContext";
import type { EditConditionSource } from "./conditionEditRequest";
type Mode = "action" | "extra" | "request" | "sensory" | "forced" | "cover" | "allies";
type Tri = "unknown" | "yes" | "no";
type Values = {
  mode: Mode;
  action?: EncounterActionKindView;
  extra?: ConditionActionView;
  other?: string;
  targetMode: "direct" | "area" | "guessed_location";
  reaction: Tri;
  reasonable: Tri;
  againstGoals: Tri;
  basis: NonNullable<PreviewEncounterActionRequest["sensory"]>["basis"];
  distance?: number | null;
  movement: Tri;
  holding: Tri;
  dc?: number | null;
  total?: number | null;
  ranged: Tri;
  normallyAlly: Tri;
};
const actionLabels: Record<EncounterActionKindView, string> = {
  strike: "Strike",
  cast: "Cast",
  escape: "Escape",
  force_open: "Force Open",
  stand: "Stand",
  crawl: "Crawl",
  seek: "Seek",
  delay: "Delay",
  ready: "Ready",
  retch: "Retch",
  request: "Request",
  traveling_exploration: "Exploration while traveling",
  other: "Other",
};
const extraLabels: Record<ConditionActionView, string> = {
  stride: "Stride",
  strike: "Strike",
  step: "Step",
  leap: "Leap",
  stand: "Stand",
  interact: "Interact",
  cast_spell: "Cast a Spell",
  sustain: "Sustain",
  escape: "Escape",
  crawl: "Crawl",
  fly: "Fly",
  swim: "Swim",
  climb: "Climb",
  burrow: "Burrow",
};
const tri = (value: Tri) => (value === "unknown" ? null : value === "yes");
const triOptions = [
  { value: "unknown", label: "Unspecified" },
  { value: "yes", label: "Yes" },
  { value: "no", label: "No" },
];
const integer = (value: number | null | undefined) =>
  value != null && Number.isSafeInteger(value) ? BigInt(value) : null;
export function EncounterActionContext({
  encounterRef,
  revision,
  participant,
  participants,
  disabled,
  onEditCondition,
  initialMode = "action",
  fixedOther,
  initiallyOpen = false,
  title = "Action context",
}: {
  encounterRef: string;
  revision: bigint;
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  disabled: boolean;
  onEditCondition?: EditConditionSource;
  initialMode?: Mode;
  fixedOther?: string;
  initiallyOpen?: boolean;
  title?: string;
}) {
  const initial: Values = {
    mode: initialMode,
    targetMode: "direct",
    reaction: "unknown",
    reasonable: "unknown",
    againstGoals: "unknown",
    basis: "general",
    movement: "unknown",
    holding: "unknown",
    ranged: "unknown",
    normallyAlly: "unknown",
  };
  const [confusion, setConfusion] = useState<EncounterConfusionContextView | null>(
    null,
  );
  const [targetContext, setTargetContext] = useState<
    Omit<EncounterActionTargetView, "participant_key" | "mode">
  >({
    vision_only_precise: null,
    distance_feet: null,
    guessed_location_correct: null,
    visual_effect: null,
    auditory_effect: null,
  });
  const [open, setOpen] = useState(initiallyOpen);
  const [values, setValues] = useState(initial);
  const [answers, setAnswers] = useState<
    Array<EncounterActionJudgmentView & { issue: EncounterActionIssueView }>
  >([]);
  const other = fixedOther ?? values.other;
  const request: PreviewEncounterActionRequest = {
    participant_key: participant.participant_key,
    normally_ally: values.mode === "allies" ? tri(values.normallyAlly) : null,
    confusion,
    action:
      values.mode === "request"
        ? "request"
        : values.mode === "action"
          ? (values.action ?? "other")
          : "other",
    reaction: values.mode === "action" ? tri(values.reaction) : null,
    target:
      (values.mode === "request" ||
        values.mode === "action" ||
        values.mode === "allies") &&
      other
        ? { participant_key: other, mode: values.targetMode, ...targetContext }
        : null,
    judgments: answers.map(({ source_condition_id, fact, value }) => ({
      source_condition_id,
      fact,
      value,
    })),
    extra_action: values.mode === "extra" ? (values.extra ?? null) : null,
    request_context:
      values.mode === "request"
        ? {
            reasonable: tri(values.reasonable),
            against_goals: tri(values.againstGoals),
          }
        : null,
    sensory:
      values.mode === "sensory"
        ? {
            basis: values.basis,
            distance_feet: values.distance ?? null,
            requires_movement: tri(values.movement),
          }
        : null,
    forced_movement:
      values.mode === "forced"
        ? {
            holding_in_place: tri(values.holding),
            difficulty_class: integer(values.dc),
            check_total: integer(values.total),
          }
        : null,
    prone_cover:
      values.mode === "cover" ? { against_ranged_attack: tri(values.ranged) } : null,
  };
  const identity = JSON.stringify(request, (_key, value) =>
    typeof value === "bigint" ? value.toString() : value,
  );
  const query = useQuery({
    queryKey: [
      "action-context",
      encounterRef,
      participant.participant_key,
      identity,
      revision.toString(),
    ],
    queryFn: () => previewEncounterAction(encounterRef, request),
    enabled:
      open &&
      (values.mode !== "action" || !!values.action) &&
      (values.mode !== "extra" || !!values.extra),
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef &&
      query.queryKey[2] === participant.participant_key &&
      query.queryKey[3] === identity
        ? previous
        : undefined,
  });
  const sourceNames = (ids: bigint[]) =>
    ids
      .map(
        (id) =>
          participants
            .flatMap((participant) => participant.conditions)
            .find((condition) => condition.condition_id === id)?.name ?? `Source ${id}`,
      )
      .join(", ");
  const issues = query.data?.issues ?? [];
  return (
    <Collapse
      ghost
      defaultActiveKey={initiallyOpen ? ["context"] : []}
      onChange={(keys) => setOpen(keys.includes("context"))}
      items={[
        {
          key: "context",
          label: title,
          children: (
            <>
              <Form
                layout="vertical"
                disabled={disabled}
                initialValues={initial}
                onValuesChange={(changed, all) => {
                  setAnswers([]);
                  if (
                    "other" in changed ||
                    "targetMode" in changed ||
                    "mode" in changed
                  )
                    setTargetContext({
                      vision_only_precise: null,
                      distance_feet: null,
                      guessed_location_correct: null,
                      visual_effect: null,
                      auditory_effect: null,
                    });
                  if ("action" in changed || "mode" in changed) setConfusion(null);
                  setValues((previous) => ({ ...previous, ...all }));
                }}
              >
                {!fixedOther && (
                  <Form.Item name="mode" label="Context">
                    <Select
                      aria-label="Action context type"
                      options={[
                        { value: "action", label: "Action" },
                        { value: "extra", label: "Extra action" },
                        { value: "request", label: "Request" },
                        { value: "sensory", label: "Perception and senses" },
                        { value: "forced", label: "Forced movement" },
                        { value: "cover", label: "Prone cover" },
                        { value: "allies", label: "Allies and flanking" },
                      ]}
                    />
                  </Form.Item>
                )}
                {values.mode === "action" && (
                  <Form.Item name="action" label="Action">
                    <Select
                      aria-label="Action to check"
                      placeholder="Choose action"
                      options={Object.entries(actionLabels)
                        .filter(([value]) => value !== "request")
                        .map(([value, label]) => ({
                          value,
                          label,
                        }))}
                    />
                  </Form.Item>
                )}
                {values.mode === "action" && (
                  <Form.Item name="reaction" label="Reaction? ">
                    <Select aria-label="Is this a reaction" options={triOptions} />
                  </Form.Item>
                )}
                {values.mode === "extra" && (
                  <Form.Item name="extra" label="Extra action">
                    <Select
                      aria-label="Extra action to check"
                      placeholder="Choose action"
                      options={Object.entries(extraLabels).map(([value, label]) => ({
                        value,
                        label,
                      }))}
                    />
                  </Form.Item>
                )}
                {(values.mode === "action" ||
                  values.mode === "request" ||
                  values.mode === "allies") &&
                  !fixedOther && (
                    <Form.Item
                      name="other"
                      label={values.mode === "request" ? "Recipient" : "Target"}
                    >
                      <Select
                        allowClear
                        aria-label="Action other participant"
                        placeholder="Choose participant"
                        options={participants.map((participant) => ({
                          value: participant.participant_key,
                          label: participant.display_name,
                        }))}
                      />
                    </Form.Item>
                  )}
                {values.mode === "allies" && (
                  <Form.Item name="normallyAlly" label="Normally an ally">
                    <Select aria-label="Normally an ally" options={triOptions} />
                  </Form.Item>
                )}
                {values.mode === "action" && other && (
                  <Form.Item name="targetMode" label="Targeting">
                    <Select
                      aria-label="Action targeting"
                      options={[
                        { value: "direct", label: "Direct" },
                        { value: "area", label: "Area" },
                        { value: "guessed_location", label: "Guessed location" },
                      ]}
                    />
                  </Form.Item>
                )}
                {values.mode === "request" && (
                  <Space wrap>
                    <Form.Item name="reasonable" label="Reasonable request?">
                      <Select aria-label="Reasonable request" options={triOptions} />
                    </Form.Item>
                    <Form.Item name="againstGoals" label="Against recipient’s goals?">
                      <Select
                        aria-label="Against recipient goals"
                        options={triOptions}
                      />
                    </Form.Item>
                  </Space>
                )}
                {values.mode === "sensory" && (
                  <>
                    <Form.Item name="basis" label="Perception check">
                      <Select
                        aria-label="Perception basis"
                        options={[
                          { value: "general", label: "General" },
                          { value: "visual_only", label: "Visual only" },
                          { value: "auditory_only", label: "Auditory only" },
                          { value: "mixed_sound", label: "Mixed, involving sound" },
                          { value: "initiative", label: "Initiative" },
                        ]}
                      />
                    </Form.Item>
                    <Space wrap>
                      <Form.Item name="distance" label="Distance (feet)">
                        <InputNumber aria-label="Sensory distance" min={0} />
                      </Form.Item>
                      <Form.Item name="movement" label="Requires movement?">
                        <Select
                          aria-label="Sensory check requires movement"
                          options={triOptions}
                        />
                      </Form.Item>
                    </Space>
                  </>
                )}
                {values.mode === "forced" && (
                  <>
                    <Form.Item name="holding" label="Held in place?">
                      <Select aria-label="Held in place" options={triOptions} />
                    </Form.Item>
                    <Space wrap>
                      <Form.Item name="dc" label="Holding effect DC">
                        <InputNumber aria-label="Holding effect DC" precision={0} />
                      </Form.Item>
                      <Form.Item name="total" label="Check total">
                        <InputNumber
                          aria-label="Forced movement check total"
                          precision={0}
                        />
                      </Form.Item>
                    </Space>
                  </>
                )}
                {values.mode === "cover" && (
                  <Form.Item name="ranged" label="Against a ranged attack?">
                    <Select
                      aria-label="Cover against ranged attack"
                      options={triOptions}
                    />
                  </Form.Item>
                )}
              </Form>
              {query.error && (
                <Alert
                  type="error"
                  title="Could not check action context"
                  action={<Button onClick={() => void query.refetch()}>Retry</Button>}
                />
              )}
              <Spin spinning={query.isFetching} delay={300}>
                <div aria-busy={query.isFetching}>
                  <ConfusionContextFields
                    requirements={query.data?.confusion}
                    value={confusion}
                    participants={participants}
                    onChange={setConfusion}
                  />
                  <TargetContextFields
                    targeting={query.data?.targeting}
                    target={request.target}
                    onChange={(value) =>
                      setTargetContext({
                        vision_only_precise: value.vision_only_precise,
                        distance_feet: value.distance_feet,
                        guessed_location_correct: value.guessed_location_correct,
                        visual_effect: value.visual_effect,
                        auditory_effect: value.auditory_effect,
                      })
                    }
                  />
                  {answers.map((answer) => (
                    <Form.Item
                      key={`${answer.source_condition_id}:${answer.fact}`}
                      label={
                        <>
                          {answer.issue.reason}
                          <ActionIssueContext
                            source={answer.issue}
                            participants={participants}
                          />
                        </>
                      }
                    >
                      <Select
                        aria-label={actionIssueQuestionLabel(
                          answer.issue.reason,
                          answer.issue,
                          participants,
                          answer.source_condition_id,
                        )}
                        value={answer.value ? "yes" : "no"}
                        disabled={disabled}
                        options={[
                          { value: "yes", label: "Yes" },
                          { value: "no", label: "No" },
                        ]}
                        onChange={(value) =>
                          setAnswers((previous) =>
                            previous.map((current) =>
                              current.source_condition_id ===
                                answer.source_condition_id &&
                              current.fact === answer.fact
                                ? { ...current, value: value === "yes" }
                                : current,
                            ),
                          )
                        }
                      />
                    </Form.Item>
                  ))}
                  {issues
                    .filter(
                      (issue) =>
                        issue.kind !== "needs_fact" ||
                        !answers.some(
                          (answer) =>
                            answer.source_condition_id === issue.source_condition_id &&
                            answer.fact === issue.fact,
                        ),
                    )
                    .map((issue, index) =>
                      issue.kind === "needs_fact" && issue.fact !== null ? (
                        <Form.Item
                          key={`${issue.source_condition_id}:${issue.fact}`}
                          label={
                            <>
                              {issue.reason}
                              <ActionIssueContext
                                source={issue}
                                participants={participants}
                              />
                            </>
                          }
                        >
                          <Select
                            aria-label={actionIssueQuestionLabel(
                              issue.reason,
                              issue,
                              participants,
                              issue.source_condition_id,
                            )}
                            disabled={disabled}
                            placeholder="Choose…"
                            options={[
                              { value: "yes", label: "Yes" },
                              { value: "no", label: "No" },
                            ]}
                            onChange={(value) =>
                              setAnswers((previous) => [
                                ...previous,
                                {
                                  source_condition_id: issue.source_condition_id,
                                  fact: issue.fact!,
                                  value: value === "yes",
                                  issue,
                                },
                              ])
                            }
                          />
                        </Form.Item>
                      ) : (
                        <p key={index}>
                          {actionIssueSourceLabel(issue, participants)}: {issue.reason}
                        </p>
                      ),
                    )}
                  {query.data &&
                    values.mode === "action" &&
                    issues.length === 0 &&
                    !query.data.disruption &&
                    !query.data.confusion && <Tag>No condition restriction</Tag>}
                  {query.data?.disruption && (
                    <p>
                      Check DC {query.data.disruption.dc.toString()} ·{" "}
                      {query.data.disruption.sources
                        .map((source) => source.name)
                        .join(", ")}
                    </p>
                  )}
                  {query.data?.targeting && (
                    <>
                      {query.data.targeting.flat_check_dc !== null && (
                        <p>
                          Targeting check DC{" "}
                          {query.data.targeting.flat_check_dc.toString()}
                        </p>
                      )}
                      {query.data.targeting.requires_guessed_location && (
                        <p>Choose a guessed location.</p>
                      )}
                    </>
                  )}
                  {values.mode === "allies" && query.data && (
                    <>
                      <p>Can flank: {query.data.can_flank ? "Yes" : "No"}</p>
                      {other && (
                        <p>
                          Considers target an ally:{" "}
                          {query.data.considers_target_ally === null
                            ? "Unspecified"
                            : query.data.considers_target_ally
                              ? "Yes"
                              : "No"}
                        </p>
                      )}
                    </>
                  )}
                  {query.data?.quickened_use && (
                    <>
                      <p>
                        Extra action:{" "}
                        {query.data.quickened_use.allowed === true
                          ? "Allowed"
                          : query.data.quickened_use.allowed === false
                            ? "Not allowed"
                            : "Needs context"}
                      </p>
                      {query.data.quickened_use.allowing_source_ids.length > 0 && (
                        <p>
                          {sourceNames(query.data.quickened_use.allowing_source_ids)}
                        </p>
                      )}
                      {query.data.quickened_use.unknown_source_ids.length > 0 && (
                        <p>
                          Unspecified:{" "}
                          {sourceNames(query.data.quickened_use.unknown_source_ids)}
                        </p>
                      )}
                    </>
                  )}
                  {query.data?.request && (
                    <>
                      <p>
                        {query.data.request.outcome === "check_required"
                          ? "Request check required"
                          : query.data.request.outcome === "prohibited"
                            ? "Request prohibited"
                            : "Request needs context"}
                      </p>
                      <p>{query.data.request.reason}</p>
                      {query.data.request.required_facts.map((fact) => (
                        <p key={fact}>{fact}</p>
                      ))}
                    </>
                  )}
                  {query.data?.sensory && (
                    <>
                      <p>
                        Perception:{" "}
                        {query.data.sensory.perception === null
                          ? "Unknown"
                          : query.data.sensory.perception}
                      </p>
                      <Space wrap>
                        {query.data.sensory.critical_failure && (
                          <Tag>Critical failure</Tag>
                        )}
                        {query.data.sensory.visual_effects_immune && (
                          <Tag>Immune to visual effects</Tag>
                        )}
                        {query.data.sensory.auditory_effects_immune && (
                          <Tag>Immune to auditory effects</Tag>
                        )}
                        {query.data.sensory.all_terrain_difficult && (
                          <Tag>All terrain is difficult terrain</Tag>
                        )}
                      </Space>
                      {query.data.sensory.required_facts.map((fact) => (
                        <p key={fact}>{fact}</p>
                      ))}
                    </>
                  )}
                  {query.data?.forced_movement && (
                    <>
                      <p>
                        Forced movement:{" "}
                        {
                          {
                            unrestricted: "Unrestricted",
                            needs_check: "Check required",
                            check_failed: "Check failed",
                            check_succeeded: "Check succeeded",
                            needs_context: "Needs context",
                          }[query.data.forced_movement.outcome]
                        }
                        {query.data.forced_movement.difficulty_class !== null
                          ? ` · DC ${query.data.forced_movement.difficulty_class}`
                          : ""}
                      </p>
                      {query.data.forced_movement.required_facts.map((fact) => (
                        <p key={fact}>{fact}</p>
                      ))}
                    </>
                  )}
                  {query.data?.prone_cover && (
                    <>
                      <p>
                        AC:{" "}
                        {query.data.prone_cover.armor_class === null
                          ? "Unknown"
                          : query.data.prone_cover.armor_class}
                      </p>
                      {query.data.prone_cover.required_facts.map((fact) => (
                        <p key={fact}>{fact}</p>
                      ))}
                    </>
                  )}
                  {onEditCondition &&
                    issues
                      .filter(
                        (issue, index) =>
                          issues.findIndex(
                            (other) =>
                              other.source_condition_id === issue.source_condition_id,
                          ) === index,
                      )
                      .map((issue) => {
                        const owner = participants.find((participant) =>
                          participant.conditions.some(
                            (condition) =>
                              condition.condition_id === issue.source_condition_id,
                          ),
                        );
                        return owner ? (
                          <Button
                            key={`${issue.source_condition_id}:${issue.fact}`}
                            type="link"
                            size="small"
                            aria-label={`Edit ${actionIssueSourceLabel(issue, participants)} source`}
                            onClick={() =>
                              onEditCondition(
                                owner.participant_key,
                                issue.source_condition_id,
                              )
                            }
                          >
                            Edit {issue.condition_name} source
                          </Button>
                        ) : null;
                      })}
                </div>
              </Spin>
            </>
          ),
        },
      ]}
    />
  );
}
