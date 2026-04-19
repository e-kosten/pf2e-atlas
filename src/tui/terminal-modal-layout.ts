export type TerminalModalLayoutKind = "dialog" | "text" | "command" | "select" | "multiselect" | "policy";
export type TerminalModalPresentation = "inline" | "screen";
export type TerminalModalPaneMode = "single-column" | "two-pane";
export type TerminalModalOverflowMode = "fit" | "scroll" | "window";

export type TerminalModalOverflowPolicy = {
  body: TerminalModalOverflowMode;
  list: TerminalModalOverflowMode;
  detail: TerminalModalOverflowMode;
};

export type TerminalModalListSizing = {
  itemCount: number;
  chromeRows?: number;
  minVisibleRowCount?: number;
  preferredVisibleRowCount?: number;
  maxVisibleRowCount?: number;
};

export type TerminalModalDetailSizing = {
  lineCount: number;
  chromeRows?: number;
  minVisibleLineCount?: number;
  preferredVisibleLineCount?: number;
  maxVisibleLineCount?: number;
};

export type TerminalModalBodySizingDescriptor = {
  staticBodyLineCount: number;
  minBodyLineCount?: number;
  maxBodyLineCount?: number;
  minimumInlineWidth?: number;
  preferredPaneMode?: TerminalModalPaneMode | "auto";
  preferredLeftPaneWidth?: number;
  minLeftPaneWidth?: number;
  minRightPaneWidth?: number;
  separatorWidth?: number;
  list?: TerminalModalListSizing;
  detail?: TerminalModalDetailSizing;
};

export type TerminalModalLayoutDefaults = {
  inlineReservedMainRows: number;
  defaultInlineMinimumWidth: number;
  defaultChoiceInlineMinimumWidth: number;
  defaultMinLeftPaneWidth: number;
  defaultMinRightPaneWidth: number;
  defaultPaneSeparatorWidth: number;
  defaultPreferredLeftPaneWidth: number;
  defaultMinVisibleListRows: number;
  defaultPreferredVisibleListRows: number;
  defaultMaxVisibleListRows: number;
  defaultMinVisibleDetailLines: number;
  defaultPreferredVisibleDetailLines: number;
  defaultMaxVisibleDetailLines: number;
};

export type PlanTerminalModalLayoutInput = {
  terminalWidth: number;
  terminalHeight: number;
  kind: TerminalModalLayoutKind;
  forcedPresentation?: TerminalModalPresentation;
  headerRows: number;
  footerRows: number;
  descriptor: TerminalModalBodySizingDescriptor;
  defaults?: Partial<TerminalModalLayoutDefaults>;
};

export type TerminalModalPaneWidths = {
  left: number;
  right: number;
  separator: number;
};

export type TerminalModalChoiceLayout = {
  mode: TerminalModalPaneMode;
  detailWidth: number;
  leftWidth?: number;
  rightWidth?: number;
  separatorWidth?: number;
  visibleListCount: number;
  visibleDetailLineCount: number;
};

export type TerminalModalLayoutRegions = {
  staticRows: number;
  listRows: number;
  detailRows: number;
};

export type TerminalModalLayoutResult = {
  kind: TerminalModalLayoutKind;
  presentation: TerminalModalPresentation;
  totalHeight: number;
  bodyHeight: number;
  showTopBorder: boolean;
  paneMode: TerminalModalPaneMode;
  paneWidths?: TerminalModalPaneWidths;
  visibleListCapacity: number;
  overflowPolicy: TerminalModalOverflowPolicy;
  regions: TerminalModalLayoutRegions;
  reservedMainScreenHeight: number;
  maxInlineTotalHeight: number;
  constrained: boolean;
  choiceLayout?: TerminalModalChoiceLayout;
};

type NormalizedTerminalModalListSizing = Required<TerminalModalListSizing>;
type NormalizedTerminalModalDetailSizing = Required<TerminalModalDetailSizing>;
type NormalizedTerminalModalBodySizingDescriptor = Omit<
  Required<TerminalModalBodySizingDescriptor>,
  "list" | "detail"
> & {
  list: NormalizedTerminalModalListSizing;
  detail: NormalizedTerminalModalDetailSizing;
};

