import { createContext, createElement, useContext, type ComponentType } from "react";
import type { QueryRelationship, StaticSearchContext } from "../../generated/atlas";
import type { PickerSelection } from "../ui/pickers/SearchPickerModal";

export type LinkPickerProps = {
  open: boolean;
  parent: StaticSearchContext;
  initial?: QueryRelationship;
  initialExcluded?: boolean;
  onCancel: () => void;
  afterClose?: () => void;
  onConfirm: (
    relationship: QueryRelationship,
    selection: PickerSelection | null,
    excluded: boolean,
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
