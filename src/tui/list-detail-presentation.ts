import React from "react";

import {
  getRenderedTerminalLineCount,
  sliceRenderedTerminalLines,
} from "./framework/line-rendering.js";
import { getTerminalPaneBodyHeight, getTerminalTwoPaneDetailWidth } from "./framework/screen-layout.js";
import type {
  DerivedTagTerminalHyperlinkSupport,
  DerivedTagTerminalLine,
  DerivedTagTerminalPaneScreenProps,
  DerivedTagTerminalTone,
  DerivedTagTerminalTwoPaneFocus,
  DerivedTagTerminalTwoPaneLayoutMode,
  DerivedTagTerminalTwoPaneScreenProps,
} from "./framework/types.js";
import type { TerminalInteractionAction } from "./interaction-bindings.js";
import {
  createTerminalActionTargetInteractionContext,
  createTerminalDetailInteractionContext,
  createTerminalListInteractionContext,
  createTerminalTextEntryInteractionContext,
  useTerminalInteractionContextRouter,
  type TerminalInteractionContextRoute,
} from "./interaction-context-router.js";
import {
  ROUTE_TRANSITION_STATUS_KIND,
  appendRouteTransitionFooterLine,
  getRouteTransitionFooterLineCount,
  type RouteTransitionStatus,
} from "./route-transition-status.js";
import { buildTerminalListDetailGroupLine, type TerminalListDetailGroup } from "./list-detail-formatting.js";
import type {
  DerivedTagTerminalActionTargetOrientation,
  DerivedTagTerminalActionTargetState,
} from "./action-target.js";
import type { DerivedTagTerminalPointerEvent } from "./framework/types.js";

export const TERMINAL_LIST_DETAIL_NOTIFICATION_DURATION_MS = 1800;

export type TerminalListDetailNotificationTone = Extract<
  DerivedTagTerminalTone,
  "accent" | "success" | "warning" | "danger"
>;

export type TerminalListDetailNotification = {
  message: string;
  tone: TerminalListDetailNotificationTone;
  expiresAt: number;
};

export function createTerminalListDetailNotification(options: {
  message: string;
  tone?: TerminalListDetailNotificationTone;
  durationMs?: number;
  now?: number;
}): TerminalListDetailNotification {
  const durationMs = options.durationMs ?? TERMINAL_LIST_DETAIL_NOTIFICATION_DURATION_MS;
  const now = options.now ?? Date.now();

  return {
    message: options.message,
    tone: options.tone ?? "accent",
    expiresAt: now + durationMs,
  };
}

export function getActiveTerminalListDetailNotification(
  notification: TerminalListDetailNotification | null | undefined,
  now = Date.now(),
): TerminalListDetailNotification | null {
  return notification && notification.expiresAt > now ? notification : null;
}

export function buildTerminalListDetailNotificationLine(
  notification: TerminalListDetailNotification | null | undefined,
  now = Date.now(),
): DerivedTagTerminalLine | null {
  const activeNotification = getActiveTerminalListDetailNotification(notification, now);
  if (!activeNotification) {
    return null;
  }

  return {
    text: activeNotification.message,
    tone: activeNotification.tone,
    noWrap: true,
  };
}

export function useTerminalListDetailNotification(): {
  notification: TerminalListDetailNotification | null;
  showNotification: (options: {
    message: string;
    tone?: TerminalListDetailNotificationTone;
    durationMs?: number;
  }) => void;
  clearNotification: () => void;
} {
  const [notification, setNotification] = React.useState<TerminalListDetailNotification | null>(null);

  const clearNotification = React.useCallback(() => {
    setNotification(null);
  }, []);

  const showNotification = React.useCallback(
    (options: {
      message: string;
      tone?: TerminalListDetailNotificationTone;
      durationMs?: number;
    }) => {
      setNotification(createTerminalListDetailNotification(options));
    },
    [],
  );

  React.useEffect(() => {
    if (!notification) {
      return undefined;
    }

    const remainingMs = notification.expiresAt - Date.now();
    if (remainingMs <= 0) {
      setNotification(null);
      return undefined;
    }

    const timeout = setTimeout(() => {
      setNotification((current) =>
        current && current.expiresAt === notification.expiresAt && current.message === notification.message
          ? null
          : current,
      );
    }, remainingMs);

    return () => {
      clearTimeout(timeout);
    };
  }, [notification]);

  return {
    notification: getActiveTerminalListDetailNotification(notification),
    showNotification,
    clearNotification,
  };
}

