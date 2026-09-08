import { createContext, useContext } from "react";
import type React from "react";
import type { PreviewPopoverTrigger } from "../ui/overlays/PreviewPopover";

export type RecordPreviewRenderer = (
  recordKey: string,
  trigger: PreviewPopoverTrigger,
) => React.ReactNode;

export const RecordPreviewContext = createContext<RecordPreviewRenderer | null>(null);

export function useRecordPreviewRenderer() {
  return useContext(RecordPreviewContext);
}