export const TERMINAL_MODAL_LAYOUT_DEFAULTS: TerminalModalLayoutDefaults = {
  inlineReservedMainRows: 6,
  defaultInlineMinimumWidth: 24,
  defaultChoiceInlineMinimumWidth: 32,
  defaultMinLeftPaneWidth: 24,
  defaultMinRightPaneWidth: 20,
  defaultPaneSeparatorWidth: 1,
  defaultPreferredLeftPaneWidth: 38,
  defaultMinVisibleListRows: 4,
  defaultPreferredVisibleListRows: 8,
  defaultMaxVisibleListRows: 12,
  defaultMinVisibleDetailLines: 2,
  defaultPreferredVisibleDetailLines: 6,
  defaultMaxVisibleDetailLines: 10,
};

export function createTerminalMessageSizingDescriptor(options: {
  bodyLineCount: number;
  minBodyLineCount?: number;
  maxBodyLineCount?: number;
  minimumInlineWidth?: number;
}): TerminalModalBodySizingDescriptor {
  return {
    staticBodyLineCount: options.bodyLineCount,
    minBodyLineCount: options.minBodyLineCount,
    maxBodyLineCount: options.maxBodyLineCount,
    minimumInlineWidth: options.minimumInlineWidth,
  };
}

export function createTerminalTextInputSizingDescriptor(options: {
  bodyLineCount: number;
  minBodyLineCount?: number;
  maxBodyLineCount?: number;
  minimumInlineWidth?: number;
}): TerminalModalBodySizingDescriptor {
  return {
    staticBodyLineCount: options.bodyLineCount,
    minBodyLineCount: options.minBodyLineCount,
    maxBodyLineCount: options.maxBodyLineCount,
    minimumInlineWidth: options.minimumInlineWidth,
  };
}

export function createTerminalChoiceSizingDescriptor(options: {
  staticBodyLineCount: number;
  list: TerminalModalListSizing;
  detail?: TerminalModalDetailSizing;
  minimumInlineWidth?: number;
  preferredPaneMode?: TerminalModalPaneMode | "auto";
  preferredLeftPaneWidth?: number;
  minLeftPaneWidth?: number;
  minRightPaneWidth?: number;
  separatorWidth?: number;
}): TerminalModalBodySizingDescriptor {
  return {
    staticBodyLineCount: options.staticBodyLineCount,
    minimumInlineWidth: options.minimumInlineWidth,
    preferredPaneMode: options.preferredPaneMode,
    preferredLeftPaneWidth: options.preferredLeftPaneWidth,
    minLeftPaneWidth: options.minLeftPaneWidth,
    minRightPaneWidth: options.minRightPaneWidth,
    separatorWidth: options.separatorWidth,
    list: options.list,
    detail: options.detail,
  };
}

export function planTerminalModalLayout(input: PlanTerminalModalLayoutInput): TerminalModalLayoutResult {
  const defaults = { ...TERMINAL_MODAL_LAYOUT_DEFAULTS, ...input.defaults };
  const descriptor = normalizeDescriptor(input.kind, input.descriptor, defaults);
  const paneMode = resolvePaneMode(input.terminalWidth, descriptor);
  const paneWidths = paneMode === "two-pane" ? resolvePaneWidths(input.terminalWidth, descriptor) : undefined;
  const idealBodyHeight = getIdealBodyHeight(descriptor, paneMode);
  const minBodyHeight = getMinimumBodyHeight(descriptor, paneMode);
  const forcedPresentation = input.forcedPresentation;
  const maxInlineTotalHeight = forcedPresentation === "inline"
    ? clamp(input.terminalHeight, 0, input.terminalHeight)
    : clamp(input.terminalHeight - defaults.inlineReservedMainRows, 0, input.terminalHeight);
  const inlineBodyCapacity = Math.max(0, maxInlineTotalHeight - input.headerRows - input.footerRows);
  const canInline =
    input.terminalWidth >= descriptor.minimumInlineWidth &&
    inlineBodyCapacity >= minBodyHeight &&
    maxInlineTotalHeight > 0;
  const presentation =
    forcedPresentation ?? (canInline ? "inline" : "screen");
  const totalHeight =
    presentation === "inline"
      ? clamp(idealBodyHeight + input.headerRows + input.footerRows, 0, maxInlineTotalHeight || input.terminalHeight)
      : clamp(input.terminalHeight, 0, input.terminalHeight);
  const bodyHeight = Math.max(0, totalHeight - input.headerRows - input.footerRows);
  const regions = resolveRegions(bodyHeight, descriptor, paneMode);
  const overflowPolicy = resolveOverflowPolicy(descriptor, regions);
  const constrained =
    overflowPolicy.body !== "fit" ||
    overflowPolicy.list !== "fit" ||
    overflowPolicy.detail !== "fit" ||
    (presentation === "inline" && idealBodyHeight + input.headerRows + input.footerRows > maxInlineTotalHeight);

  return {
    kind: input.kind,
    presentation,
    totalHeight,
    bodyHeight,
    showTopBorder: presentation === "inline",
    paneMode,
    paneWidths,
    visibleListCapacity: Math.max(0, regions.listRows - descriptor.list.chromeRows),
    overflowPolicy,
    regions,
    reservedMainScreenHeight: presentation === "inline" ? Math.max(0, input.terminalHeight - totalHeight) : 0,
    maxInlineTotalHeight,
    constrained,
    choiceLayout:
      descriptor.list.itemCount > 0 || descriptor.detail.lineCount > 0
        ? {
            mode: paneMode,
            detailWidth: paneMode === "two-pane" ? (paneWidths?.right ?? input.terminalWidth) : input.terminalWidth,
            leftWidth: paneWidths?.left,
            rightWidth: paneWidths?.right,
            separatorWidth: paneWidths?.separator,
            visibleListCount:
              paneMode === "two-pane"
                ? Math.max(0, bodyHeight - 2)
                : Math.max(0, regions.listRows - descriptor.list.chromeRows),
            visibleDetailLineCount:
              paneMode === "two-pane"
                ? Math.max(0, bodyHeight - 2)
                : Math.max(0, regions.detailRows - descriptor.detail.chromeRows),
          }
        : undefined,
  };
}

