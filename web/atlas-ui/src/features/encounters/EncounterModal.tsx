import { Modal, type ModalProps } from "antd";
import {
  createContext,
  useContext,
  useLayoutEffect,
  useState,
  type ReactNode,
  type RefObject,
} from "react";

const WorkArea = createContext<RefObject<HTMLDivElement | null> | null>(null);
export function EncounterModalProvider({
  area,
  children,
}: {
  area: RefObject<HTMLDivElement | null>;
  children: ReactNode;
}) {
  return <WorkArea.Provider value={area}>{children}</WorkArea.Provider>;
}
type Bounds = { top: number; left: number; width: number; height: number };
export function EncounterModal({
  open,
  style,
  ...props
}: Omit<ModalProps, "styles" | "centered">) {
  const area = useContext(WorkArea);
  const [bounds, setBounds] = useState<Bounds>();
  useLayoutEffect(() => {
    if (!open) return;
    const measure = () => {
      const viewport = window.visualViewport;
      const top = viewport?.offsetTop ?? 0;
      const left = viewport?.offsetLeft ?? 0;
      const width = viewport?.width ?? window.innerWidth;
      const height = viewport?.height ?? window.innerHeight;
      const rect = area?.current
        ?.querySelector<HTMLElement>(".workspace-grid")
        ?.getBoundingClientRect();
      const visibleTop = Math.max(top, rect?.top ?? top);
      const visibleLeft = Math.max(left, rect?.left ?? left);
      // A short participant panel must not shrink the editor. Use the available
      // viewport below its heading, regardless of the background content length.
      const bottom = top + height;
      const right = Math.min(left + width, rect?.right ?? left + width);
      setBounds(
        bottom - visibleTop < Math.min(240, height) ||
          right - visibleLeft < Math.min(240, width)
          ? { top, left, width, height }
          : {
              top: visibleTop,
              left: visibleLeft,
              width: right - visibleLeft,
              height: bottom - visibleTop,
            },
      );
    };
    measure();
    window.addEventListener("resize", measure);
    window.visualViewport?.addEventListener("resize", measure);
    return () => {
      window.removeEventListener("resize", measure);
      window.visualViewport?.removeEventListener("resize", measure);
    };
  }, [open, area]);
  const maximumHeight = bounds
    ? Math.max(0, bounds.height - 32)
    : "calc(100dvh - 32px)";
  return (
    <Modal
      {...props}
      open={open}
      centered
      transitionName=""
      maskTransitionName=""
      style={{ ...style, top: 0, paddingBottom: 0, maxWidth: "calc(100% - 32px)" }}
      styles={{
        wrapper: bounds
          ? {
              // Keep Ant's full-viewport click surface. Padding positions the
              // dialog within the work area without cutting off backdrop dismissal.
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              boxSizing: "border-box",
              paddingTop: bounds.top,
              paddingLeft: bounds.left,
              paddingRight: Math.max(0, window.innerWidth - bounds.left - bounds.width),
              paddingBottom: Math.max(
                0,
                window.innerHeight - bounds.top - bounds.height,
              ),
            }
          : undefined,
        container: {
          display: "flex",
          flexDirection: "column",
          maxHeight: maximumHeight,
        },
        header: { flexShrink: 0 },
        body: { overflowY: "auto", minHeight: 0 },
        footer: { flexShrink: 0 },
      }}
    />
  );
}
