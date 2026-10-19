import { useState } from "react";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";

export function InitiativeValueField({
  name,
  value,
  disabled,
  onSave,
  compact = false,
}: {
  name: string;
  value: number | bigint | undefined;
  disabled: boolean;
  onSave: (value: number | undefined) => Promise<unknown>;
  compact?: boolean;
}) {
  const [error, setError] = useState<string>();
  return (
    <span className="initiative-total-editor">
      <EditableCommitField
        ariaLabel={`${name} initiative total`}
        className={compact ? "initiative-roster-input" : "initiative-roll-total"}
        inputMode="numeric"
        size={compact ? "small" : undefined}
        stopPropagation
        disabled={disabled}
        value={value?.toString() ?? ""}
        onCommit={async (text) => {
          setError(undefined);
          const trimmed = text.trim();
          const next = trimmed === "" ? undefined : Number(trimmed);
          if (
            next !== undefined &&
            (!/^-?\d+$/.test(trimmed) || !Number.isSafeInteger(next))
          ) {
            setError("Enter a whole number.");
            throw new Error("Invalid initiative");
          }
          try {
            await onSave(next);
          } catch (cause) {
            setError("Couldn’t save initiative. Try again.");
            throw cause;
          }
        }}
      />
      {error && <span role="alert">{error}</span>}
    </span>
  );
}
