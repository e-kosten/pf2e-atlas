import { Input } from "antd";
import type { InputProps } from "antd";
import { useRef, useState } from "react";

export function EditableCommitField({
  ariaLabel,
  className,
  inputMode,
  onCommit,
  placeholder,
  size,
  stopPropagation = false,
  value,
}: {
  ariaLabel: string;
  className?: string;
  inputMode?: InputProps["inputMode"];
  onCommit: (value: string) => void;
  placeholder?: string;
  size?: InputProps["size"];
  stopPropagation?: boolean;
  value: string;
}) {
  const [draftState, setDraftState] = useState({
    committedValue: value,
    draft: value,
  });
  const draft = draftState.committedValue === value ? draftState.draft : value;
  const setDraft = (nextDraft: string) =>
    setDraftState({ committedValue: value, draft: nextDraft });
  const editFinished = useRef(false);

  const finishEdit = (commit: boolean) => {
    if (editFinished.current) {
      return;
    }
    editFinished.current = true;
    if (commit && draft !== value) {
      onCommit(draft);
    }
    if (!commit) {
      setDraft(value);
    }
  };

  return (
    <Input
      aria-label={ariaLabel}
      className={className}
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
