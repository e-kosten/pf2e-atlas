import { createContext, useContext } from "react";
import type { RecordReferenceHandler } from "./PreparedContent";

export type ReferencePreview = {
  open: RecordReferenceHandler;
  openTrait: (traitKey: string, anchor?: HTMLElement) => void;
};
export const ReferencePreviewContext = createContext<ReferencePreview | null>(null);
export function useReferencePreview() {
  return useContext(ReferencePreviewContext);
}
