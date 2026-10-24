import { Button, InputNumber, Popover, Space, Tooltip } from "antd";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import type {
  TurnResourceAction,
  TurnResourceKind,
  TurnResourcesView,
} from "../../generated/atlas";
import { RuntimeCount } from "../../shared/records/ActorReference";
import { RuntimeCountCalculation } from "../../shared/records/RuntimeCountCalculation";
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
          <div className="encounter-turn-resources__actions">
            <ResourceCounter
              label="Actions"
              kind="action"
              turnKey={value.turn_key}
              forecast={value.forecast}
              extraTotal={
                value.extra_total === null ? undefined : Number(value.extra_total)
              }
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
                turnKey={value.turn_key}
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
          </div>
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
            {Number(value.reactions_total) === 0 ? (
              <span>None this turn</span>
            ) : (
              <Tooltip
                title={
                  Number(stock) > 0
                    ? value.can_spend_reactions.available
                      ? "Spend reaction"
                      : value.can_spend_reactions.reason
                    : "Restore reaction"
                }
                trigger={["hover", "focus"]}
              >
                <Button
                  disabled={
                    unavailable ||
                    (Number(stock) > 0 && !value.can_spend_reactions.available)
                  }
                  aria-disabled={
                    busy ||
                    unavailable ||
                    (Number(stock) > 0 && !value.can_spend_reactions.available)
                  }
                  aria-label={Number(stock) > 0 ? "Spend reaction" : "Restore reaction"}
                  aria-pressed={Number(stock) > 0}
                  color={Number(stock) > 0 ? "primary" : "default"}
                  variant={Number(stock) > 0 ? "filled" : "outlined"}
                  icon={
                    <span
                      className="action-cost__glyph"
                      data-atlas-action-glyph="reaction"
                      aria-hidden="true"
                    />
                  }
                  onClick={() => {
                    void act({
                      kind: Number(stock) > 0 ? "spend" : "restore",
                      resource: "reaction",
                    }).catch(() => {});
                  }}
                />
              </Tooltip>
            )}
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
  turnKey,
  forecast,
  extraTotal,
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
  turnKey: string | null;
  forecast?: TurnResourcesView["forecast"];
  extraTotal?: number;
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
  // Slot positions are presentation only; the canonical count always owns stock.
  const [slots, setSlots] = useState(() => ({
    turnKey,
    total,
    remaining,
    available: Array.from({ length: total }, (_, index) => index < remaining),
    target: undefined as { index: number; remaining: number } | undefined,
  }));
  if (
    slots.turnKey !== turnKey ||
    slots.total !== total ||
    slots.remaining !== remaining
  ) {
    const available =
      slots.turnKey !== turnKey || slots.total !== total
        ? Array.from({ length: total }, (_, index) => index < remaining)
        : [...slots.available];
    if (
      slots.turnKey === turnKey &&
      slots.total === total &&
      slots.target?.remaining === remaining
    )
      available[slots.target.index] = remaining > slots.remaining;
    let difference = remaining - available.filter(Boolean).length;
    for (let index = 0; index < total && difference > 0; index++) {
      if (!available[index]) {
        available[index] = true;
        difference--;
      }
    }
    for (let index = total - 1; index >= 0 && difference < 0; index--) {
      if (available[index]) {
        available[index] = false;
        difference++;
      }
    }
    setSlots({ turnKey, total, remaining, available, target: undefined });
  }
  return (
    <div className="encounter-turn-resources__group">
      {forecast ? (
        <Popover
          trigger="click"
          open={open}
          onOpenChange={(next) => {
            if (!next) restoreOverlayFocus(button.current, popup.current);
            setOpen(next);
            if (next) setDraft(remaining);
          }}
          title={label}
          content={
            <div ref={popup} className="encounter-turn-resource-details">
              <p>
                This turn: {total} {total === 1 ? "action" : "actions"}
                {extraTotal ? ` + ${extraTotal} extra` : null}
              </p>
              <div className="encounter-turn-resource-details__correction">
                <span>{extraTotal ? "Ordinary remaining" : "Remaining"}</span>
                <Space>
                  <InputNumber
                    aria-label={
                      extraTotal
                        ? "Remaining ordinary actions"
                        : `Remaining ${label.toLowerCase()}`
                    }
                    min={0}
                    max={total}
                    precision={0}
                    value={draft}
                    readOnly={pending || disabled || !canEdit}
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
              <div>
                <strong>Next turn</strong>
                <RuntimeCountCalculation count={forecast} />
              </div>
            </div>
          }
        >
          <Button
            ref={button}
            type="text"
            styles={{ root: { paddingInline: 0, fontSize: "inherit" } }}
            icon={<ChevronDown size={14} />}
            iconPlacement="end"
            aria-expanded={open}
            aria-label={`${label}: details and remaining actions`}
          >
            <strong>{label}</strong>
          </Button>
        </Popover>
      ) : null}
      <Space size={4} wrap>
        {slots.available.map((available, index) => (
          <Tooltip
            key={index}
            title={
              available
                ? !canSpend
                  ? reason
                  : `Spend ${kind === "action" ? "action" : "extra action"}`
                : `Restore ${kind === "action" ? "action" : "extra action"}`
            }
            trigger={["hover", "focus"]}
          >
            <Button
              aria-label={`${available ? "Spend" : "Restore"} ${kind === "action" ? "action" : "extra action"} ${index + 1}`}
              aria-pressed={available}
              disabled={disabled || !canEdit || (available && !canSpend)}
              aria-disabled={
                pending || disabled || !canEdit || (available && !canSpend)
              }
              color={available ? "primary" : "default"}
              variant={available ? "filled" : "outlined"}
              icon={
                <span
                  className="action-cost__glyph"
                  data-atlas-action-glyph="one_action"
                  aria-hidden="true"
                />
              }
              onClick={() => {
                if (pending) return;
                setSlots((current) => ({
                  ...current,
                  target: { index, remaining: remaining + (available ? -1 : 1) },
                }));
                void onAction({
                  kind: available ? "spend" : "restore",
                  resource: kind,
                }).catch(() =>
                  setSlots((current) => ({ ...current, target: undefined })),
                );
              }}
            />
          </Tooltip>
        ))}
      </Space>
    </div>
  );
}
