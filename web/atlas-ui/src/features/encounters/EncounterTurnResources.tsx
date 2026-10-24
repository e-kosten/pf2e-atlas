import { Button, InputNumber, Popover, Space, Tooltip } from "antd";
import { Undo2 } from "lucide-react";
import { useRef, useState } from "react";
import type {
  TurnResourceAction,
  TurnResourceKind,
  TurnResourcesView,
} from "../../generated/atlas";
import { RuntimeCount } from "../../shared/records/ActorReference";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";

export type TurnResourceHandler = (action: TurnResourceAction) => Promise<void>;
export function EncounterTurnResources({
  value,
  disabled,
  isCurrent,
  onChange,
}: {
  value?: TurnResourcesView;
  disabled: boolean;
  isCurrent: boolean;
  onChange?: TurnResourceHandler;
}) {
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const [error, setError] = useState<string>();
  if (!value) return null;
  const act = async (action: TurnResourceAction) => {
    if (!onChange || inFlight.current || disabled)
      throw new Error("Wait for the encounter to refresh before editing again.");
    inFlight.current = true;
    setError(undefined);
    setBusy(true);
    try {
      await onChange(action);
    } catch (error) {
      setError(
        error instanceof Error ? error.message : "Could not update turn resources.",
      );
      throw error;
    } finally {
      inFlight.current = false;
      setBusy(false);
    }
  };
  const unavailable = (disabled && !busy) || !onChange;
  const blockedActions =
    !value.can_spend_actions.available && value.regular_total !== null && isCurrent;
  const blockedReactions =
    !value.can_spend_reactions.available && value.reactions_remaining !== null;
  const sameRestriction =
    blockedActions &&
    blockedReactions &&
    value.can_spend_actions.reason === value.can_spend_reactions.reason;
  const stock = value.reactions_remaining;
  return (
    <section className="encounter-turn-resources" aria-label="Turn" aria-busy={busy}>
      <div className="encounter-turn-resources__controls">
        {value.regular_total !== null && value.regular_remaining !== null ? (
          <>
            <ResourceCounter
              label="Actions"
              kind="action"
              total={Number(value.regular_total)}
              remaining={Number(value.regular_remaining)}
              disabled={unavailable}
              pending={busy}
              canEdit={value.can_edit_actions}
              canSpend={value.can_spend_actions.available}
              reason={value.can_spend_actions.reason}
              onAction={act}
            />
            {value.extra_total !== null && Number(value.extra_total) > 0 && (
              <ResourceCounter
                label="Extra action"
                kind="extra_action"
                total={Number(value.extra_total)}
                remaining={Number(value.extra_remaining)}
                disabled={unavailable}
                pending={busy}
                canEdit={value.can_edit_actions}
                canSpend={value.can_spend_actions.available}
                reason={value.can_spend_actions.reason}
                onAction={act}
              />
            )}
          </>
        ) : (
          <div className="encounter-turn-resources__group">
            <RuntimeCount
              count={value.forecast}
              label="At turn start"
              unit=" actions"
            />
            {isCurrent && (
              <Button
                disabled={unavailable}
                onClick={() => {
                  void act({ kind: "regain" }).catch(() => {});
                }}
              >
                Regain actions
              </Button>
            )}
          </div>
        )}
        {stock !== null && (
          <div className="encounter-turn-resources__group">
            <strong>Reaction</strong>
            <Tooltip
              title={
                !value.can_spend_reactions.available
                  ? value.can_spend_reactions.reason
                  : undefined
              }
              trigger={["hover", "focus"]}
            >
              <Button
                disabled={
                  unavailable ||
                  Number(value.reactions_total) === 0 ||
                  (Number(stock) > 0 && !value.can_spend_reactions.available)
                }
                aria-label={Number(stock) > 0 ? "Spend reaction" : "Restore reaction"}
                color="default"
                variant="filled"
                onClick={() => {
                  void act({
                    kind: Number(stock) > 0 ? "spend" : "restore",
                    resource: "reaction",
                  }).catch(() => {});
                }}
              >
                {Number(value.reactions_total) === 0
                  ? "None this turn"
                  : Number(stock) === 0
                    ? "Spent"
                    : value.can_spend_reactions.available
                      ? "Available"
                      : "Unspent"}
              </Button>
            </Tooltip>
          </div>
        )}
      </div>
      {Number(value.extra_total) > 0 &&
        value.extra_uses.map(
          (use, index) =>
            use.kind === "restricted" && (
              <p className="muted" key={index}>
                Extra action:{" "}
                {[
                  use.actions.map((action) => action.replace(/_/g, " ")).join(", "),
                  use.guidance,
                ]
                  .filter(Boolean)
                  .join(" · ")}
              </p>
            ),
        )}
      {value.pending_reason && <p role="status">{value.pending_reason}</p>}
      {sameRestriction ? (
        <p>
          Cannot act or react
          {value.can_spend_actions.reason
            ? ` · ${value.can_spend_actions.reason.replace(/\.$/, "")}`
            : ""}
        </p>
      ) : (
        <>
          {blockedActions && (
            <p>
              Cannot act
              {value.can_spend_actions.reason
                ? ` · ${value.can_spend_actions.reason.replace(/\.$/, "")}`
                : ""}
            </p>
          )}
          {blockedReactions && (
            <p>
              Cannot react
              {value.can_spend_reactions.reason
                ? ` · ${value.can_spend_reactions.reason.replace(/\.$/, "")}`
                : ""}
            </p>
          )}
        </>
      )}
      {value.start_results.map((result, index) => (
        <p className="encounter-turn-resources__result" key={index}>
          {result}
        </p>
      ))}
      {value.regular_total !== null &&
        value.extra_total !== null &&
        value.forecast.adjusted_value !== null &&
        Number(value.forecast.adjusted_value) !==
          Number(value.regular_total) + Number(value.extra_total) && (
          <RuntimeCount count={value.forecast} label="Next turn" unit=" actions" />
        )}
      {error && <p role="alert">{error}</p>}
    </section>
  );
}
function ResourceCounter({
  label,
  kind,
  total,
  remaining,
  disabled,
  pending,
  canEdit,
  canSpend,
  reason,
  onAction,
}: {
  label: string;
  kind: TurnResourceKind;
  total: number;
  remaining: number;
  disabled: boolean;
  pending: boolean;
  canEdit: boolean;
  canSpend: boolean;
  reason?: string;
  onAction: TurnResourceHandler;
}) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<number | null>(remaining);
  const button = useRef<HTMLButtonElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  return (
    <div className="encounter-turn-resources__group">
      <strong>{label}</strong>
      <Space size={4} wrap>
        {Array.from({ length: total }, (_, index) => (
          <Tooltip
            key={index}
            title={
              !canSpend
                ? reason
                : index < remaining
                  ? "Spend action"
                  : "Use Restore action to correct the count"
            }
            trigger={["hover", "focus"]}
          >
            <Button
              aria-label={`Spend ${kind === "action" ? "action" : "extra action"} ${index + 1}`}
              disabled={disabled || !canSpend || index >= remaining}
              aria-disabled={pending || !canSpend || index >= remaining}
              color={index < remaining ? "primary" : "default"}
              variant={index < remaining ? "filled" : "outlined"}
              icon={
                <span
                  className="action-cost__glyph"
                  data-atlas-action-glyph="one_action"
                  aria-hidden="true"
                />
              }
              onClick={(event) => {
                const trigger = event.currentTarget;
                const focused = trigger === document.activeElement;
                void onAction({ kind: "spend", resource: kind })
                  .then(() => {
                    if (focused && index >= remaining - 1)
                      restoreOverlayFocus(button.current, trigger);
                  })
                  .catch(() => {});
              }}
            />
          </Tooltip>
        ))}
        <Popover
          trigger="click"
          open={open}
          onOpenChange={(next) => {
            if (!next) restoreOverlayFocus(button.current, popup.current);
            setOpen(next);
            if (next) setDraft(remaining);
          }}
          title={`Remaining ${label.toLowerCase()}`}
          content={
            <div ref={popup}>
              <Space>
                <InputNumber
                  aria-label={`Remaining ${label.toLowerCase()}`}
                  min={0}
                  max={total}
                  precision={0}
                  value={draft}
                  readOnly={pending || disabled}
                  onChange={setDraft}
                />
                <Button
                  type="primary"
                  disabled={disabled || !canEdit || draft === null}
                  aria-disabled={pending}
                  onClick={() => {
                    if (!pending && draft !== null)
                      void onAction({
                        kind: "correct",
                        resource: kind,
                        remaining: BigInt(draft),
                      })
                        .then(() => {
                          restoreOverlayFocus(button.current, popup.current);
                          setOpen(false);
                        })
                        .catch(() => {});
                  }}
                >
                  Apply
                </Button>
              </Space>
            </div>
          }
        >
          <Button
            ref={button}
            color="default"
            variant="filled"
            disabled={disabled || !canEdit}
            aria-label={`Edit remaining ${label.toLowerCase()}: ${remaining} of ${total}`}
          >
            {remaining} / {total}
          </Button>
        </Popover>
        <Tooltip
          title={`Restore ${kind === "action" ? "action" : "extra action"}`}
          trigger={["hover", "focus"]}
        >
          <Button
            aria-label={`Restore ${kind === "action" ? "action" : "extra action"}`}
            disabled={disabled || !canEdit || remaining >= total}
            icon={<Undo2 size={14} />}
            onClick={(event) => {
              const trigger = event.currentTarget;
              const focused = trigger === document.activeElement;
              void onAction({ kind: "restore", resource: kind })
                .then(() => {
                  if (focused && remaining + 1 === total)
                    restoreOverlayFocus(button.current, trigger);
                })
                .catch(() => {});
            }}
          />
        </Tooltip>
      </Space>
    </div>
  );
}