function normalizeDescriptor(
  kind: TerminalModalLayoutKind,
  descriptor: TerminalModalBodySizingDescriptor,
  defaults: TerminalModalLayoutDefaults,
): NormalizedTerminalModalBodySizingDescriptor {
  const hasChoiceRegions = Boolean(descriptor.list);
  return {
    staticBodyLineCount: Math.max(0, descriptor.staticBodyLineCount),
    minBodyLineCount: Math.max(0, descriptor.minBodyLineCount ?? descriptor.staticBodyLineCount),
    maxBodyLineCount: Math.max(
      descriptor.minBodyLineCount ?? descriptor.staticBodyLineCount,
      descriptor.maxBodyLineCount ?? Number.POSITIVE_INFINITY,
    ),
    minimumInlineWidth:
      descriptor.minimumInlineWidth ??
      (hasChoiceRegions || isChoiceKind(kind)
        ? defaults.defaultChoiceInlineMinimumWidth
        : defaults.defaultInlineMinimumWidth),
    preferredPaneMode: descriptor.preferredPaneMode ?? "auto",
    preferredLeftPaneWidth: descriptor.preferredLeftPaneWidth ?? defaults.defaultPreferredLeftPaneWidth,
    minLeftPaneWidth: descriptor.minLeftPaneWidth ?? defaults.defaultMinLeftPaneWidth,
    minRightPaneWidth: descriptor.minRightPaneWidth ?? defaults.defaultMinRightPaneWidth,
    separatorWidth: descriptor.separatorWidth ?? defaults.defaultPaneSeparatorWidth,
    list: {
      itemCount: Math.max(0, descriptor.list?.itemCount ?? 0),
      chromeRows: Math.max(0, descriptor.list?.chromeRows ?? 0),
      minVisibleRowCount: Math.max(1, descriptor.list?.minVisibleRowCount ?? defaults.defaultMinVisibleListRows),
      preferredVisibleRowCount: Math.max(
        1,
        descriptor.list?.preferredVisibleRowCount ?? defaults.defaultPreferredVisibleListRows,
      ),
      maxVisibleRowCount: Math.max(1, descriptor.list?.maxVisibleRowCount ?? defaults.defaultMaxVisibleListRows),
    },
    detail: {
      lineCount: Math.max(0, descriptor.detail?.lineCount ?? 0),
      chromeRows: Math.max(0, descriptor.detail?.chromeRows ?? 0),
      minVisibleLineCount: Math.max(0, descriptor.detail?.minVisibleLineCount ?? defaults.defaultMinVisibleDetailLines),
      preferredVisibleLineCount: Math.max(
        0,
        descriptor.detail?.preferredVisibleLineCount ?? defaults.defaultPreferredVisibleDetailLines,
      ),
      maxVisibleLineCount: Math.max(0, descriptor.detail?.maxVisibleLineCount ?? defaults.defaultMaxVisibleDetailLines),
    },
  };
}

