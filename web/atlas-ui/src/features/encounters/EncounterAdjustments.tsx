import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Grid, Input, InputNumber, Popover, Select, Space } from "antd";
import { Copy, Pencil, X } from "lucide-react";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { previewEncounterAdjustment } from "../../api/atlasApi";
import type {
  AdjustmentModifierType,
  AdjustmentTarget,
  ConditionDurationUnit,
  EncounterAdjustmentAction,
  EncounterAdjustmentRequest,
  EncounterAdjustmentView,
  EncounterParticipantView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { EncounterDurationFields } from "./EncounterDurationFields";
import { EncounterModal } from "./EncounterModal";
import { EncounterRowAction } from "./EncounterRowAction";
import { isEncounterConflict } from "./useEncounterMutations";

type Request = Omit<EncounterAdjustmentRequest, "request_id">;
export type AdjustmentHandler = (
  participantKey: string,
  request: Request,
) => Promise<{ active: boolean; refreshed: boolean }>;

type Draft = {
  mode: "add" | "edit" | "repeat" | "end";
  entry?: EncounterAdjustmentView;
  id: string;
  modal: boolean;
};
type Values = {
  stat?: string;
  name: string;
  amount?: number;
  modifierType: AdjustmentModifierType;
  lifetime: "manual" | "timed";
  durationAmount: number;
  durationUnit: ConditionDurationUnit;
  durationEvent: "start" | "end";
  durationParticipant: string;
  remaining?: number;
};

const targetKey = (value: AdjustmentTarget) => JSON.stringify(value);
const signed = (value: number) => (value > 0 ? `+${value}` : `−${-value}`);

function targetGroup(target: AdjustmentTarget): string {
  switch (target.kind) {
    case "armor_class":
    case "fortitude":
    case "reflex":
    case "will":
      return "Defenses";
    case "perception":
    case "skill":
      return "Checks";
    case "speed":
      return "Movement";
    case "strike_attacks":
    case "spell_attacks":
    case "spell_dcs":
      return "Attacks and spells";
  }
}

function remainingText(entry: EncounterAdjustmentView) {
  if (!entry.timing) return "Until removed";
  const { duration, remaining, anchor_available: available } = entry.timing;
  const units =
    duration.unit === "rounds"
      ? `${remaining} ${remaining === 1 ? "round" : "rounds"}`
      : [
          Math.floor(remaining / 3600) && `${Math.floor(remaining / 3600)}h`,
          Math.floor((remaining % 3600) / 60) &&
            `${Math.floor((remaining % 3600) / 60)}m`,
          remaining % 60 && `${remaining % 60}s`,
        ]
          .filter(Boolean)
          .join(" ");
  return available ? `${units} left` : `${units} left (timing participant unavailable)`;
}

export function EncounterAdjustments({
  participant,
  participants,
  encounterRef,
  encounterRevision,
  referenceGeneration,
  disabled,
  pending,
  error,
  recovery,
  onChange,
}: {
  participant: EncounterParticipantView;
  participants: EncounterParticipantView[];
  encounterRef: string;
  encounterRevision: number;
  referenceGeneration: string | null;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onChange?: AdjustmentHandler;
}) {
  const heading = useId();
  const screens = Grid.useBreakpoint();
  const add = useRef<HTMLButtonElement>(null);
  const opener = useRef<HTMLElement | null>(null);
  const panel = useRef<HTMLDivElement>(null);
  const [draft, setDraft] = useState<Draft>();
  const [closingModal, setClosingModal] = useState(false);
  const data = participant.gm_adjustments;
  if (!data || !onChange || (!data.targets.length && !data.entries.length)) return null;

  const open = (
    element: HTMLElement,
    mode: Draft["mode"],
    entry?: EncounterAdjustmentView,
  ) => {
    opener.current = element;
    setDraft({
      mode,
      entry,
      id: crypto.randomUUID(),
      modal: !screens.lg || window.innerHeight < 640,
    });
  };
  const returnFocus = () => {
    const element = opener.current?.isConnected ? opener.current : add.current;
    restoreOverlayFocus(element, panel.current);
  };
  const close = () => {
    if (draft?.modal) {
      setClosingModal(true);
    } else {
      returnFocus();
      setDraft(undefined);
    }
  };
  const editor = draft && (
    <AdjustmentEditor
      key={draft.id}
      draft={draft}
      encounterRef={encounterRef}
      encounterRevision={encounterRevision}
      referenceGeneration={referenceGeneration}
      participant={participant}
      roster={participants}
      error={error}
      pending={pending}
      disabled={disabled}
      recovery={recovery}
      panel={panel}
      onClose={close}
      onSave={async (request) => {
        const result = await onChange(participant.participant_key, request);
        if (result.active) close();
      }}
    />
  );
  const title =
    draft?.mode === "end"
      ? "End adjustment"
      : draft?.mode === "edit"
        ? "Edit adjustment"
        : "Add adjustment";
  return (
    <section className="encounter-conditions" aria-labelledby={heading}>
      <div className="encounter-conditions__header">
        <h3 id={heading} className="encounter-inspector-heading">
          Adjustments
        </h3>
        <Popover
          open={!!draft && !draft.modal}
          trigger="click"
          placement="left"
          onOpenChange={(next) => {
            if (!next && !pending && !draft?.modal) close();
          }}
          title={title}
          styles={{
            root: { width: "min(25rem, calc(100vw - 2rem))" },
            content: {
              maxHeight: "min(38rem, calc(100dvh - 8rem))",
              overflowY: "auto",
            },
          }}
          content={!draft?.modal ? editor : null}
          destroyOnHidden
        >
          <Button
            ref={add}
            size="small"
            disabled={disabled || !data.targets.length}
            onClick={(event) => {
              if (!draft) open(event.currentTarget, "add");
            }}
            aria-haspopup="dialog"
          >
            Add adjustment
          </Button>
        </Popover>
      </div>
      {!data.targets.length && (
        <p className="encounter-empty-note">No adjustable stats available.</p>
      )}
      <div className="encounter-condition-list">
        {data.entries.map((entry) => (
          <div className="encounter-condition-row" key={entry.adjustment_key}>
            <span className="encounter-condition-row__name">{entry.name}</span>
            <span className="encounter-condition-actions">
              <EncounterRowAction
                tooltip="Edit"
                icon={<Pencil size={14} aria-hidden="true" />}
                aria-label={`Edit ${entry.name}`}
                disabled={disabled}
                onClick={(event) => open(event.currentTarget, "edit", entry)}
              />
              <EncounterRowAction
                tooltip="Apply again"
                icon={<Copy size={14} aria-hidden="true" />}
                aria-label={`Apply ${entry.name} again`}
                disabled={disabled || !entry.target_available}
                onClick={(event) => open(event.currentTarget, "repeat", entry)}
              />
              <EncounterRowAction
                tooltip="End now"
                icon={<X size={14} aria-hidden="true" />}
                aria-label={`End ${entry.name} now`}
                disabled={disabled}
                onClick={(event) => open(event.currentTarget, "end", entry)}
              />
            </span>
            <span className="encounter-condition-meta">
              {entry.label} {signed(entry.amount)} · {remainingText(entry)}
              {!entry.target_available && " · Stat unavailable"}
            </span>
          </div>
        ))}
      </div>
      {draft?.modal && (
        <EncounterModal
          open={!closingModal}
          title={title}
          footer={null}
          onCancel={close}
          afterClose={() => {
            returnFocus();
            setDraft(undefined);
            setClosingModal(false);
          }}
          closable={!pending}
          keyboard={!pending}
          mask={{ closable: !pending }}
        >
          {editor}
        </EncounterModal>
      )}
    </section>
  );
}

function initialValues(
  entry: EncounterAdjustmentView | undefined,
  participantKey: string,
): Values {
  return {
    stat: entry ? targetKey(entry.target) : undefined,
    name: entry?.name ?? "",
    amount: entry?.amount,
    modifierType: entry?.modifier_type ?? "stacks",
    lifetime: entry?.timing ? "timed" : "manual",
    durationAmount: entry?.timing?.duration.amount ?? 1,
    durationUnit: entry?.timing?.duration.unit ?? "rounds",
    durationEvent: entry?.timing?.duration.timing?.event ?? "start",
    durationParticipant:
      entry?.timing?.duration.timing?.participant_key ?? participantKey,
    remaining: undefined,
  };
}

function adjustmentSettings(
  entry: EncounterAdjustmentView | undefined,
  includeRemaining: boolean,
) {
  return !entry
    ? null
    : {
        name: entry.name,
        target: targetKey(entry.target),
        amount: entry.amount,
        modifierType: entry.modifier_type,
        duration: entry.timing?.duration,
        remaining: includeRemaining ? entry.timing?.remaining : undefined,
      };
}

function AdjustmentEditor({
  draft,
  encounterRef,
  encounterRevision,
  referenceGeneration,
  participant,
  roster,
  disabled,
  pending,
  error,
  recovery,
  panel,
  onClose,
  onSave,
}: {
  draft: Draft;
  encounterRef: string;
  encounterRevision: number;
  referenceGeneration: string | null;
  participant: EncounterParticipantView;
  roster: EncounterParticipantView[];
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  panel: React.RefObject<HTMLDivElement | null>;
  onClose: () => void;
  onSave: (request: Request) => Promise<void>;
}) {
  const [form] = Form.useForm<Values>();
  const queryClient = useQueryClient();
  const participantKey = participant.participant_key;
  const [baseline, setBaseline] = useState(draft.entry);
  const [values, setValues] = useState(() =>
    initialValues(draft.entry, participant.participant_key),
  );
  const ending = draft.mode === "end";
  const editing = draft.mode === "edit";
  const entryKey = draft.entry?.adjustment_key;
  const current = participant.gm_adjustments?.entries.find(
    (entry) => entry.adjustment_key === entryKey,
  );
  const removed = (editing || ending) && !current;
  const conflict =
    (editing || ending) &&
    current &&
    JSON.stringify(adjustmentSettings(baseline, values.remaining != null)) !==
      JSON.stringify(adjustmentSettings(current, values.remaining != null));
  const options = participant.gm_adjustments?.targets ?? [];
  const unavailable =
    editing &&
    current &&
    !options.some((choice) => targetKey(choice.target) === targetKey(current.target));
  const choices = unavailable ? [...options, current] : options;
  const target = choices.find(
    (choice) => targetKey(choice.target) === values.stat,
  )?.target;
  const groupedChoices = new Map<string, { label: string; value: string }[]>();
  for (const choice of choices) {
    const group = targetGroup(choice.target);
    const option = { label: choice.label, value: targetKey(choice.target) };
    const bucket = groupedChoices.get(group);
    if (bucket) bucket.push(option);
    else groupedChoices.set(group, [option]);
  }
  const timed = values.lifetime === "timed";
  const validAnchor =
    !timed ||
    roster.some((p) => p.participant_key === values.durationParticipant) ||
    (editing &&
      current?.timing?.duration.timing?.participant_key === values.durationParticipant);

  const amount = values.amount;
  let action: EncounterAdjustmentAction | undefined;
  if (ending && entryKey) {
    action = { kind: "end", adjustment_key: entryKey };
  } else if (
    target &&
    Number.isSafeInteger(amount) &&
    amount !== undefined &&
    amount !== 0 &&
    Math.abs(amount) <= 10000 &&
    (!timed ||
      (Number.isSafeInteger(values.durationAmount) &&
        values.durationAmount > 0 &&
        validAnchor &&
        (values.remaining == null ||
          (Number.isSafeInteger(values.remaining) && values.remaining > 0))))
  ) {
    const value = {
      target,
      name: values.name,
      amount,
      modifier_type: values.modifierType,
      timing: timed
        ? {
            duration: {
              amount: values.durationAmount,
              unit: values.durationUnit,
              timing: {
                participant_key: values.durationParticipant,
                event: values.durationEvent,
              },
            },
            ...(values.remaining == null ? {} : { remaining: values.remaining }),
          }
        : null,
    };
    action =
      editing && entryKey
        ? { kind: "edit", adjustment_key: entryKey, value }
        : { kind: "add", value };
  }
  const request = action && {
    expected_encounter_revision: encounterRevision,
    expected_participant_revision: participant.revision,
    action,
  };
  const body = request ? JSON.stringify(request) : "";
  const [readyBody, setReadyBody] = useState("");
  useEffect(() => {
    const timer = window.setTimeout(() => setReadyBody(body), 150);
    return () => window.clearTimeout(timer);
  }, [body]);
  const preview = useQuery({
    enabled:
      !!request &&
      !!readyBody &&
      readyBody === body &&
      !removed &&
      !conflict &&
      !disabled,
    queryKey: [
      "encounter-adjustment-preview",
      encounterRef,
      participant.participant_key,
      referenceGeneration,
      readyBody,
    ],
    queryFn: async ({ signal }) => {
      try {
        return await previewEncounterAdjustment(
          encounterRef,
          participantKey,
          { ...(JSON.parse(readyBody) as Request), request_id: draft.id },
          signal,
        );
      } catch (error) {
        if (isEncounterConflict(error)) {
          await queryClient.invalidateQueries({
            queryKey: ["encounter", encounterRef],
            exact: true,
          });
        }
        throw error;
      }
    },
    retry: false,
    staleTime: Infinity,
  });
  const matchingPreview =
    readyBody === body && !preview.isFetching && !conflict && !removed
      ? preview.data
      : undefined;
  const staleEntry = removed ? "This adjustment has ended or was removed." : undefined;
  const applying = useRef(false);
  const submit = async () => {
    if (
      !request ||
      !matchingPreview ||
      disabled ||
      removed ||
      conflict ||
      applying.current
    )
      return;
    applying.current = true;
    try {
      await onSave({
        ...request,
        expected_reference_revision: matchingPreview.reference_revision,
      });
    } catch {
      // The encounter mutation keeps the form open and reports the save state.
    } finally {
      applying.current = false;
    }
  };
  const focusSelector = ending
    ? "button[data-adjustment-cancel]"
    : draft.mode === "add"
      ? 'input[role="combobox"]'
      : 'input[role="spinbutton"]';
  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      panel.current
        ?.querySelector<HTMLElement>(focusSelector)
        ?.focus({ preventScroll: true });
    });
    return () => cancelAnimationFrame(frame);
  }, [focusSelector, panel]);

  return (
    <div
      ref={panel}
      role={draft.modal ? undefined : "dialog"}
      aria-label={
        draft.modal
          ? undefined
          : ending
            ? "End adjustment"
            : editing
              ? "Edit adjustment"
              : "Add adjustment"
      }
      onKeyDown={(event) => {
        if (
          event.key === "Escape" &&
          !draft.modal &&
          !event.defaultPrevented &&
          !pending
        ) {
          event.stopPropagation();
          onClose();
        }
      }}
    >
      {recovery}
      {staleEntry && <p role="status">{staleEntry}</p>}
      {conflict && current && (
        <ErrorNotice
          compact
          type="warning"
          title="This adjustment changed"
          error={undefined}
          description={`Saved: ${current.name}, ${current.label} ${signed(current.amount)}, ${current.modifier_type}, ${remainingText(current)}.`}
          action={
            <Space wrap>
              {editing && (
                <Button
                  onClick={() => {
                    const next = initialValues(current, participant.participant_key);
                    form.setFieldsValue(next);
                    setValues(next);
                    setBaseline(current);
                  }}
                >
                  Load saved values
                </Button>
              )}
              <Button onClick={() => setBaseline(current)}>
                {editing ? "Keep my edits" : "Review current adjustment"}
              </Button>
            </Space>
          }
        />
      )}
      {error && (
        <ErrorNotice
          error={error}
          compact
          title="Adjustment update could not be confirmed"
        />
      )}
      {ending ? (
        <p>End {current?.name ?? draft.entry?.name}?</p>
      ) : (
        <Form<Values>
          form={form}
          initialValues={values}
          onValuesChange={() => setValues(form.getFieldsValue(true))}
          onFinish={() => void submit()}
          layout="vertical"
          disabled={disabled || removed}
        >
          <Form.Item label="Statistic" name="stat">
            <Select
              showSearch={{ optionFilterProp: "label" }}
              placeholder="Choose a statistic"
              options={Array.from(groupedChoices, ([label, options]) => ({
                label,
                options,
              }))}
            />
          </Form.Item>
          <Form.Item
            label="Change"
            name="amount"
            help={amount === 0 ? "Enter a nonzero change." : undefined}
            validateStatus={amount === 0 ? "error" : undefined}
          >
            <InputNumber min={-10000} max={10000} precision={0} placeholder="+ or −" />
          </Form.Item>
          <Form.Item label="Name" name="name">
            <Input placeholder="GM adjustment" maxLength={120} />
          </Form.Item>
          <Form.Item label="Modifier type" name="modifierType">
            <Select
              options={[
                { label: "Stacks", value: "stacks" },
                { label: "Status", value: "status" },
                { label: "Circumstance", value: "circumstance" },
              ]}
            />
          </Form.Item>
          <Form.Item label="Lifetime" name="lifetime">
            <Select
              options={[
                { label: "Until removed", value: "manual" },
                { label: "Timed", value: "timed" },
              ]}
            />
          </Form.Item>
          {timed && (
            <>
              <EncounterDurationFields
                participants={roster}
                amountRules={[{ required: true }]}
                timingRules={[{ required: true }]}
                maximumAmount={Number.MAX_SAFE_INTEGER}
              />
              {editing && current?.timing && (
                <Form.Item
                  label={
                    values.durationUnit === "rounds"
                      ? "Remaining rounds"
                      : "Remaining seconds"
                  }
                  name="remaining"
                >
                  <InputNumber
                    placeholder={String(current.timing.remaining)}
                    min={1}
                    max={Number.MAX_SAFE_INTEGER}
                    precision={0}
                  />
                </Form.Item>
              )}
            </>
          )}
        </Form>
      )}
      {request && !removed && !conflict && !matchingPreview && !preview.isError && (
        <p role="status">Calculating preview…</p>
      )}
      {preview.isError && readyBody === body && (
        <ErrorNotice
          error={preview.error}
          compact
          title="Preview unavailable"
          action={<Button onClick={() => void preview.refetch()}>Retry</Button>}
        />
      )}
      {matchingPreview && (
        <div className="encounter-condition-list" aria-live="polite">
          {matchingPreview.rows.map((row, index) => (
            <div key={`${row.label}:${index}`}>
              {row.label} {row.before ?? "—"} → {row.after ?? "—"}
              {row.suppressed
                ? " · Suppressed"
                : row.after == null
                  ? " · Stat unavailable"
                  : ""}
            </div>
          ))}
        </div>
      )}
      <Space>
        <Button data-adjustment-cancel disabled={pending} onClick={onClose}>
          Cancel
        </Button>
        <Button
          type="primary"
          danger={ending}
          disabled={disabled || removed || !!conflict || !matchingPreview}
          loading={pending}
          onClick={() => void submit()}
        >
          {ending ? "End now" : editing ? "Save changes" : "Apply"}
        </Button>
      </Space>
    </div>
  );
}
