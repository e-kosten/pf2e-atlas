import { createContext } from "react";
import type { RecordNavigationView } from "../../generated/atlas";

export type NavigateReaderMatch = (
  navigation: RecordNavigationView,
  anchor: HTMLElement,
) => void;

/** The reader owns scroll/history/focus; feature-supplied context uses its navigation. */
export const ReaderMatchNavigation = createContext<NavigateReaderMatch | null>(null);
