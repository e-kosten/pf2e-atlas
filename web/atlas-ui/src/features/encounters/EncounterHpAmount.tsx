import { Button, Input } from "antd";
import { useId, useRef, useState } from "react";
import type { EncounterHpAction } from "../../generated/atlas";

export function EncounterHpAmount({
  disabled,
  pending = false,
  onAction,
  actions = ["damage", "heal"],
}: {
  disabled: boolean;
  pending?: boolean;
  onAction: (action: EncounterHpAction) => Promise<boolean>;
  actions?: readonly ("damage" | "heal")[];
}) {
  const id = useId();
  const inFlight = useRef(false);
  const [amount, setAmount] = useState("");
  const apply = async (kind: "damage" | "heal") => {
    if (disabled || inFlight.current || !amount.trim()) return;
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
            <Button
              key={kind}
              disabled={disabled || !amount.trim()}
              onClick={() => void apply(kind)}
            >
              {kind === "damage" ? "Damage" : "Heal"}
            </Button>
          ))}
        </div>
      </div>
    </div>
  );
}
