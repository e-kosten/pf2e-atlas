import { Button } from "antd";
import { useLayoutEffect, useRef, useState } from "react";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";

export function EncounterHpValue({
  label,
  value,
  disabled,
  addLabel,
  onCommit,
}: {
  label: string;
  value?: bigint;
  disabled: boolean;
  addLabel?: string;
  onCommit: (input: string) => Promise<boolean>;
}) {
  const [editing, setEditing] = useState(false);
  const [error, setError] = useState<string>();
  const valueButton = useRef<HTMLButtonElement>(null);
  const editor = useRef<HTMLSpanElement>(null);
  const returnFocus = useRef(false);
  const finishEditing = () => {
    returnFocus.current = editor.current?.contains(document.activeElement) ?? false;
    setEditing(false);
  };
  useLayoutEffect(() => {
    if (!editing && returnFocus.current) {
      returnFocus.current = false;
      valueButton.current?.focus({ preventScroll: true });
    }
  }, [editing]);
  return editing ? (
    <span ref={editor}>
      <EditableCommitField
        onCancel={() => {
          finishEditing();
          setError(undefined);
        }}
        focusOnMount
        blurOnEnter={false}
        ariaLabel={label}
        className="encounter-hp-value__input"
        size="small"
        inputMode="numeric"
        value={value?.toString() ?? ""}
        disabled={disabled}
        onCommit={async (input) => {
          if (
            input.trim() &&
            (!/^\d+$/.test(input.trim()) || !Number.isSafeInteger(Number(input)))
          ) {
            setError("Enter a nonnegative whole number.");
            throw new Error("Invalid HP");
          }
          setError(undefined);
          await onCommit(input);
          finishEditing();
        }}
      />
      {error && <span role="alert">{error}</span>}
    </span>
  ) : (
    <Button
      ref={valueButton}
      size="small"
      color="default"
      variant={addLabel ? "outlined" : "filled"}
      className={addLabel ? undefined : "encounter-hp-value"}
      aria-label={addLabel ?? `Edit ${label}`}
      disabled={disabled}
      onClick={() => setEditing(true)}
    >
      {addLabel ?? value?.toString() ?? "—"}
    </Button>
  );
}
