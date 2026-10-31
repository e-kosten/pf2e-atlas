import {
  useCallback,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type RefObject,
} from "react";
import type { ModalProps } from "antd";

export type ModalLayout = "form" | "browse" | "confirmation";
export type ModalWorkArea = { root: RefObject<HTMLElement | null>; selector?: string };
function workAreaElement(area?: ModalWorkArea) {
  const root = area?.root.current;
  return area?.selector
    ? (root?.querySelector<HTMLElement>(area.selector) ?? null)
    : (root ?? null);
}
type Bounds = { top: number; left: number; width: number; height: number };
type Area = { viewport: Bounds; bounds: Bounds };

function visibleArea(workArea?: ModalWorkArea): Area {
  const view = window.visualViewport;
  const viewport = {
    top: view?.offsetTop ?? 0,
    left: view?.offsetLeft ?? 0,
    width: view?.width ?? window.innerWidth,
    height: view?.height ?? window.innerHeight,
  };
  const rect = workAreaElement(workArea)?.getBoundingClientRect();
  if (!rect || rect.bottom <= viewport.top) return { viewport, bounds: viewport };
  const top = Math.max(viewport.top, rect.top);
  const left = Math.max(viewport.left, rect.left);
  const right = Math.min(viewport.left + viewport.width, rect.right);
  const height = viewport.top + viewport.height - top;
  return {
    viewport,
    bounds:
      height < Math.min(240, viewport.height) ||
      right - left < Math.min(240, viewport.width)
        ? viewport
        : { top, left, width: right - left, height },
  };
}

function sameBounds(a: Bounds, b: Bounds) {
  return (
    a.top === b.top && a.left === b.left && a.width === b.width && a.height === b.height
  );
}

export function useModalLayout({
  open,
  layout,
  workArea,
  frame,
}: {
  open: boolean;
  layout: ModalLayout;
  workArea?: ModalWorkArea;
  frame: RefObject<HTMLDivElement | null>;
}) {
  const [area, setArea] = useState(() => visibleArea());
  const [confirmationInset, setConfirmationInset] = useState<number | null>(null);
  const captured = useRef(false);
  const areaRoot = workArea?.root;
  const areaSelector = workArea?.selector;
  const reset = useCallback(() => {
    captured.current = false;
    setConfirmationInset(null);
  }, []);
  useLayoutEffect(() => {
    if (!open) return;
    const selectedArea = areaRoot
      ? { root: areaRoot, selector: areaSelector }
      : undefined;
    const measure = () => {
      const next = visibleArea(selectedArea);
      setArea((previous) =>
        sameBounds(previous.bounds, next.bounds) &&
        sameBounds(previous.viewport, next.viewport)
          ? previous
          : next,
      );
      if (
        layout === "confirmation" &&
        !captured.current &&
        frame.current?.offsetHeight
      ) {
        captured.current = true;
        setConfirmationInset(
          Math.max(16, (next.bounds.height - frame.current.offsetHeight) / 2),
        );
      }
    };
    measure();
    window.addEventListener("resize", measure);
    window.visualViewport?.addEventListener("resize", measure);
    window.visualViewport?.addEventListener("scroll", measure);
    const observer = new ResizeObserver(measure);
    const element = workAreaElement(selectedArea);
    if (element) observer.observe(element);
    if (layout === "confirmation" && frame.current) observer.observe(frame.current);
    return () => {
      window.removeEventListener("resize", measure);
      window.visualViewport?.removeEventListener("resize", measure);
      window.visualViewport?.removeEventListener("scroll", measure);
      observer.disconnect();
    };
  }, [open, layout, areaRoot, areaSelector, frame]);
  return useMemo(() => {
    const { bounds } = area;
    const inset =
      layout === "browse"
        ? 16
        : layout === "confirmation"
          ? Math.min(confirmationInset ?? 16, Math.max(16, bounds.height - 240))
          : Math.max(16, Math.min(80, bounds.height * 0.08));
    const availableHeight = Math.max(0, bounds.height - inset - 16);
    const height =
      layout === "browse" ? Math.min(900, availableHeight) : availableHeight;
    const style: CSSProperties & { "--atlas-modal-height": string } = {
      "--atlas-modal-height": `${height}px`,
      top: 0,
      marginBlock: 0,
      paddingBottom: 0,
      maxWidth: "100%",
      maxHeight: height,
      display: "flex",
      flexDirection: "column",
      ...(layout === "browse" ? { height } : {}),
    };
    const wrapper: CSSProperties = {
      display: "flex",
      alignItems: "flex-start",
      justifyContent: "center",
      boxSizing: "border-box",
      overflow: "hidden",
      paddingTop: bounds.top + inset,
      paddingBottom: Math.max(0, window.innerHeight - bounds.top - bounds.height) + 16,
      paddingLeft: bounds.left + 16,
      paddingRight: Math.max(0, window.innerWidth - bounds.left - bounds.width) + 16,
    };
    const styles: ModalProps["styles"] = { wrapper };
    return { style, styles, reset };
  }, [area, layout, confirmationInset, reset]);
}
