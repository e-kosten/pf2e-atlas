import type { ResolvedColorScheme } from "../shared/theme/atlasTheme";
import { useDesktopAttachment } from "./desktopBridge";
import { useRouter } from "@tanstack/react-router";
import { createContext, useContext, useLayoutEffect } from "react";

export type TabCloseGuard = () => "pending" | "dirty" | undefined;

export type DesktopTabRuntime = {
  id: string;
  active: boolean;
  activation: number;
  panel: () => HTMLElement;
  title: (name: string) => void;
  theme: (scheme: ResolvedColorScheme) => void;
  guard: (guard: TabCloseGuard) => () => void;
};

export const DesktopTabContext = createContext<DesktopTabRuntime | null>(null);

export function useDesktopTab() {
  return useContext(DesktopTabContext);
}

export function useTabCloseGuard(guard: TabCloseGuard) {
  const register = useDesktopTab()?.guard;
  useLayoutEffect(() => register?.(guard), [guard, register]);
}

export function useDesktopHistoryGuard(guard: TabCloseGuard) {
  const platform = useDesktopAttachment()?.historyNavigation;
  const active = useDesktopTab()?.active ?? true;
  const history = useRouter({ warn: false })?.history;
  useLayoutEffect(() => {
    if (!platform || !active || !history) return;
    return history.block({
      enableBeforeUnload: false,
      blockerFn: ({ action }) =>
        action !== "PUSH" && action !== "REPLACE" && Boolean(guard()),
    });
  }, [active, guard, history, platform]);
}

export function canFocusTabElement(
  element: HTMLElement | null | undefined,
): element is HTMLElement {
  return (
    !!element &&
    element.isConnected &&
    !element.closest("[hidden], [inert]") &&
    !element.matches(":disabled") &&
    (element.checkVisibility?.({ visibilityProperty: true }) ?? true)
  );
}
