import { useEffect } from "react";

/** Only the primary route owns the title; reference previews keep their host's title. */
export function usePageTitle(pageName: string) {
  useEffect(() => {
    document.title = `${pageName} · Atlas`;
    return () => {
      document.title = "Atlas";
    };
  }, [pageName]);
}
