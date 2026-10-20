import { EncounterModal } from "./EncounterModal";
import { useQuery } from "@tanstack/react-query";
import {
  Alert,
  Button,
  Collapse,
  Form,
  Input,
  Select,
  Space,
  Tag,
  Tooltip,
} from "antd";
import { useState } from "react";
import { EncounterActionContext } from "./EncounterActionContext";
import { EncounterDetectionEventForm } from "./EncounterDetectionEventForm";
import { getEncounterRelationships } from "../../api/atlasApi";
import type {
  EncounterParticipantView,
  ApplyEncounterDetectionEventRequest,
  DetectionEventView,
  EncounterRelationshipView,
  RelationshipKnowledgeView,
  RelationshipPropertyView,
  RelationshipProvenanceView,
  RelationshipValueView,
  SetEncounterRelationshipRequest,
  WorsenEncounterAttitudeRequest,
} from "../../generated/atlas";
const titles: Record<RelationshipPropertyView, string> = {
  detection: "Detection",
  awareness: "Awareness",
  attitude: "Attitude",
  control: "Control",
};
function valueLabel(
  value: RelationshipValueView,
  participants: EncounterParticipantView[],
): string {
  switch (value.kind) {
    case "detection":
      return { observed: "Observed", hidden: "Hidden", undetected: "Undetected" }[
        value.value
      ];
    case "awareness":
      return value.value ? "Aware" : "Unaware";
    case "attitude":
      return value.value[0].toUpperCase() + value.value.slice(1);
    case "no_control":
      return "Not controlled";
    case "control":
      return `Controlled by ${participants.find((participant) => participant.participant_key === value.value.controller_key)?.display_name ?? value.value.controller_key}`;
  }
}
function sourceLabel(
  source: RelationshipProvenanceView,
  participants: EncounterParticipantView[],
): string {
  switch (source.kind) {
    case "table":
      return "Table";
    case "sources":
      return source.condition_ids
        .map(
          (id) =>
            participants
              .flatMap((participant) => participant.conditions)
              .find((condition) => condition.condition_id === id)?.name ??
            `Source ${id}`,
        )
        .join(", ");
    case "source_default":
      return "Source default";
    case "source":
      return (
        participants
          .flatMap((participant) => participant.conditions)
          .find((condition) => condition.condition_id === source.condition_id)?.name ??
        source.record_key ??
        "Source effect"
      );
  }
}
function Knowledge({
  knowledge,
  participants,
}: {
  knowledge: RelationshipKnowledgeView;
  participants: EncounterParticipantView[];
}) {
  switch (knowledge.kind) {
    case "known":
      return (
        <Space size="small" wrap>
          <span>{valueLabel(knowledge.value, participants)}</span>
          <Tag>{sourceLabel(knowledge.provenance, participants)}</Tag>
        </Space>
      );
    case "unknown":
      return (
        <Tooltip
          trigger={["hover", "focus", "click"]}
          title={knowledge.required_facts.join(" · ")}
        >
          <Button type="text" size="small" style={{ padding: 0, height: "auto" }}>
            Unknown
          </Button>
        </Tooltip>
      );
    case "invalidated":
      return (
        <Tooltip
          trigger={["hover", "focus", "click"]}
          title={sourceLabel(knowledge.prior_source, participants)}
        >
          <Button type="text" size="small" style={{ padding: 0, height: "auto" }}>
            Source changed
          </Button>
        </Tooltip>
      );
    case "conflicting":
      return (
        <Space size="small" wrap>
          <Tag color="warning">Conflicting</Tag>
          {knowledge.candidates.map((candidate, index) => (
            <Tooltip
              key={index}
              trigger={["hover", "focus", "click"]}
              title={sourceLabel(candidate.provenance, participants)}
            >
              <Button type="text" size="small" style={{ padding: 0, height: "auto" }}>
                {valueLabel(candidate.value, participants)}
              </Button>
            </Tooltip>
          ))}
        </Space>
      );
  }
}
function SourceConditionLinks({
  row,
  participants,
  onEdit,
}: {
  row: EncounterRelationshipView;
  participants: EncounterParticipantView[];
  onEdit: import("./conditionEditRequest").EditConditionSource | undefined;
}) {
  if (!onEdit) return null;
  return (
    <Space size="small" wrap>
      {row.source_condition_ids.map((id) => {
        const owner = participants.find((participant) =>
          participant.conditions.some((condition) => condition.condition_id === id),
        );
        const condition = owner?.conditions.find(
          (condition) => condition.condition_id === id,
        );
        return owner && condition ? (
          <Button
            key={id.toString()}
            type="link"
            size="small"
            onClick={() => onEdit(owner.participant_key, id)}
          >
            Edit {condition.name} source
          </Button>
        ) : null;
      })}
    </Space>
  );
}
export function EncounterRelationships({
  encounterRef,
  revision,
  subject,
  participants,
  disabled,
  onSave,
  onEvent,
  onEditCondition,
  onWorsen,
  initialOtherKey,
  initiallyOpen = false,
  title = "Relationships",
  properties,
}: {
  encounterRef: string;
  revision: bigint;
  subject: EncounterParticipantView;
  participants: EncounterParticipantView[];
  disabled: boolean;
  onSave: (
    request: Omit<SetEncounterRelationshipRequest, "request_id">,
  ) => Promise<boolean>;
  onEvent?: (
    request: Omit<ApplyEncounterDetectionEventRequest, "request_id">,
  ) => Promise<boolean>;
  onEditCondition?: import("./conditionEditRequest").EditConditionSource;
  onWorsen?: (
    request: Omit<WorsenEncounterAttitudeRequest, "request_id">,
  ) => Promise<boolean>;
  initialOtherKey?: string;
  initiallyOpen?: boolean;
  title?: string;
  properties?: RelationshipPropertyView[];
}) {
  const [requestOpen, setRequestOpen] = useState(false);
  const [worsen, setWorsen] = useState<{
    previous: WorsenEncounterAttitudeRequest["previous"];
    revision: bigint;
  } | null>(null);
  const [worsenError, setWorsenError] = useState<string>();
  const [other, setOther] = useState<string | undefined>(initialOtherKey);
  const [event, setEvent] = useState<{
    kind: "seek" | "became_invisible";
    revision: bigint;
    prior?: Extract<DetectionEventView, { kind: "seek" }>["prior"];
  } | null>(null);
  const [eventError, setEventError] = useState<string>();
  const [editing, setEditing] = useState<{
    row: EncounterRelationshipView;
    revision: bigint;
    layer: "baseline" | "adjudication";
  } | null>(null);
  const [committed, setCommitted] = useState(false);
  const query = useQuery({
    queryKey: [
      "relationships",
      encounterRef,
      subject.participant_key,
      other,
      revision.toString(),
    ],
    queryFn: () =>
      getEncounterRelationships(encounterRef, subject.participant_key, other!),
    enabled: !!other,
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef &&
      query.queryKey[2] === subject.participant_key &&
      query.queryKey[3] === other
        ? previous
        : undefined,
  });
  const otherName =
    participants.find((participant) => participant.participant_key === other)
      ?.display_name ?? "participant";
  const detection = query.data?.relationships.find(
    (row) => row.property === "detection",
  )?.knowledge;
  const prior =
    detection?.kind === "known" && detection.value.kind === "detection"
      ? detection.value.value
      : undefined;
  const editSource = onEditCondition
    ? (participantKey: string, conditionId: bigint) => {
        setEditing(null);
        setEvent(null);
        setRequestOpen(false);
        setWorsen(null);
        onEditCondition(participantKey, conditionId);
      }
    : undefined;
  const recordEvent = async (value: DetectionEventView, expectedRevision: bigint) => {
    if (!onEvent || !other) return;
    setEventError(undefined);
    try {
      const refreshed = await onEvent({
        expected_encounter_revision: expectedRevision,
        subject_key: subject.participant_key,
        observer_key: other,
        event: value,
      });
      if (refreshed) {
        setEvent(null);
        void query.refetch();
      } else setCommitted(true);
    } catch (error) {
      setEventError(
        error instanceof Error ? error.message : "Could not record detection event.",
      );
      throw error;
    }
  };
  return (
    <>
      <Collapse
        ghost
        defaultActiveKey={initiallyOpen ? ["relationships"] : []}
        items={[
          {
            key: "relationships",
            label: title,
            children: (
              <>
                <Select
                  allowClear
                  aria-label="Relative to participant"
                  placeholder="Relative to…"
                  value={other}
                  options={participants
                    .filter(
                      (participant) =>
                        participant.participant_key !== subject.participant_key,
                    )
                    .map((participant) => ({
                      value: participant.participant_key,
                      label: participant.display_name,
                    }))}
                  onChange={setOther}
                />
                {other &&
                  query.data &&
                  (!properties || properties.includes("attitude")) && (
                    <Button
                      size="small"
                      disabled={disabled || query.isFetching || !!query.error}
                      onClick={() => setRequestOpen(true)}
                    >
                      Request
                    </Button>
                  )}
                {other && onEvent && query.data && (
                  <Space wrap>
                    <Button
                      size="small"
                      disabled={disabled || query.isFetching || !!query.error}
                      onClick={() => {
                        setCommitted(false);
                        setEvent({
                          kind: "seek",
                          revision: query.data!.expected_revision,
                          prior,
                        });
                      }}
                    >
                      Seek
                    </Button>
                    <Button
                      size="small"
                      disabled={disabled || query.isFetching || !!query.error}
                      onClick={() => {
                        setCommitted(false);
                        setEvent({
                          kind: "became_invisible",
                          revision: query.data!.expected_revision,
                        });
                      }}
                    >
                      Became invisible
                    </Button>
                    <Button
                      size="small"
                      disabled={disabled || query.isFetching || !!query.error}
                      onClick={() =>
                        void recordEvent(
                          { kind: "became_aware" },
                          query.data!.expected_revision,
                        ).catch(() => {})
                      }
                    >
                      Became aware
                    </Button>
                  </Space>
                )}
                {eventError && !event && (
                  <Alert
                    type="error"
                    title="Detection event failed"
                    description={eventError}
                  />
                )}
                {other && query.error && (
                  <Alert
                    type="error"
                    title="Could not load relationships"
                    action={<Button onClick={() => void query.refetch()}>Retry</Button>}
                  />
                )}
                {other &&
                  query.data?.relationships
                    .filter((row) => !properties || properties.includes(row.property))
                    .map((row) => (
                      <div key={row.property}>
                        <Space wrap>
                          <span>
                            {row.property === "detection"
                              ? `Seen by ${otherName}`
                              : row.property === "awareness"
                                ? `Known to ${otherName}`
                                : row.property === "attitude"
                                  ? `Attitude toward ${otherName}`
                                  : `Control by ${otherName}`}
                          </span>
                          <Knowledge
                            knowledge={row.knowledge}
                            participants={participants}
                          />
                          {onWorsen &&
                            row.property === "attitude" &&
                            row.knowledge.kind === "known" &&
                            row.knowledge.value.kind === "attitude" &&
                            row.knowledge.value.value !== "hostile" && (
                              <Button
                                type="link"
                                size="small"
                                disabled={disabled || query.isFetching || !!query.error}
                                onClick={() => {
                                  if (
                                    row.knowledge.kind !== "known" ||
                                    row.knowledge.value.kind !== "attitude"
                                  )
                                    return;
                                  setCommitted(false);
                                  setWorsen({
                                    previous: row.knowledge.value.value,
                                    revision: query.data!.expected_revision,
                                  });
                                  setWorsenError(undefined);
                                }}
                              >
                                Worsen attitude
                              </Button>
                            )}
                          {row.source_condition_ids.length > 0 && (
                            <Button
                              type="link"
                              size="small"
                              disabled={disabled || query.isFetching || !!query.error}
                              aria-label={`Resolve ${titles[row.property].toLowerCase()}`}
                              onClick={() => {
                                setCommitted(false);
                                setEditing({
                                  row,
                                  revision: query.data!.expected_revision,
                                  layer: "adjudication",
                                });
                              }}
                            >
                              {row.adjudication ? "Reassess" : "Resolve"}
                            </Button>
                          )}
                          <SourceConditionLinks
                            row={row}
                            participants={participants}
                            onEdit={editSource}
                          />
                          <Button
                            aria-label={`Edit ${titles[row.property].toLowerCase()} baseline`}
                            type="link"
                            size="small"
                            disabled={disabled || query.isFetching || !!query.error}
                            onClick={() => {
                              setCommitted(false);
                              setEditing({
                                row,
                                layer: "baseline",
                                revision: query.data!.expected_revision,
                              });
                            }}
                          >
                            Edit baseline
                          </Button>
                        </Space>
                      </div>
                    ))}
              </>
            ),
          },
        ]}
      />
      <EncounterModal
        title={`${otherName} requests ${subject.display_name}`}
        open={requestOpen}
        footer={null}
        destroyOnHidden
        onCancel={() => setRequestOpen(false)}
      >
        {participants.find((participant) => participant.participant_key === other) && (
          <EncounterActionContext
            key={`${other}:${subject.participant_key}`}
            encounterRef={encounterRef}
            revision={revision}
            participant={
              participants.find((participant) => participant.participant_key === other)!
            }
            participants={participants}
            disabled={disabled}
            onEditCondition={editSource}
            initialMode="request"
            fixedOther={subject.participant_key}
            initiallyOpen
            title="Request"
          />
        )}
      </EncounterModal>
      <EncounterModal
        title={`Worsen attitude toward ${otherName}`}
        open={!!worsen && !(committed && !disabled)}
        footer={null}
        destroyOnHidden
        closable={!disabled}
        keyboard={!disabled}
        mask={{ closable: !disabled }}
        onCancel={() => {
          if (!disabled) setWorsen(null);
        }}
      >
        {worsenError && (
          <Alert
            type="error"
            title="Could not worsen attitude"
            description={worsenError}
          />
        )}
        {worsen && worsen.revision !== revision && (
          <Alert
            type="warning"
            title="Encounter changed"
            action={<Button onClick={() => setWorsen(null)}>Close and review</Button>}
          />
        )}
        <Form
          layout="vertical"
          disabled={disabled || committed || worsen?.revision !== revision}
          onFinish={async (values: {
            attitude: WorsenEncounterAttitudeRequest["attitude"];
          }) => {
            if (!onWorsen || !worsen || !other) return;
            setWorsenError(undefined);
            try {
              const refreshed = await onWorsen({
                expected_encounter_revision: worsen.revision,
                subject_key: subject.participant_key,
                other_key: other,
                previous: worsen.previous,
                attitude: values.attitude,
              });
              if (refreshed) {
                setWorsen(null);
                void query.refetch();
              } else setCommitted(true);
            } catch (error) {
              setWorsenError(
                error instanceof Error ? error.message : "Could not worsen attitude.",
              );
            }
          }}
        >
          <Form.Item
            name="attitude"
            label="Attitude"
            rules={[{ required: true, message: "Choose the worse attitude." }]}
          >
            <Select
              aria-label="Worse attitude"
              options={[
                { value: "helpful", label: "Helpful" },
                { value: "friendly", label: "Friendly" },
                { value: "indifferent", label: "Indifferent" },
                { value: "unfriendly", label: "Unfriendly" },
                { value: "hostile", label: "Hostile" },
              ]}
            />
          </Form.Item>
          <Button
            type="primary"
            htmlType="submit"
            disabled={disabled || committed || worsen?.revision !== revision}
          >
            Worsen attitude
          </Button>
        </Form>
      </EncounterModal>
      <EncounterModal
        title={
          event?.kind === "seek"
            ? `${otherName} seeks ${subject.display_name}`
            : `${subject.display_name} became invisible`
        }
        open={!!event && !(committed && !disabled)}
        footer={null}
        destroyOnHidden
        closable={!disabled}
        keyboard={!disabled}
        mask={{ closable: !disabled }}
        onCancel={() => {
          if (!disabled) setEvent(null);
        }}
      >
        {event && (
          <EncounterDetectionEventForm
            key={`${event.kind}:${event.revision}`}
            kind={event.kind}
            subjectName={subject.display_name}
            prior={event.prior}
            disabled={disabled || committed}
            changed={event.revision !== revision}
            onReload={() =>
              setEvent({ ...event, revision: query.data!.expected_revision, prior })
            }
            onApply={(value) => recordEvent(value, event.revision)}
          />
        )}
      </EncounterModal>
      <EncounterModal
        title={
          editing
            ? `${titles[editing.row.property]} ${editing.layer === "baseline" ? "baseline" : "resolution"}`
            : "Relationship"
        }
        open={!!editing && !(committed && !disabled)}
        destroyOnHidden
        footer={null}
        closable={!disabled}
        keyboard={!disabled}
        mask={{ closable: !disabled }}
        onCancel={() => {
          if (!disabled) setEditing(null);
        }}
      >
        {editing && editing.row.source_condition_ids.length > 0 && (
          <>
            <p>
              Current:{" "}
              <Knowledge
                knowledge={editing.row.knowledge}
                participants={participants}
              />
            </p>
            <SourceConditionLinks
              row={editing.row}
              participants={participants}
              onEdit={editSource}
            />
          </>
        )}
        {editing && (
          <RelationshipForm
            key={`${editing.row.subject_key}:${editing.row.other_key}:${editing.row.property}:${editing.revision}`}
            row={editing.row}
            layer={editing.layer}
            otherName={otherName}
            changed={editing.revision !== revision}
            disabled={disabled || committed}
            onReload={() => {
              const row = query.data?.relationships.find(
                (row) => row.property === editing.row.property,
              );
              if (row)
                setEditing({
                  row,
                  revision: query.data!.expected_revision,
                  layer: editing.layer,
                });
            }}
            onSave={async (knowledge) => {
              const refreshed = await onSave({
                expected_encounter_revision: editing.revision,
                subject_key: editing.row.subject_key,
                other_key: editing.row.other_key,
                property: editing.row.property,
                layer: editing.layer,
                knowledge,
              });
              if (refreshed) {
                setEditing(null);
                void query.refetch();
              } else setCommitted(true);
            }}
          />
        )}
      </EncounterModal>
    </>
  );
}
function RelationshipForm({
  row,
  layer,
  otherName,
  changed,
  disabled,
  onReload,
  onSave,
}: {
  row: EncounterRelationshipView;
  layer: "baseline" | "adjudication";
  otherName: string;
  changed: boolean;
  disabled: boolean;
  onReload: () => void;
  onSave: (knowledge: RelationshipKnowledgeView | null) => Promise<void>;
}) {
  const [form] = Form.useForm<{ choice: string; instructions?: string }>();
  const [error, setError] = useState<string>();
  const initialKnowledge = layer === "baseline" ? row.baseline : row.adjudication;
  const known = initialKnowledge?.kind === "known" ? initialKnowledge.value : null;
  const choice =
    initialKnowledge?.kind === "unknown"
      ? "unknown"
      : known?.kind === "detection" || known?.kind === "attitude"
        ? known.value
        : known?.kind === "awareness"
          ? String(known.value)
          : known?.kind === "control"
            ? "control"
            : known?.kind === "no_control"
              ? "no_control"
              : undefined;
  const selected = Form.useWatch("choice", form) ?? choice;
  const options =
    row.property === "detection"
      ? [
          { value: "observed", label: "Observed" },
          { value: "hidden", label: "Hidden" },
          { value: "undetected", label: "Undetected" },
        ]
      : row.property === "awareness"
        ? [
            { value: "true", label: "Aware" },
            { value: "false", label: "Unaware" },
          ]
        : row.property === "attitude"
          ? [
              { value: "helpful", label: "Helpful" },
              { value: "friendly", label: "Friendly" },
              { value: "indifferent", label: "Indifferent" },
              { value: "unfriendly", label: "Unfriendly" },
              { value: "hostile", label: "Hostile" },
            ]
          : [
              ...(layer === "adjudication"
                ? [{ value: "control", label: `Controlled by ${otherName}` }]
                : []),
              { value: "no_control", label: `Not controlled by ${otherName}` },
            ];
  async function save(knowledge: RelationshipKnowledgeView | null) {
    setError(undefined);
    try {
      await onSave(knowledge);
    } catch (value) {
      setError(value instanceof Error ? value.message : "Could not save relationship.");
    }
  }
  return (
    <>
      {changed && (
        <Alert
          type="warning"
          title="Encounter changed"
          action={
            <Button disabled={disabled} onClick={onReload}>
              Reload relationship
            </Button>
          }
        />
      )}
      {error && <Alert type="error" title="Relationship failed" description={error} />}
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          choice,
          instructions:
            known?.kind === "control" ? (known.value.instructions ?? "") : "",
        }}
        disabled={disabled || changed}
        onFinish={(values) => {
          if (values.choice === "unknown") {
            void save({ kind: "unknown", required_facts: [] });
            return;
          }
          let value: RelationshipValueView;
          if (row.property === "detection")
            value = {
              kind: "detection",
              value: values.choice as Extract<
                RelationshipValueView,
                { kind: "detection" }
              >["value"],
            };
          else if (row.property === "awareness")
            value = { kind: "awareness", value: values.choice === "true" };
          else if (row.property === "attitude")
            value = {
              kind: "attitude",
              value: values.choice as Extract<
                RelationshipValueView,
                { kind: "attitude" }
              >["value"],
            };
          else
            value =
              values.choice === "no_control"
                ? { kind: "no_control" }
                : {
                    kind: "control",
                    value: {
                      controller_key: row.other_key,
                      instructions: values.instructions?.trim() || null,
                    },
                  };
          void save({ kind: "known", value, provenance: { kind: "table" } });
        }}
      >
        <Form.Item
          name="choice"
          label="State"
          rules={[{ required: true, message: "Choose the relationship state." }]}
        >
          <Select
            aria-label="Relationship state"
            options={
              layer === "baseline"
                ? [...options, { value: "unknown", label: "Unknown" }]
                : options
            }
          />
        </Form.Item>
        {row.property === "control" && selected === "control" && (
          <Form.Item name="instructions" label="Instructions">
            <Input.TextArea
              aria-label="Control instructions"
              autoSize={{ minRows: 2, maxRows: 4 }}
            />
          </Form.Item>
        )}
        <Space>
          <Button type="primary" htmlType="submit" disabled={disabled || changed}>
            Save
          </Button>
          <Button disabled={disabled || changed} onClick={() => void save(null)}>
            Reset
          </Button>
        </Space>
      </Form>
    </>
  );
}
