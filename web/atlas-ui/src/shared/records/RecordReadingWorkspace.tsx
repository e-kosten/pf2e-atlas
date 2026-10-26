import { Activity, useRef, useState, useLayoutEffect, type ReactNode } from "react";
import { Tooltip } from "antd";
import { PaneIconButton } from "../ui/actions/PaneAction";
import { BookOpen, Compass } from "lucide-react";
import { useRecordExploreLocation } from "./recordExploreLocation";

/** Read and explore share one identity and the host's existing scroll surface. */
export function RecordReadingWorkspace({
  children,
  explore,
  identity,
  status,
}: {
  children: ReactNode;
  explore: ReactNode;
  identity?: (action: ReactNode) => ReactNode;
  status?: ReactNode;
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
  function changeMode(next: "read" | "explore") {
    if (next === "explore") {
      let host = root.current?.parentElement ?? null;
      while (host && !/(auto|scroll)/.test(getComputedStyle(host).overflowY))
        host = host.parentElement;
      bookmark.current = (host ?? document.scrollingElement)?.scrollTop ?? 0;
      setExplored(true);
    }
    setMode(next);
  }
  return (
    <div className="record-workspace" ref={root}>
      {identity?.(
        <Tooltip
          title={mode === "read" ? "Explore related records" : "Read document"}
          trigger={["hover", "focus"]}
        >
          <PaneIconButton
            label={mode === "read" ? "Explore related records" : "Read document"}
            onClick={() => changeMode(mode === "read" ? "explore" : "read")}
            icon={mode === "read" ? <Compass size={16} /> : <BookOpen size={16} />}
          />
        </Tooltip>,
      )}
      {status}
      <Activity mode={mode === "read" ? "visible" : "hidden"}>
        <div className="record-workspace__reading">{children}</div>
      </Activity>
      <Activity mode={mode === "explore" ? "visible" : "hidden"}>
        {explored || mode === "explore" ? explore : null}
      </Activity>
    </div>
  );
}
