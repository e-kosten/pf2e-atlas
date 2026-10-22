import { createContext, createElement, useContext, type ComponentType } from "react";
import type { QueryRelationship } from "../../generated/atlas";
import type { PickerSelection } from "../ui/pickers/SearchPickerModal";

export type LinkPickerProps = {
  open: boolean;
  initial?: QueryRelationship;
  initialTitle?: string;
  onCancel: () => void;
  onConfirm: (
    relationship: QueryRelationship,
    selection: PickerSelection | null,
  ) => string | null;
};

// The app composes the record picker; filter controls depend only on this port.
export const LinkPickerContext = createContext<ComponentType<LinkPickerProps> | null>(
  null,
);

/** Stable app-composed host; the concrete picker remains outside filter ownership. */
export function LinkPicker(props: LinkPickerProps) {
  const Component = useContext(LinkPickerContext);
  if (!Component) throw new Error("Link condition picker provider is required");
  return createElement(Component, props);
}
