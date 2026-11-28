import { Activity, useRef, useState, useLayoutEffect, type ReactNode } from "react";
import { PaneIconButton } from "../ui/actions/PaneAction";
import { BookOpen, Compass } from "lucide-react";
import type { RecordExploreTab } from "../../app/urlState";
import {
  OpenRecordProgression,
  useRecordExploreLocation,
} from "./recordExploreLocation";

/** Read and explore share one identity and the host's existing scroll surface. */
export function RecordReadingWorkspace({
  children,
  explore,
  identity,
  status,
  readingNotice,
}: {
  children: ReactNode;
  explore: ReactNode;
  identity?: (action: ReactNode) => ReactNode;
  status?: ReactNode;
  readingNotice?: ReactNode;
}) {
  const { mode, setMode } = useRecordExploreLocation();
  const [explored, setExplored] = useState(mode === "explore");
  const previousMode = useRef(mode);
  const root = useRef<HTMLDivElement>(null);
  const bookmark = useRef(0);
  useLayoutEffect(() => {
    if (previousMode.current === mode) return;
    previousMode.current = mode;
    let host = root.current?.parentElement ?? null;
    while (host && !/(auto|scroll)/.test(getComputedStyle(host).overflowY))
      host = host.parentElement;
    const scroller = host ?? document.scrollingElement;
    if (mode === "read" && scroller) scroller.scrollTop = bookmark.current;
  }, [mode]);
  function changeMode(next: "read" | "explore", tab?: RecordExploreTab) {
    if (next === "explore") {
      let host = root.current?.parentElement ?? null;
      while (host && !/(auto|scroll)/.test(getComputedStyle(host).overflowY))
        host = host.parentElement;
      bookmark.current = (host ?? document.scrollingElement)?.scrollTop ?? 0;
      setExplored(true);
    }
    setMode(next, tab);
  }
  return (
    <div className="record-workspace" ref={root}>
      {identity?.(
        <PaneIconButton
          outlined
          label={mode === "read" ? "Explore related records" : "Read document"}
          onClick={() => changeMode(mode === "read" ? "explore" : "read")}
          icon={mode === "read" ? <Compass size={16} /> : <BookOpen size={16} />}
        />,
      )}
      {status}
      {mode === "read" && readingNotice}
      <Activity mode={mode === "read" ? "visible" : "hidden"}>
        <OpenRecordProgression value={() => changeMode("explore", "progression")}>
          <div className="record-workspace__reading">{children}</div>
        </OpenRecordProgression>
      </Activity>
      <Activity mode={mode === "explore" ? "visible" : "hidden"}>
        {explored || mode === "explore" ? explore : null}
      </Activity>
    </div>
  );
}
