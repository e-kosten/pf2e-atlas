import type { KeyboardEvent } from "react";
import { useEffect, useRef } from "react";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export function useActiveResultScroll<T extends HTMLElement>(
  activeResultKey: string | null,
) {
  const containerRef = useRef<T | null>(null);
  useEffect(() => {
    containerRef.current
      ?.querySelector('[data-active-result="true"]')
      ?.scrollIntoView({ block: "nearest" });
  }, [activeResultKey]);
  return containerRef;
}

export function handleResultKeyboard(
  event: KeyboardEvent<HTMLElement>,
  workspace: AtlasWorkspaceState,
) {
  switch (event.key) {
    case "ArrowDown":
    case "j":
      event.preventDefault();
      workspace.moveResultSelection("next");
      break;
    case "ArrowUp":
    case "k":
      event.preventDefault();
      workspace.moveResultSelection("previous");
      break;
    case "Enter":
      event.preventDefault();
      workspace.openActiveResult();
      break;
    case "Escape":
      if (workspace.selectedRecordKey) {
        event.preventDefault();
        workspace.selectRecord(null);
      }
      break;
    default:
      break;
  }
}
