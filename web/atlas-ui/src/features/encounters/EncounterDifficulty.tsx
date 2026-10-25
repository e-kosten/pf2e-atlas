import { useQuery } from "@tanstack/react-query";
import { Alert, Button, InputNumber, Popover, Table, Typography } from "antd";
import { ChevronDown } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import type {
  EncounterDifficultyReferenceView,
  EncounterDifficultyView,
  SetEncounterDifficultyRequest,
  RecordNavigationView,
} from "../../generated/atlas";
import { EncounterDifficultyMeter } from "./EncounterDifficultyMeter";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import type { RecordReferenceHandler } from "../../shared/records/PreparedContent";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";

interface Props {
  encounterKey: string;
  revision: bigint;
  value: EncounterDifficultyView;
  disabled: boolean;
  onPreview: (
    request: SetEncounterDifficultyRequest,
  ) => Promise<EncounterDifficultyView>;
  onReference: RecordReferenceHandler;
  onSave: (reference: EncounterDifficultyReferenceView) => Promise<void>;
}
function summary(value: EncounterDifficultyView) {
  if (value.party_level === null || value.party_size === null)
    return "Set party reference";
  if (value.total_xp === null) return "Difficulty incomplete";
  return `${value.rating} · ${value.total_xp} XP`;
}
export function EncounterDifficulty(props: Props) {
  const [open, setOpen] = useState(false);
  const [ruleDraft, setRuleDraft] = useState<EncounterDifficultyReferenceView>();
  const button = useRef<HTMLButtonElement>(null);
  const content = useRef<HTMLDivElement>(null);
  function changeOpen(next: boolean) {
    if (!next) {
      setRuleDraft(undefined);
      restoreOverlayFocus(button.current, content.current);
    }
    setOpen(next);
  }
  return (
    <div className="encounter-difficulty">
      <Popover
        trigger="click"
        placement="bottomLeft"
        align={{
          offset: [0, 0],
          overflow: { adjustX: true, adjustY: true, shiftX: true, shiftY: true },
          htmlRegion: "visible",
        }}
        open={open}
        onOpenChange={changeOpen}
        afterOpenChange={(next) => {
          if (next && open && document.activeElement === button.current)
            content.current?.focus({ preventScroll: true });
        }}
        destroyOnHidden
        content={
          <div
            ref={content}
            className="encounter-difficulty__popover"
            role="dialog"
            aria-label="Encounter difficulty"
            // eslint-disable-next-line jsx-a11y/no-noninteractive-tabindex -- A bounded reading region needs keyboard scrolling in WebKit.
            tabIndex={0}
          >
            <DifficultyReference
              {...props}
              onClose={() => changeOpen(false)}
              initialReference={ruleDraft ?? props.value.reference}
              onReadRule={(selection, anchor, draft) => {
                changeOpen(false);
                setRuleDraft(draft);
                props.onReference(
                  selection.record_key,
                  button.current ?? anchor,
                  selection,
                );
              }}
            />
          </div>
        }
      >
        <Button
          ref={button}
          size="small"
          aria-expanded={open}
          aria-label={`Encounter difficulty: ${summary(props.value)}`}
          icon={<ChevronDown size={14} />}
          iconPlacement="end"
        >
          {summary(props.value)}
        </Button>
      </Popover>
      {props.value.party_level !== null && props.value.party_size !== null && (
        <span className="encounter-difficulty__reference">
          Level {props.value.party_level} ·{" "}
          {props.value.reference_note ? "Budget for " : ""}
          {props.value.party_size} PCs
        </span>
      )}
    </div>
  );
}
function DifficultyReference({
  encounterKey,
  revision,
  value,
  disabled,
  onSave,
  onPreview,
  onReadRule,
  initialReference,
  onClose,
}: Props & {
  onClose: () => void;
  initialReference: EncounterDifficultyReferenceView;
  onReadRule: (
    selection: RecordNavigationView,
    anchor: HTMLElement,
    draft: EncounterDifficultyReferenceView,
  ) => void;
}) {
  const levelId = useId();
  const sizeId = useId();
  const [draft, setDraft] = useState(initialReference);
  const [debounced, setDebounced] = useState(draft);
  const [saveError, setSaveError] = useState<Error | null>(null);
  const [saving, setSaving] = useState(false);
  const active = useRef(true);
  useEffect(() => {
    active.current = true;
    return () => {
      active.current = false;
    };
  }, []);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(draft), 300);
    return () => clearTimeout(timer);
  }, [draft]);
  const valid =
    (draft.party_level === null ||
      (Number.isInteger(draft.party_level) &&
        draft.party_level >= 1 &&
        draft.party_level <= 20)) &&
    (draft.party_size === null ||
      (Number.isInteger(draft.party_size) &&
        draft.party_size >= 1 &&
        draft.party_size <= 100));
  const unchanged =
    draft.party_level === value.reference.party_level &&
    draft.party_size === value.reference.party_size;
  const preview = useQuery({
    queryKey: [
      "encounter-difficulty-preview",
      encounterKey,
      revision.toString(),
      debounced,
    ],
    queryFn: () =>
      onPreview({
        expected_revision: revision,
        reference: debounced,
      }),
    enabled: valid && !unchanged,
    retry: false,
  });
  const currentDraft =
    draft.party_level === debounced.party_level &&
    draft.party_size === debounced.party_size;
  const shown = unchanged ? value : currentDraft ? preview.data : undefined;
  const [retained, setRetained] = useState(value);
  if (shown && shown !== retained) setRetained(shown);
  const display = shown ?? retained;
  const previewError = !unchanged && currentDraft ? preview.error : null;
  const pending = valid && !unchanged && !shown && !previewError;
  async function save() {
    if (!valid || !shown || disabled || saving) return;
    setSaveError(null);
    setSaving(true);
    try {
      await onSave(draft);
      if (active.current) onClose();
    } catch (error) {
      setSaveError(
        error instanceof Error ? error : new Error("Party reference could not save."),
      );
    } finally {
      setSaving(false);
    }
  }
  return (
    <>
      <h3>Encounter difficulty</h3>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void save();
        }}
      >
        <div className="encounter-difficulty__fields">
          <label htmlFor={levelId}>
            Party level
            <InputNumber
              className="encounter-difficulty__input"
              id={levelId}
              aria-label="Party level"
              min={1}
              max={20}
              precision={0}
              value={draft.party_level ?? display.party_level}
              formatter={automaticFormatter(draft.party_level)}
              parser={parseReferenceNumber}
              placeholder={
                display.party_level === null ? "Choose" : `Auto: ${display.party_level}`
              }
              disabled={saving}
              onChange={(next) => setDraft({ ...draft, party_level: next })}
            />
          </label>
          <label htmlFor={sizeId}>
            Party size
            <InputNumber
              className="encounter-difficulty__input"
              id={sizeId}
              aria-label="PC count"
              min={1}
              max={100}
              precision={0}
              value={draft.party_size ?? display.party_size}
              formatter={automaticFormatter(draft.party_size)}
              parser={parseReferenceNumber}
              placeholder={
                display.party_size === null ? "Choose" : `Auto: ${display.party_size}`
              }
              disabled={saving}
              onChange={(next) => setDraft({ ...draft, party_size: next })}
            />
          </label>
          <Button
            disabled={
              saving || (draft.party_level === null && draft.party_size === null)
            }
            onClick={() => setDraft({ party_level: null, party_size: null })}
          >
            Use party
          </Button>
          <Button
            type="primary"
            htmlType="submit"
            loading={saving}
            disabled={disabled || unchanged || !valid || !shown || !!previewError}
          >
            Save
          </Button>
        </div>
        <p className="encounter-difficulty__basis">
          {display.basis}
          {display.reference_note ? ` ${display.reference_note}` : ""}
        </p>
        {saveError && <Alert type="error" title={saveError.message} />}
        {!valid && (
          <p role="alert">
            Choose level 1–20 and 1–100 PCs, or leave blank to follow the party.
          </p>
        )}
        {!unchanged && previewError && (
          <Alert
            type="error"
            title="Difficulty preview could not update"
            action={<Button onClick={() => void preview.refetch()}>Retry</Button>}
          />
        )}
      </form>
      <div className="encounter-difficulty__calculation" aria-busy={pending}>
        <div className="encounter-difficulty__total" role="status">
          <strong>{summary(display)}</strong>
          <span>
            {pending
              ? "Updating preview…"
              : !valid
                ? "Previous calculation · invalid reference"
                : !unchanged && previewError
                  ? "Previous calculation · update failed"
                  : !unchanged
                    ? "Preview · not saved"
                    : display.reference.party_level !== null ||
                        display.reference.party_size !== null
                      ? "GM reference"
                      : "From party"}
          </span>
        </div>
        {display.total_xp === null && display.party_level !== null && (
          <p>{display.known_xp} known XP · missing contributions below</p>
        )}
        <EncounterDifficultyMeter value={display} />
        {display.opposition.length > 0 ? (
          <Table
            size="small"
            pagination={false}
            rowKey="participant_key"
            dataSource={display.opposition}
            columns={[
              {
                title: "Opposition",
                key: "name",
                render: (_, row) => (
                  <>
                    <span className="encounter-difficulty__opponent-name">
                      {row.name}
                    </span>
                    <span className="encounter-difficulty__opponent-kind">
                      {row.kind}
                      {row.level !== null ? ` · Level ${row.level}` : ""}
                      {row.unavailable_reason ? ` · ${row.unavailable_reason}` : ""}
                    </span>
                  </>
                ),
              },
              {
                title: "XP",
                dataIndex: "xp",
                key: "xp",
                align: "right",
                width: 52,
                render: (xp: number | null) => xp ?? "—",
              },
            ]}
          />
        ) : (
          <p>No enemy creatures or hazards.</p>
        )}
      </div>
      <div className="encounter-difficulty__sources">
        {display.rules.map((rule) => (
          <Typography.Link
            key={rule.title}
            href={destinationHref(
              recordDestination({
                recordKey: rule.navigation.record_key,
                selection: rule.navigation,
              }),
            )}
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              onReadRule(rule.navigation, event.currentTarget, draft);
            }}
          >
            {rule.title}
          </Typography.Link>
        ))}
      </div>
    </>
  );
}

function automaticFormatter(override: number | null) {
  return (value: number | undefined, info: { userTyping: boolean; input: string }) =>
    info.userTyping
      ? info.input
      : value === undefined
        ? ""
        : override === null
          ? `Auto: ${value}`
          : String(value);
}
function parseReferenceNumber(value: string | undefined): number {
  const text = value?.replace(/^Auto:\s*/, "").trim();
  return text ? Number(text) : Number.NaN;
}
