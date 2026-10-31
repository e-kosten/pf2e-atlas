import { Button } from "antd";
import { useLayoutEffect, useRef, useState, type RefObject } from "react";
import { EditableCommitField } from "./EditableCommitField";

export function EditableValue({
  label,
  value,
  disabled,
  addLabel,
  buttonRef,
  onCommit,
  onCancel,
  onRestoreFocus,
  buttonClassName,
  editorClassName,
  inputClassName,
  displayValue = value,
}: {
  label: string;
  value: string;
  displayValue?: string;
  disabled: boolean;
  addLabel?: string;
  buttonRef?: RefObject<HTMLButtonElement | null>;
  onCommit: (input: string) => Promise<boolean>;
  onCancel?: () => void;
  onRestoreFocus?: () => void;
  buttonClassName?: string;
  editorClassName?: string;
  inputClassName?: string;
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
    if (!editing && !disabled && returnFocus.current) {
      returnFocus.current = false;
      if (document.activeElement === document.body) {
        trigger.current?.focus({ preventScroll: true });
      }
    }
  }, [disabled, editing, trigger]);
  return editing ? (
    <span ref={editor} className={editorClassName}>
      <EditableCommitField
        autoSize
        onCancel={() => {
          finishEditing();
          onCancel?.();
        }}
        focusOnMount
        blurOnEnter={false}
        ariaLabel={label}
        className={inputClassName}
        size="small"
        value={value}
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
            }
            if (refreshed && hadFocus) onRestoreFocus?.();
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
      className={buttonClassName}
      aria-label={addLabel ?? `Edit ${label}`}
      disabled={disabled}
      onClick={() => setEditing(true)}
    >
      {addLabel ?? displayValue}
    </Button>
  );
}