function resolvePaneMode(
  terminalWidth: number,
  descriptor: NormalizedTerminalModalBodySizingDescriptor,
): TerminalModalPaneMode {
  if (!descriptor.list.itemCount && descriptor.detail.lineCount === 0) {
    return "single-column";
  }
  if (descriptor.preferredPaneMode === "single-column") {
    return "single-column";
  }
  if (descriptor.detail.lineCount === 0) {
    return "single-column";
  }

  const separator = descriptor.separatorWidth;
  const minimumTwoPaneWidth = descriptor.minLeftPaneWidth + separator + descriptor.minRightPaneWidth;
  return terminalWidth >= minimumTwoPaneWidth ? "two-pane" : "single-column";
}

function resolvePaneWidths(
  terminalWidth: number,
  descriptor: NormalizedTerminalModalBodySizingDescriptor,
): TerminalModalPaneWidths {
  const separator = descriptor.separatorWidth;
  const maximumLeftWidth = Math.max(
    descriptor.minLeftPaneWidth,
    terminalWidth - separator - descriptor.minRightPaneWidth,
  );
  const preferredLeft = clamp(
    descriptor.preferredLeftPaneWidth,
    descriptor.minLeftPaneWidth,
    maximumLeftWidth,
  );
  const right = Math.max(descriptor.minRightPaneWidth, terminalWidth - preferredLeft - separator);
  const left = Math.max(descriptor.minLeftPaneWidth, terminalWidth - right - separator);
  return { left, right, separator };
}

function getIdealBodyHeight(
  descriptor: NormalizedTerminalModalBodySizingDescriptor,
  paneMode: TerminalModalPaneMode,
): number {
  const staticRows = clamp(
    descriptor.staticBodyLineCount,
    descriptor.minBodyLineCount,
    descriptor.maxBodyLineCount,
  );
  if (descriptor.list.itemCount === 0 && descriptor.detail.lineCount === 0) {
    return staticRows;
  }

  const listTargetRows = getPreferredListRegionRows(descriptor.list);
  const detailTargetRows = descriptor.detail.lineCount > 0 ? getPreferredDetailRegionRows(descriptor.detail) : 0;

  if (paneMode === "two-pane") {
    return staticRows + Math.max(listTargetRows, detailTargetRows);
  }

  return staticRows + listTargetRows + detailTargetRows;
}

function getMinimumBodyHeight(
  descriptor: NormalizedTerminalModalBodySizingDescriptor,
  paneMode: TerminalModalPaneMode,
): number {
  const staticRows = Math.max(0, descriptor.minBodyLineCount);
  if (descriptor.list.itemCount === 0 && descriptor.detail.lineCount === 0) {
    return staticRows;
  }

  const listMinimumRows = getMinimumListRegionRows(descriptor.list);
  const detailMinimumRows = descriptor.detail.lineCount > 0 ? getMinimumDetailRegionRows(descriptor.detail) : 0;

  if (paneMode === "two-pane") {
    return staticRows + Math.max(listMinimumRows, detailMinimumRows);
  }

  return staticRows + listMinimumRows + detailMinimumRows;
}

