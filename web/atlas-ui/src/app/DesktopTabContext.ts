import type { ResolvedColorScheme } from "../shared/theme/atlasTheme";
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
