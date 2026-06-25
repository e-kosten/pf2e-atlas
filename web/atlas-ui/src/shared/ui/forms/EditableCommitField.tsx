import { Input } from "antd";
import type { InputProps } from "antd";
import { useState } from "react";

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

  const commit = () => {
    if (draft !== value) {
      onCommit(draft);
    }
  };

  return (
    <Input
      aria-label={ariaLabel}
      className={className}
      inputMode={inputMode}
      onBlur={commit}
      onChange={(event) => setDraft(event.currentTarget.value)}
      onClick={stopPropagation ? (event) => event.stopPropagation() : undefined}
      onKeyDown={(event) => {
        if (stopPropagation) {
          event.stopPropagation();
        }
        if (event.key === "Enter") {
          event.preventDefault();
          commit();
          event.currentTarget.blur();
        }
        if (event.key === "Escape") {
          event.preventDefault();
          setDraftState({ committedValue: value, draft: value });
          event.currentTarget.blur();
        }
      }}
      placeholder={placeholder}
      size={size}
      value={draft}
    />
  );
}
