import type { RefObject } from "react";
import { EditableValue } from "../../shared/ui/forms/EditableValue";

export function EncounterHpValue({
  value,
  ...props
}: {
  label: string;
  value?: bigint;
  disabled: boolean;
  addLabel?: string;
  buttonRef?: RefObject<HTMLButtonElement | null>;
  onCommit: (input: string) => Promise<boolean>;
  onRestoreFocus?: () => void;
}) {
  return (
    <EditableValue
      {...props}
      value={value?.toString() ?? ""}
      displayValue={value?.toString() ?? "—"}
      buttonClassName={props.addLabel ? undefined : "encounter-hp-value"}
      editorClassName="encounter-hp-value__editor"
      inputClassName="encounter-hp-value__input"
    />
  );
}
