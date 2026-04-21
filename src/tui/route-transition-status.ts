import type { DerivedTagTerminalLine, DerivedTagTerminalSegment, DerivedTagTerminalTone } from "./framework/types.js";

export const ROUTE_TRANSITION_STATUS_KIND = {
  PENDING: "pending",
  ERROR: "error",
} as const;

export type RouteTransitionStatus =
  | {
      kind: (typeof ROUTE_TRANSITION_STATUS_KIND)["PENDING"];
      message: string;
      frame?: number;
    }
  | {
      kind: (typeof ROUTE_TRANSITION_STATUS_KIND)["ERROR"];
      message: string;
    };

const ROUTE_TRANSITION_INDICATOR_TONES: readonly DerivedTagTerminalTone[] = ["accent", "success", "warning"];

function buildRouteTransitionIndicatorSegments(frame: number): DerivedTagTerminalSegment[] {
  const activeIndex = Math.abs(frame) % ROUTE_TRANSITION_INDICATOR_TONES.length;

  return [
    { text: "[", tone: "dim" },
    ...ROUTE_TRANSITION_INDICATOR_TONES.flatMap((tone, index) => [
      { text: index === activeIndex ? "●" : "•", tone: index === activeIndex ? tone : "dim" },
      ...(index < ROUTE_TRANSITION_INDICATOR_TONES.length - 1 ? [{ text: " ", tone: "dim" as const }] : []),
    ]),
    { text: "]", tone: "dim" },
  ];
}

export function buildRouteTransitionStatusBody(status: RouteTransitionStatus): DerivedTagTerminalLine[] {
  if (status.kind === ROUTE_TRANSITION_STATUS_KIND.PENDING) {
    const frame = status.frame ?? 0;
    return [
      {
        text: status.message,
        segments: [...buildRouteTransitionIndicatorSegments(frame), { text: "  ", tone: "dim" }, { text: status.message, tone: "section" }],
        noWrap: true,
      },
    ];
  }

  return [
    { text: "Could not complete the route transition.", tone: "section" },
    { text: "" },
    { text: status.message },
  ];
}

export function buildRouteTransitionStatusFooter(status: RouteTransitionStatus): DerivedTagTerminalLine[] {
  return [
    status.kind === ROUTE_TRANSITION_STATUS_KIND.PENDING
      ? {
          text: `Loading next view | ${status.message}`,
          segments: [
            ...buildRouteTransitionIndicatorSegments(status.frame ?? 0),
            { text: "  ", tone: "dim" },
            { text: "Loading next view", tone: "dim" },
            { text: " | ", tone: "dim" },
            { text: status.message, tone: "accent" },
          ],
          noWrap: true,
        }
      : {
          text: "Route transition failed.",
          tone: "danger",
        },
  ];
}

export function buildRouteTransitionStatusFooterLine(status: RouteTransitionStatus): DerivedTagTerminalLine {
  if (status.kind === ROUTE_TRANSITION_STATUS_KIND.PENDING) {
    return {
      text: `Loading next view | ${status.message}`,
      segments: [
        ...buildRouteTransitionIndicatorSegments(status.frame ?? 0),
        { text: "  ", tone: "dim" },
        { text: "Loading next view", tone: "dim" },
        { text: " | ", tone: "dim" },
        { text: status.message, tone: "accent" },
      ],
      noWrap: true,
    };
  }

  return {
    text: `Route transition failed | ${status.message}`,
    tone: "danger",
  };
}

export function appendRouteTransitionFooterLine(
  footer: DerivedTagTerminalLine[],
  status: RouteTransitionStatus | null | undefined,
): DerivedTagTerminalLine[] {
  return status ? [...footer, buildRouteTransitionStatusFooterLine(status)] : footer;
}

export function getRouteTransitionFooterLineCount(status: RouteTransitionStatus | null | undefined): number {
  return status ? 1 : 0;
}
