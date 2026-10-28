import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useQuery } from "@tanstack/react-query";
import { Button, InputNumber, Popover, Table, Tooltip, Typography } from "antd";
import { ChevronDown, Info } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import type {
  EncounterDifficultyReferenceView,
  EncounterDifficultyView,
  PreviewEncounterDifficultyRequest,
  RecordNavigationView,
} from "../../generated/atlas";
import { EncounterDifficultyMeter } from "./EncounterDifficultyMeter";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import type { RecordReferenceHandler } from "../../shared/records/PreparedContent";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";

export interface EncounterDifficultyProps {
  encounterKey: string;
  revision: bigint;
  artifactGeneration?: string | null;
  value: EncounterDifficultyView;
  disabled: boolean;
  onPreview: (
    request: PreviewEncounterDifficultyRequest,
  ) => Promise<EncounterDifficultyView>;
  onReference: RecordReferenceHandler;
  onSave: (reference: EncounterDifficultyReferenceView) => Promise<void>;
}
export function difficultySummary(value: EncounterDifficultyView) {
  if (value.party_level === null || value.party_size === null)
    return "Difficulty unavailable";
  if (value.total_xp === null) return "Difficulty incomplete";
  return `${value.rating} · ${value.total_xp} XP`;
}
export function EncounterDifficulty(props: EncounterDifficultyProps) {
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
  const buttonLabel =
    props.value.party_level === null || props.value.party_size === null
      ? "Difficulty"
      : difficultySummary(props.value);
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
          type="text"
          aria-expanded={open}
          aria-label={`Encounter difficulty: ${buttonLabel}`}
          icon={<ChevronDown size={14} />}
          iconPlacement="end"
        >
          {buttonLabel}
        </Button>
      </Popover>
    </div>
  );
}
function DifficultyReference({
  encounterKey,
  revision,
  artifactGeneration,
  value,
  disabled,
  onSave,
  onPreview,
  onReadRule,
  initialReference,
  onClose,
}: EncounterDifficultyProps & {
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
      artifactGeneration,
    ],
    queryFn: () =>
      onPreview({
        expected_revision: revision,
        reference: debounced,
        addition: null,
      }),
    enabled: valid && !unchanged,
    retry: false,
  });
  const currentDraft =
    draft.party_level === debounced.party_level &&
    draft.party_size === debounced.party_size;
  const shown = unchanged ? value : currentDraft ? preview.data : undefined;
  const identity = JSON.stringify([
    encounterKey,
    revision.toString(),
    artifactGeneration,
  ]);
  const [retained, setRetained] = useState({ identity, value });
  if (shown && (retained.identity !== identity || shown !== retained.value))
    setRetained({ identity, value: shown });
  const display = shown ?? (retained.identity === identity ? retained.value : value);
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
        error instanceof Error ? error : new Error("Party details could not save."),
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
            Use PCs
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
        {display.reference_note && (
          <p className="encounter-difficulty__basis">{display.reference_note}</p>
        )}
        {(display.party_level === null || display.party_size === null) && (
          <p>Enter party level and size, or add PCs.</p>
        )}
        {saveError && (
          <ErrorNotice error={saveError} title="Difficulty could not be saved" />
        )}
        {!valid && (
          <p role="alert">
            Use level 1–20 and 1–100 PCs. Leave blank to use encounter PCs.
          </p>
        )}
        {!unchanged && previewError && (
          <ErrorNotice
            error={preview.error}
            compact
            title="Difficulty preview could not update"
            action={<Button onClick={() => void preview.refetch()}>Retry</Button>}
          />
        )}
      </form>
      <div className="encounter-difficulty__calculation" aria-busy={pending}>
        <div className="encounter-difficulty__total" role="status">
          <strong>{difficultySummary(display)}</strong>
          <span>
            {pending
              ? "Updating preview…"
              : !valid
                ? "Previous calculation · check party details"
                : !unchanged && previewError
                  ? "Previous calculation · update failed"
                  : !unchanged
                    ? "Preview · not saved"
                    : display.reference.party_level !== null ||
                        display.reference.party_size !== null
                      ? "Custom party details"
                      : "From encounter PCs"}
          </span>
        </div>
        {display.total_xp === null && display.party_level !== null && (
          <p>{display.known_xp} known XP · missing contributions below</p>
        )}
        <EncounterDifficultyMeter value={display} />
        <dl className="encounter-difficulty__budgets" aria-label="XP budgets">
          {display.budgets.map((budget) => (
            <div key={budget.label}>
              <dt>{budget.label}</dt>
              <dd>{budget.xp} XP</dd>
            </div>
          ))}
        </dl>
        <div className="encounter-difficulty__award">
          <span>
            Suggested XP per PC
            {display.award_note && (
              <Tooltip title={display.award_note} trigger={["hover", "focus"]}>
                <Button
                  type="text"
                  size="small"
                  aria-label="XP award calculation"
                  icon={<Info size={14} />}
                />
              </Tooltip>
            )}
          </span>
          <strong>{display.xp_per_pc ?? "—"}</strong>
        </div>
        {display.opposition.length > 0 ? (
          <Table
            size="small"
            pagination={false}
            rowKey={(row) => row.participant_key ?? "prospective-addition"}
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
      : value == null || String(value).trim() === ""
        ? ""
        : override === null
          ? `Auto: ${value}`
          : String(value);
}
function parseReferenceNumber(value: string | undefined): number {
  const text = value?.replace(/^Auto:\s*/, "").trim();
  return text ? Number(text) : Number.NaN;
}
