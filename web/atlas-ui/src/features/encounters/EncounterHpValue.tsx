import { Button } from "antd";
import { useLayoutEffect, useRef, useState, type RefObject } from "react";
import { EditableCommitField } from "../../shared/ui/forms/EditableCommitField";

export function EncounterHpValue({
  label,
  value,
  disabled,
  addLabel,
  buttonRef,
  onCommit,
  onRestoreFocus,
}: {
  label: string;
  value?: bigint;
  disabled: boolean;
  addLabel?: string;
  buttonRef?: RefObject<HTMLButtonElement | null>;
  onCommit: (input: string) => Promise<boolean>;
  onRestoreFocus?: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [saving, setSaving] = useState(false);
  const valueButton = useRef<HTMLButtonElement>(null);
  const trigger = buttonRef ?? valueButton;
  const editor = useRef<HTMLSpanElement>(null);
  const returnFocus = useRef(false);
  const finishEditing = () => {
    returnFocus.current = editor.current?.contains(document.activeElement) ?? false;
    setEditing(false);
  };
  useLayoutEffect(() => {
    if (!editing && returnFocus.current) {
      returnFocus.current = false;
      trigger.current?.focus({ preventScroll: true });
    }
  }, [editing, trigger]);
  return editing ? (
    <span ref={editor} className="encounter-hp-value__editor">
      <EditableCommitField
        autoSize
        onCancel={() => {
          finishEditing();
        }}
        focusOnMount
        blurOnEnter={false}
        ariaLabel={label}
        className="encounter-hp-value__input"
        size="small"
        value={value?.toString() ?? ""}
        disabled={disabled && !saving}
        readOnly={saving}
        onCommit={async (input) => {
          const editingElement = editor.current;
          const hadFocus = editingElement?.contains(document.activeElement);
          setSaving(true);
          try {
            const refreshed = await onCommit(input);
            if (editingElement?.isConnected) {
              finishEditing();
            } else if (
              refreshed &&
              hadFocus &&
              document.activeElement === document.body
            ) {
              onRestoreFocus?.();
            }
          } finally {
            setSaving(false);
          }
        }}
      />
    </span>
  ) : (
    <Button
      ref={trigger}
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
