import { createContext } from "react";

export type ExpandedField = { id: string; title: string; trigger: HTMLButtonElement };

export const ModalEditorExpansion = createContext<{
  field: ExpandedField | null;
  expand: (field: ExpandedField) => void;
  release: (id: string) => void;
} | null>(null);
