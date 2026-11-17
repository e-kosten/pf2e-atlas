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

export type ModalLayout = "form" | "browse" | "document" | "confirmation";
export type ModalWorkArea = { root: RefObject<HTMLElement | null>; selector?: string };
function workAreaElement(area?: ModalWorkArea) {
  const root = area?.root.current;
  return area?.selector
    ? (root?.querySelector<HTMLElement>(area.selector) ?? null)
    : (root ?? null);
}
type Bounds = { top: number; left: number; width: number; height: number };
type Area = { viewport: Bounds; bounds: Bounds };
const gutter = 16;
const preferredBody = 272;
// Grow the preferred inset only with space the work area can spare.
const comfortableWorkArea = 562;
const usableBody = 80;

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
      right - left < Math.min(240, viewport.width)
        ? viewport
        : { top, left, width: right - left, height },
  };
}

function frameMetrics(frame: HTMLDivElement | null) {
  const body = frame?.querySelector<HTMLElement>(".ant-modal-body");
  const container = body?.parentElement;
  if (!frame?.getClientRects().length || !body || !container) return null;
  const verticalMargins = (element: Element) => {
    const style = getComputedStyle(element);
    return parseFloat(style.marginTop) + parseFloat(style.marginBottom);
  };
  const headersAndActions = Array.from(container.children)
    .filter((element) => element.matches(".ant-modal-header, .ant-modal-footer"))
    .reduce(
      (height, el) => height + el.getBoundingClientRect().height + verticalMargins(el),
      0,
    );
  const style = getComputedStyle(container);
  const feedback = frame.firstElementChild?.getBoundingClientRect().height ?? 0;
  return {
    chrome: Math.ceil(
      headersAndActions +
        parseFloat(style.paddingTop) +
        parseFloat(style.paddingBottom) +
        (feedback ? feedback + parseFloat(getComputedStyle(frame).rowGap) : 0),
    ),
    body: Math.min(preferredBody, body.scrollHeight),
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
  frame: HTMLDivElement | null;
}) {
  const [area, setArea] = useState(() => visibleArea());
  const [openingMetrics, setOpeningMetrics] =
    useState<ReturnType<typeof frameMetrics>>(null);
  const [currentChrome, setCurrentChrome] = useState<number | null>(null);
  const [confirmationInset, setConfirmationInset] = useState<number | null>(null);
  const captured = useRef(false);
  const areaRoot = workArea?.root;
  const areaSelector = workArea?.selector;
  const fill = layout === "browse" || layout === "document";
  const reset = useCallback(() => {
    captured.current = false;
    setConfirmationInset(null);
    setOpeningMetrics(null);
    setCurrentChrome(null);
  }, []);
  useLayoutEffect(() => {
    if (!open) return;
    const selectedArea = areaRoot
      ? { root: areaRoot, selector: areaSelector }
      : undefined;
    const measure = () => {
      const next = visibleArea(selectedArea);
      const metrics = frameMetrics(frame);
      if (metrics) {
        setCurrentChrome(metrics.chrome);
        setOpeningMetrics((previous) =>
          previous?.chrome === metrics.chrome && previous.body === metrics.body
            ? previous
            : metrics,
        );
      }
      setArea((previous) =>
        sameBounds(previous.bounds, next.bounds) &&
        sameBounds(previous.viewport, next.viewport)
          ? previous
          : next,
      );
      if (layout === "confirmation" && !captured.current && frame?.offsetHeight) {
        captured.current = true;
        setConfirmationInset(
          Math.max(gutter, (next.bounds.height - frame.offsetHeight) / 2),
        );
      }
    };
    measure();
    window.addEventListener("resize", measure);
    window.visualViewport?.addEventListener("resize", measure);
    window.visualViewport?.addEventListener("scroll", measure);
    // Underlying content can shrink and clamp page scrolling while a task is open.
    // Keep its captured work-area anchor until an explicit viewport change.
    const observer = new ResizeObserver(() => {
      const metrics = frameMetrics(frame);
      if (metrics) setCurrentChrome(metrics.chrome);
      if (layout === "confirmation" && !captured.current) measure();
    });
    for (const element of frame?.querySelectorAll(
      ".ant-modal-header, .ant-modal-footer, .global-feedback-host",
    ) ?? []) {
      observer.observe(element);
    }
    return () => {
      window.removeEventListener("resize", measure);
      window.visualViewport?.removeEventListener("resize", measure);
      window.visualViewport?.removeEventListener("scroll", measure);
      observer.disconnect();
    };
  }, [open, layout, areaRoot, areaSelector, frame]);
  return useMemo(() => {
    const { bounds, viewport } = area;
    const inset = fill
      ? gutter
      : layout === "confirmation"
        ? (confirmationInset ?? gutter)
        : Math.max(
            gutter,
            Math.min(80, bounds.height * 0.08, bounds.height - comfortableWorkArea),
          );
    const bottom = viewport.top + viewport.height - gutter;
    const minimumTop = viewport.top + gutter;
    const chrome = openingMetrics?.chrome ?? 120;
    const body = fill ? usableBody : (openingMetrics?.body ?? preferredBody);
    const top = Math.max(
      minimumTop,
      Math.min(bounds.top + inset, bottom - chrome - body),
    );
    const height = Math.max(0, bottom - top);
    const scrollFrame = height - (currentChrome ?? chrome) < Math.min(body, usableBody);
    const style: CSSProperties & { "--atlas-modal-height": string } = {
      "--atlas-modal-height": `${height}px`,
      top: 0,
      marginBlock: 0,
      paddingBottom: 0,
      maxWidth: "100%",
      maxHeight: height,
      display: "flex",
      flexDirection: "column",
      ...(fill ? { height } : {}),
    };
    const wrapper: CSSProperties = {
      display: "flex",
      alignItems: "flex-start",
      justifyContent: "center",
      boxSizing: "border-box",
      overflow: "hidden",
      paddingTop: top,
      paddingBottom: Math.max(0, window.innerHeight - bottom),
      paddingLeft: bounds.left + gutter,
      paddingRight:
        Math.max(0, window.innerWidth - bounds.left - bounds.width) + gutter,
    };
    const styles: ModalProps["styles"] = { wrapper };
    return { fill, height, scrollFrame, style, styles, reset };
  }, [area, fill, layout, openingMetrics, currentChrome, confirmationInset, reset]);
}
