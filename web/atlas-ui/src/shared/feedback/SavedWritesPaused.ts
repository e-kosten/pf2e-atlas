import { createContext, useContext } from "react";

export const SavedWritesPaused = createContext(false);

export function useSavedWritesPaused() {
  return useContext(SavedWritesPaused);
}
