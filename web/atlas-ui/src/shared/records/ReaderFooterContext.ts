import { createContext } from "react";

export const ReaderFooterContext = createContext<{
  active: boolean;
  rememberReaderPosition: () => void;
} | null>(null);
