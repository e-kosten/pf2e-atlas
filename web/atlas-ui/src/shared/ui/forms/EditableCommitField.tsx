import { Input } from "antd";
import type { InputProps, InputRef } from "antd";
import { useEffect, useRef, useState } from "react";

export function EditableCommitField({
  autoSize = false,
  focusOnMount,
  ariaLabel,
  blurOnEnter = true,
  className,
  disabled = false,
  readOnly = false,
  inputMode,
  onCancel,
  onCommit,
  placeholder,
  size,
  stopPropagation = false,
  suffix,
  value,
  variant,
}: {
  autoSize?: boolean;
  focusOnMount?: boolean;
  ariaLabel: string;
  blurOnEnter?: boolean;
  className?: string;
  disabled?: InputProps["disabled"];
  readOnly?: InputProps["readOnly"];
  inputMode?: InputProps["inputMode"];
  onCancel?: () => void;
  onCommit: (value: string) => void | Promise<unknown>;
  placeholder?: string;
  size?: InputProps["size"];
  stopPropagation?: boolean;
  suffix?: InputProps["suffix"];
  value: string;
  variant?: InputProps["variant"];
}) {
  const inputRef = useRef<InputRef>(null);
  useEffect(() => {
    if (focusOnMount) inputRef.current?.focus({ cursor: "all" });
  }, [focusOnMount]);
  const [draftState, setDraftState] = useState({
    committedValue: value,
    draft: value,
    failed: false,
  });
  const draft =
    draftState.failed || draftState.committedValue === value ? draftState.draft : value;
  const setDraft = (nextDraft: string) =>
    setDraftState({ committedValue: value, draft: nextDraft, failed: false });
  const editFinished = useRef(false);

  const finishEdit = (commit: boolean) => {
    if (disabled || readOnly || editFinished.current) {
      return;
    }
    editFinished.current = true;
    if (commit && draft !== value) {
      const result = onCommit(draft);
      if (result) {
        void result
          .then(() => {
            setDraftState((current) =>
              current.draft === draft ? { ...current, failed: false } : current,
            );
          })
          .catch(() => {
            setDraftState({ committedValue: value, draft, failed: true });
            editFinished.current = false;
          });
      }
    }
    if (!commit) {
      setDraft(value);
    }
    if (!commit || draft === value) onCancel?.();
  };

  return (
    <Input
      ref={inputRef}
      aria-label={ariaLabel}
      className={className}
      htmlSize={autoSize ? Math.max(1, draft.length) : undefined}
      disabled={disabled}
      readOnly={readOnly}
      inputMode={inputMode}
      suffix={suffix}
      onBlur={() => finishEdit(true)}
      onChange={(event) => {
        editFinished.current = false;
        setDraft(event.currentTarget.value);
      }}
      onClick={stopPropagation ? (event) => event.stopPropagation() : undefined}
      onFocus={() => {
        editFinished.current = false;
      }}
      onKeyDown={(event) => {
        if (stopPropagation) {
          event.stopPropagation();
        }
        if (event.key === "Enter") {
          event.preventDefault();
          finishEdit(true);
          if (blurOnEnter) event.currentTarget.blur();
        }
        if (event.key === "Escape") {
          event.preventDefault();
          finishEdit(false);
          event.currentTarget.blur();
        }
      }}
      placeholder={placeholder}
      size={size}
      value={draft}
      variant={variant}
    />
  );
}
