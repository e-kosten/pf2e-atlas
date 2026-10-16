import { Button, Input } from "antd";
import { useId, useState } from "react";
import type { EncounterHpAction } from "../../generated/atlas";

export function EncounterHpAmount({
  disabled,
  onAction,
  actions = ["damage", "heal"],
}: {
  disabled: boolean;
  onAction: (action: EncounterHpAction) => Promise<boolean>;
  actions?: readonly ("damage" | "heal")[];
}) {
  const id = useId();
  const [amount, setAmount] = useState("");
  const apply = async (kind: "damage" | "heal") => {
    if (disabled || !amount.trim()) return;
    try {
      if (await onAction({ kind, amount })) setAmount("");
    } catch {
      /* Keep raw input; the feature mutation owner displays the error. */
    }
  };
  return (
    <div className="encounter-hp-control encounter-hp-control--wide">
      <label htmlFor={id} className="encounter-hp-control__label">
        Amount (zero or greater)
      </label>
      <div className="encounter-hp-control__row">
        <Input
          id={id}
          aria-label="HP change"
          disabled={disabled}
          value={amount}
          onChange={(event) => setAmount(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") event.preventDefault();
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