function resolveRegions(
  bodyHeight: number,
  descriptor: NormalizedTerminalModalBodySizingDescriptor,
  paneMode: TerminalModalPaneMode,
): TerminalModalLayoutRegions {
  const staticTargetRows = clamp(
    descriptor.staticBodyLineCount,
    descriptor.minBodyLineCount,
    descriptor.maxBodyLineCount,
  );
  const staticRows = Math.min(staticTargetRows, bodyHeight);
  const availableDynamicRows = Math.max(0, bodyHeight - staticRows);

  if (descriptor.list.itemCount === 0 && descriptor.detail.lineCount === 0) {
    return {
      staticRows,
      listRows: 0,
      detailRows: 0,
    };
  }

  if (paneMode === "two-pane") {
    return {
      staticRows,
      listRows: availableDynamicRows,
      detailRows: availableDynamicRows,
    };
  }

  const listMinimumRows = getMinimumListRegionRows(descriptor.list);
  const listPreferredRows = getPreferredListRegionRows(descriptor.list);
  const detailMinimumRows = descriptor.detail.lineCount > 0 ? getMinimumDetailRegionRows(descriptor.detail) : 0;
  const detailPreferredRows = descriptor.detail.lineCount > 0 ? getPreferredDetailRegionRows(descriptor.detail) : 0;

  if (descriptor.detail.lineCount === 0) {
    return {
      staticRows,
      listRows: Math.min(availableDynamicRows, listPreferredRows),
      detailRows: 0,
    };
  }

  if (availableDynamicRows <= listMinimumRows) {
    return {
      staticRows,
      listRows: availableDynamicRows,
      detailRows: 0,
    };
  }

  if (availableDynamicRows < listMinimumRows + detailMinimumRows) {
    return {
      staticRows,
      listRows: listMinimumRows,
      detailRows: availableDynamicRows - listMinimumRows,
    };
  }

  let remainingRows = availableDynamicRows;
  let listRows = listMinimumRows;
  let detailRows = detailMinimumRows;
  remainingRows -= listMinimumRows + detailMinimumRows;

  const listExtraCapacity = Math.max(0, listPreferredRows - listMinimumRows);
  const detailExtraCapacity = Math.max(0, detailPreferredRows - detailMinimumRows);

  const listExtraRows = Math.min(remainingRows, listExtraCapacity);
  listRows += listExtraRows;
  remainingRows -= listExtraRows;

  const detailExtraRows = Math.min(remainingRows, detailExtraCapacity);
  detailRows += detailExtraRows;
  remainingRows -= detailExtraRows;

  if (remainingRows > 0) {
    listRows += remainingRows;
  }

  return {
    staticRows,
    listRows,
    detailRows,
  };
}

function resolveOverflowPolicy(
  descriptor: NormalizedTerminalModalBodySizingDescriptor,
  regions: TerminalModalLayoutRegions,
): TerminalModalOverflowPolicy {
  const staticTargetRows = clamp(
    descriptor.staticBodyLineCount,
    descriptor.minBodyLineCount,
    descriptor.maxBodyLineCount,
  );
  const listVisibleCapacity = Math.max(0, regions.listRows - descriptor.list.chromeRows);
  const detailVisibleCapacity = Math.max(0, regions.detailRows - descriptor.detail.chromeRows);
  const listVisibleTarget = Math.max(1, descriptor.list.itemCount || 1);

  return {
    body: staticTargetRows > regions.staticRows ? "scroll" : "fit",
    list:
      descriptor.list.itemCount === 0 && regions.listRows === 0
        ? "fit"
        : listVisibleTarget > listVisibleCapacity
          ? "window"
          : "fit",
    detail:
      descriptor.detail.lineCount > detailVisibleCapacity
        ? "scroll"
        : "fit",
  };
}

function getMinimumListRegionRows(list: NormalizedTerminalModalListSizing): number {
  const visibleRowCount = Math.min(Math.max(1, list.itemCount || 1), list.minVisibleRowCount);
  return list.chromeRows + visibleRowCount;
}

function getPreferredListRegionRows(list: NormalizedTerminalModalListSizing): number {
  const contentRowCount = Math.max(1, list.itemCount || 1);
  const preferredVisibleRowCount = clamp(
    list.preferredVisibleRowCount,
    Math.min(contentRowCount, list.minVisibleRowCount),
    list.maxVisibleRowCount,
  );
  return list.chromeRows + Math.min(contentRowCount, preferredVisibleRowCount);
}

function getMinimumDetailRegionRows(detail: NormalizedTerminalModalDetailSizing): number {
  const visibleLineCount = Math.min(detail.lineCount, detail.minVisibleLineCount);
  return detail.chromeRows + visibleLineCount;
}

function getPreferredDetailRegionRows(detail: NormalizedTerminalModalDetailSizing): number {
  const preferredVisibleLineCount = clamp(
    detail.preferredVisibleLineCount,
    Math.min(detail.lineCount, detail.minVisibleLineCount),
    detail.maxVisibleLineCount,
  );
  return detail.chromeRows + Math.min(detail.lineCount, preferredVisibleLineCount);
}

function isChoiceKind(kind: TerminalModalLayoutKind): boolean {
  return kind === "command" || kind === "select" || kind === "multiselect" || kind === "policy";
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(value, max));
}
