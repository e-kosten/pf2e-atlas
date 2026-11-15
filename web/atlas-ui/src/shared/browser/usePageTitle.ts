import { useDesktopTab } from "../../app/DesktopTabContext";
import { useEffect } from "react";

type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: { token: string; nativeTabTitles?: boolean };
  __TAURI__?: {
    core: { invoke: (command: string, args: object) => Promise<unknown> };
  };
};

/** Only the primary route owns the title; reference previews keep their host's title. */
export function usePageTitle(pageName: string, tabName = pageName) {
  const tab = useDesktopTab();
  const title = tab?.title;
  const active = tab?.active ?? true;
  useEffect(() => title?.(tabName), [tabName, title]);
  useEffect(() => {
    const host = window as DesktopWindow;
    const core = host.__TAURI__?.core;
    if (!core || tab) return;
    let publishedToken: string | undefined;
    function publish() {
      const attachment = host.__ATLAS_DESKTOP_SETUP__;
      if (!core || !attachment?.nativeTabTitles || attachment.token === publishedToken)
        return;
      publishedToken = attachment.token;
      void core
        .invoke("desktop_native_tab_title", {
          token: attachment.token,
          title: tabName,
        })
        .catch((error) => console.error("Could not name Atlas tab", error));
    }
    publish();
    window.addEventListener("atlas-desktop-setup", publish);
    return () => window.removeEventListener("atlas-desktop-setup", publish);
  }, [tab, tabName]);
  useEffect(() => {
    if (!active) return;
    document.title = `${pageName} · Atlas`;
    return () => {
      document.title = "Atlas";
    };
  }, [active, pageName]);
}
