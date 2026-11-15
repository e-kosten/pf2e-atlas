import { useDesktopTab } from "../../app/DesktopTabContext";
import { useEffect } from "react";

/** Only the primary route owns the title; reference previews keep their host's title. */
export function usePageTitle(pageName: string, tabName = pageName) {
  const tab = useDesktopTab();
  const title = tab?.title;
  const active = tab?.active ?? true;
  useEffect(() => title?.(tabName), [tabName, title]);
  useEffect(() => {
    if (!active) return;
    document.title = `${pageName} · Atlas`;
    return () => {
      document.title = "Atlas";
    };
  }, [active, pageName]);
}
