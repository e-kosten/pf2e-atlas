import { Input } from "antd";
import type { InputProps } from "antd";
import { useRef, useState } from "react";

export function EditableCommitField({
  ariaLabel,
  className,
  disabled = false,
  inputMode,
  onCommit,
  placeholder,
  size,
  stopPropagation = false,
  value,
}: {
  ariaLabel: string;
  className?: string;
  disabled?: InputProps["disabled"];
  inputMode?: InputProps["inputMode"];
  onCommit: (value: string) => void | Promise<unknown>;
  placeholder?: string;
  size?: InputProps["size"];
  stopPropagation?: boolean;
  value: string;
}) {
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
    if (disabled || editFinished.current) {
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
  };

  return (
    <Input
      aria-label={ariaLabel}
      className={className}
      disabled={disabled}
      inputMode={inputMode}
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
          event.currentTarget.blur();
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
    />
  );
}
