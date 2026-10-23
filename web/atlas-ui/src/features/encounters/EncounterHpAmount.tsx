import { Button, Input, Tooltip } from "antd";
import { useId, useRef, useState } from "react";
import type { EncounterHpAction } from "../../generated/atlas";

export function EncounterHpAmount({
  disabled,
  pending = false,
  onAction,
  healUnavailableReason,
  actions = ["damage", "heal"],
}: {
  disabled: boolean;
  pending?: boolean;
  onAction: (action: EncounterHpAction) => Promise<boolean>;
  healUnavailableReason?: string;
  actions?: readonly ("damage" | "heal")[];
}) {
  const id = useId();
  const inFlight = useRef(false);
  const [amount, setAmount] = useState("");
  const apply = async (kind: "damage" | "heal") => {
    if (
      disabled ||
      inFlight.current ||
      !amount.trim() ||
      (kind === "heal" && healUnavailableReason)
    )
      return;
    inFlight.current = true;
    try {
      if (await onAction({ kind, amount })) setAmount("");
    } catch {
      /* Keep raw input; the feature mutation owner displays the error. */
    } finally {
      inFlight.current = false;
    }
  };
  return (
    <div className="encounter-hp-control encounter-hp-control--wide">
      <label htmlFor={id} className="encounter-hp-control__label">
        HP change
      </label>
      <div className="encounter-hp-control__row">
        <Input
          id={id}
          aria-label="HP change"
          disabled={disabled && !pending}
          readOnly={disabled}
          value={amount}
          onChange={(event) => setAmount(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              if (!event.repeat && actions.includes("heal")) void apply("heal");
            }
          }}
        />
        <div className="encounter-hp-actions">
          {actions.map((kind) => (
            <Tooltip
              key={kind}
              title={kind === "heal" ? healUnavailableReason : undefined}
              trigger={["hover", "focus"]}
            >
              <span>
                <Button
                  disabled={
                    disabled ||
                    !amount.trim() ||
                    (kind === "heal" && !!healUnavailableReason)
                  }
                  onClick={() => void apply(kind)}
                >
                  {kind === "damage" ? "Damage" : "Heal"}
                </Button>
              </span>
            </Tooltip>
          ))}
        </div>
      </div>
    </div>
  );
}