export type TerminalListDetailPresentationMetrics = {
  bodyHeight: number;
  detailWidth: number;
  pageSize: number;
  selectionJumpSize: number;
  detailJumpSize: number;
  renderedDetailLineCount: number;
  maxDetailScroll: number;
  detailScroll: number;
  visibleDetailLines: DerivedTagTerminalLine[];
};

export type TerminalListDetailScreenModel =
  | { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps }
  | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps };

export function buildTerminalGroupedListLines<T>(options: {
  items: readonly T[];
  selectedIndex: number;
  buildItemLine: (item: T, options: { selected: boolean; itemIndex: number }) => DerivedTagTerminalLine;
  getGroup?: ((item: T) => TerminalListDetailGroup | null | undefined) | null;
}): DerivedTagTerminalLine[] {
  if (!options.getGroup) {
    return options.items.map((item, itemIndex) =>
      options.buildItemLine(item, {
        selected: itemIndex === options.selectedIndex,
        itemIndex,
      }),
    );
  }

  const lines: DerivedTagTerminalLine[] = [];
  let previousGroupKey: string | null = null;
  options.items.forEach((item, itemIndex) => {
    const group = options.getGroup?.(item) ?? null;
    if (group && group.key !== previousGroupKey) {
      if (lines.length > 0) {
        lines.push({ text: "" });
      }
      lines.push(buildTerminalListDetailGroupLine(group));
      previousGroupKey = group.key;
    }
    lines.push(
      options.buildItemLine(item, {
        selected: itemIndex === options.selectedIndex,
        itemIndex,
      }),
    );
  });
  return lines;
}

export type MeasureTerminalListDetailPresentationOptions = {
  terminalWidth: number;
  terminalHeight: number;
  hasSubtitle?: boolean;
  transitionStatus?: RouteTransitionStatus | null;
  footerLineCount: number;
  notification?: TerminalListDetailNotification | null;
  detailLines: DerivedTagTerminalLine[];
  detailScroll: number;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  leftWidth: number;
  hyperlinkSupport?: DerivedTagTerminalHyperlinkSupport;
};

export function measureTerminalListDetailPresentation(
  options: MeasureTerminalListDetailPresentationOptions,
): TerminalListDetailPresentationMetrics {
  const activeNotification = getActiveTerminalListDetailNotification(options.notification);
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(options.terminalHeight, {
      hasSubtitle: options.hasSubtitle ?? true,
      footerLineCount:
        options.footerLineCount +
        (activeNotification ? 1 : 0) +
        getRouteTransitionFooterLineCount(options.transitionStatus),
    }),
  );
  const detailWidth = getTerminalTwoPaneDetailWidth(options.terminalWidth, options.layoutMode, options.leftWidth);
  const renderedDetailLineCount = getRenderedTerminalLineCount(options.detailLines, detailWidth, {
    hyperlinkSupport: options.hyperlinkSupport,
  });
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(options.detailScroll, maxDetailScroll);

  return {
    bodyHeight,
    detailWidth,
    pageSize: Math.max(1, bodyHeight - 1),
    selectionJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    detailJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    renderedDetailLineCount,
    maxDetailScroll,
    detailScroll,
    visibleDetailLines: sliceRenderedTerminalLines(options.detailLines, detailWidth, detailScroll, bodyHeight, {
      hyperlinkSupport: options.hyperlinkSupport,
    }),
  };
}

