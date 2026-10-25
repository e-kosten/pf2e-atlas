import { useQuery } from "@tanstack/react-query";
import { Alert, Button, InputNumber, Popover, Table, Typography } from "antd";
import { ChevronDown } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import type {
  EncounterDifficultyReferenceView,
  EncounterDifficultyView,
  SetEncounterDifficultyRequest,
} from "../../generated/atlas";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";

interface Props {
  encounterKey: string;
  revision: bigint;
  value: EncounterDifficultyView;
  disabled: boolean;
  onPreview: (
    request: SetEncounterDifficultyRequest,
  ) => Promise<EncounterDifficultyView>;
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
  const button = useRef<HTMLButtonElement>(null);
  const content = useRef<HTMLDivElement>(null);
  function changeOpen(next: boolean) {
    if (!next) restoreOverlayFocus(button.current, content.current);
    setOpen(next);
  }
  return (
    <div className="encounter-difficulty">
      <Popover
        trigger="click"
        placement="bottomLeft"
        align={{
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
            <DifficultyReference {...props} onClose={() => changeOpen(false)} />
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
  onClose,
}: Props & { onClose: () => void }) {
  const levelId = useId();
  const sizeId = useId();
  const [draft, setDraft] = useState(value.reference);
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
              value={draft.party_level}
              placeholder={
                display.party_level === null ? "Choose" : `Auto: ${display.party_level}`
              }
              disabled={saving}
              onChange={(next) => setDraft({ ...draft, party_level: next })}
            />
          </label>
          <label htmlFor={sizeId}>
            Budget PCs
            <InputNumber
              className="encounter-difficulty__input"
              id={sizeId}
              aria-label="PC count"
              min={1}
              max={100}
              precision={0}
              value={draft.party_size}
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
        {display.budgets.length > 0 && (
          <div className="encounter-difficulty__budgets" aria-label="XP budgets">
            {display.budgets.map((budget) => (
              <div key={budget.label}>
                <span>{budget.label}</span>
                <strong>{budget.xp}</strong>
              </div>
            ))}
          </div>
        )}
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
      <p className="encounter-difficulty__basis">
        Includes defeated and disabled opponents. Allies and neutral participants are
        excluded.
      </p>
      <div className="encounter-difficulty__sources">
        <Typography.Link
          href="https://2e.aonprd.com/Rules.aspx?ID=2716"
          target="_blank"
          rel="noreferrer"
        >
          XP budgets
        </Typography.Link>
        <Typography.Link
          href="https://2e.aonprd.com/Rules.aspx?ID=2653"
          target="_blank"
          rel="noreferrer"
        >
          Mixed-level parties
        </Typography.Link>
      </div>
    </>
  );
}