export function buildTerminalListDetailScreenModel(options: {
  title: string;
  subtitle?: string;
  activePane: DerivedTagTerminalTwoPaneFocus;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  leftWidth: number;
  leftPane: {
    title: string;
    lines: DerivedTagTerminalLine[];
  };
  rightPane: {
    title: string;
    detailOnlyTitle?: string;
  };
  metrics: Pick<TerminalListDetailPresentationMetrics, "visibleDetailLines">;
  footer: DerivedTagTerminalLine[];
  notification?: TerminalListDetailNotification | null;
  transitionStatus?: RouteTransitionStatus | null;
  pointerRegions?: {
    list?: {
      onPointerEvent: (event: DerivedTagTerminalPointerEvent) => boolean | void;
      priority?: number;
    };
    detail?: {
      onPointerEvent: (event: DerivedTagTerminalPointerEvent) => boolean | void;
      priority?: number;
    };
  };
}): TerminalListDetailScreenModel {
  const notificationLine = buildTerminalListDetailNotificationLine(options.notification);
  const footer = appendRouteTransitionFooterLine(
    notificationLine ? [...options.footer, notificationLine] : options.footer,
    options.transitionStatus,
  );

  if (options.layoutMode === "detail-only") {
    return {
      kind: "detail-only",
      props: {
        title: options.title,
        subtitle: options.subtitle,
        pane: {
          title: options.rightPane.detailOnlyTitle ?? options.rightPane.title,
          lines: options.metrics.visibleDetailLines,
          active: true,
          pointerRegion: options.pointerRegions?.detail,
        },
        footer,
      },
    };
  }

  return {
    kind: "two-pane",
    props: {
      title: options.title,
      subtitle: options.subtitle,
      left: {
        title: options.leftPane.title,
        lines: options.leftPane.lines,
        active: options.activePane === "list",
        pointerRegion: options.pointerRegions?.list,
      },
      right: {
        title: options.rightPane.title,
        lines: options.metrics.visibleDetailLines,
        active: options.activePane === "detail",
        pointerRegion: options.pointerRegions?.detail,
      },
      footer,
      leftWidth: options.leftWidth,
    },
  };
}

export type TerminalListDetailRouteSet = {
  list: TerminalInteractionContextRoute<string>;
  detail: TerminalInteractionContextRoute<string>;
  textEntry?: TerminalInteractionContextRoute<string>;
  actionTarget?: TerminalInteractionContextRoute<string>;
};

export type TerminalListDetailInteractionConfig = {
  interactionActions?: TerminalInteractionAction[];
  pageSize: number;
  jumpSize: number;
};

export function useTerminalListDetailInteractionRouter(options: {
  enabled?: boolean;
  transitionStatus?: RouteTransitionStatus | null;
  list: TerminalListDetailInteractionConfig & {
    includeConfirmKeys?: boolean;
    includeHorizontalConfirmKeys?: boolean;
  };
  detail: TerminalListDetailInteractionConfig & {
    mode?: "viewport" | "hybrid";
    includeConfirmKeys?: boolean;
    includeHorizontalConfirmKeys?: boolean;
    includeCancelKeys?: boolean;
    includeHorizontalCancelKeys?: boolean;
  };
  textEntry?: {
    interactionActions?: TerminalInteractionAction[];
  };
  actionTarget?: {
    interactionActions?: TerminalInteractionAction[];
    state: DerivedTagTerminalActionTargetState;
    orientation: DerivedTagTerminalActionTargetOrientation;
  };
  onRoute: (routes: TerminalListDetailRouteSet) => void;
}): void {
  useTerminalInteractionContextRouter({
    enabled: options.enabled,
    contexts: [
      createTerminalListInteractionContext("list", {
        interactionActions: options.list.interactionActions,
        pageSize: options.list.pageSize,
        jumpSize: options.list.jumpSize,
        includeConfirmKeys: options.list.includeConfirmKeys,
        includeHorizontalConfirmKeys: options.list.includeHorizontalConfirmKeys,
      }),
      createTerminalDetailInteractionContext("detail", {
        interactionActions: options.detail.interactionActions,
        pageSize: options.detail.pageSize,
        jumpSize: options.detail.jumpSize,
        mode: options.detail.mode,
        includeConfirmKeys: options.detail.includeConfirmKeys,
        includeHorizontalConfirmKeys: options.detail.includeHorizontalConfirmKeys,
        includeCancelKeys: options.detail.includeCancelKeys,
        includeHorizontalCancelKeys: options.detail.includeHorizontalCancelKeys,
      }),
      ...(options.textEntry
        ? [createTerminalTextEntryInteractionContext("textEntry", options.textEntry.interactionActions)]
        : []),
      ...(options.actionTarget
        ? [
            createTerminalActionTargetInteractionContext("actionTarget", {
              interactionActions: options.actionTarget.interactionActions,
              state: options.actionTarget.state,
              orientation: options.actionTarget.orientation,
            }),
          ]
        : []),
    ],
    onRoute: (routes) => {
      if (options.transitionStatus?.kind === ROUTE_TRANSITION_STATUS_KIND.PENDING) {
        return;
      }

      options.onRoute({
        list: routes.list,
        detail: routes.detail,
        textEntry: options.textEntry ? routes.textEntry : undefined,
        actionTarget: options.actionTarget ? routes.actionTarget : undefined,
      });
    },
  });
}
