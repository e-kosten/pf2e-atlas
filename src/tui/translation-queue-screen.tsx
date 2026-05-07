import React from "react";

import type { SearchCategory } from "../domain/search-types.js";
import type { DerivedTagTranslationRecord } from "../domain/derived-tag-types.js";
import { DERIVED_TAG_MANAGED_CATEGORIES } from "../tags/manifest.js";
import {
  buildDerivedTagTerminalActionTargetHelpLines,
  createMergedReturnFooterBinding,
  createSharedReturnInteractionActions,
  moveSelectionWrapped,
  TerminalActionMenuScreen,
  type DerivedTagTerminalActionTargetOption,
  type DerivedTagTerminalLine,
  type RouteTransitionStatus,
  type TerminalInteractionAction,
  type TerminalMenuScreenInteractions,
} from "./terminal-ui.js";

type TranslationQueueCategoryFilter = SearchCategory | "all";
type TranslationQueueStatusFilter = "all" | "provisional" | "unmapped";
type TranslationQueueActionId = "cycle_category" | "cycle_status" | "reset_filters";

type TranslationQueueMenuItem = {
  label: string;
  translation: DerivedTagTranslationRecord;
};

function formatFilterLabel(value: TranslationQueueCategoryFilter | TranslationQueueStatusFilter): string {
  return value === "all" ? "all" : value;
}

function buildTranslationQueueMenuItems(items: DerivedTagTranslationRecord[]): TranslationQueueMenuItem[] {
  return items.map((translation) => ({
    label: `[${translation.translationStatus}] ${translation.currentCategory} / ${translation.currentFamily} / ${translation.currentTag} -> ${translation.canonicalConceptLabel}`,
    translation,
  }));
}

function buildTranslationDetailLines(args: {
  selectedItem: TranslationQueueMenuItem | undefined;
  totalCount: number;
  visibleCount: number;
  categoryFilter: TranslationQueueCategoryFilter;
  statusFilter: TranslationQueueStatusFilter;
}): DerivedTagTerminalLine[] {
  const summaryLines: DerivedTagTerminalLine[] = [
    { text: "Translation queue", tone: "section" },
    { text: `Showing ${args.visibleCount} of ${args.totalCount} unresolved rows.`, indent: 2 },
    { text: `Category filter: ${formatFilterLabel(args.categoryFilter)}`, indent: 2 },
    { text: `Status filter: ${formatFilterLabel(args.statusFilter)}`, indent: 2 },
  ];

  const selected = args.selectedItem?.translation;
  if (!selected) {
    return [
      ...summaryLines,
      { text: "" },
      { text: "No translation row is selected for the current filters.", tone: "dim" },
    ];
  }

  const facetLine =
    selected.primaryFacetKind && selected.primaryFacetValue
      ? `${selected.primaryFacetKind} / ${selected.primaryFacetValue}`
      : "(none)";

  const lines: DerivedTagTerminalLine[] = [
    ...summaryLines,
    { text: "" },
    { text: selected.currentTag, tone: "section" },
    { text: `Status: ${selected.translationStatus}`, indent: 2 },
    { text: `Current category: ${selected.currentCategory}`, indent: 2 },
    { text: `Current axis: ${selected.currentBrowseAxis}`, indent: 2 },
    { text: `Current family: ${selected.currentFamily}`, indent: 2 },
    { text: `Current assignment mode: ${selected.currentAssignmentMode}`, indent: 2 },
    { text: "" },
    { text: "Proposed canonical target", tone: "section" },
    { text: `Label: ${selected.canonicalConceptLabel}`, indent: 2 },
    { text: `ID: ${selected.canonicalConceptId}`, indent: 2 },
    { text: `Schema: ${selected.schemaKind}`, indent: 2 },
    { text: `Projection axis: ${selected.projectionAxis}`, indent: 2 },
    { text: `Projection family: ${selected.projectionFamily}`, indent: 2 },
    { text: `Primary facet: ${facetLine}`, indent: 2 },
  ];

  if (selected.domainId) {
    lines.push({ text: `Domain: ${selected.domainId}`, indent: 2 });
  }
  if (selected.operation) {
    lines.push({ text: `Operation: ${selected.operation}`, indent: 2 });
  }
  if (selected.renameNote) {
    lines.push({ text: "" }, { text: "Rename note", tone: "section" }, { text: selected.renameNote, indent: 2 });
  }
  if (selected.notes) {
    lines.push({ text: "" }, { text: "Editorial notes", tone: "section" }, { text: selected.notes, indent: 2 });
  }

  return lines;
}

function getTranslationQueueInteractionActions(): TerminalInteractionAction[] {
  return [
    { id: "actions", helpText: "focus the translation-review actions rail" },
    { id: "help", helpText: "show this help" },
    ...createSharedReturnInteractionActions("tag refinement").map((action) => ({
      ...action,
      helpText: "return to tag refinement",
    })),
  ];
}

function buildTranslationQueueActionEntries(args: {
  categoryFilter: TranslationQueueCategoryFilter;
  statusFilter: TranslationQueueStatusFilter;
}): DerivedTagTerminalActionTargetOption<TranslationQueueActionId>[] {
  return [
    {
      id: "cycle_category",
      label: `Cycle Category (${formatFilterLabel(args.categoryFilter)})`,
      description: "Rotate the category filter across unresolved translation rows.",
    },
    {
      id: "cycle_status",
      label: `Cycle Status (${formatFilterLabel(args.statusFilter)})`,
      description: "Rotate between all unresolved rows, provisional only, and unmapped only.",
    },
    {
      id: "reset_filters",
      label: "Reset Filters",
      description: "Show all unresolved translation rows again.",
    },
  ];
}

function buildTranslationQueueHelpLines(
  actionEntries: DerivedTagTerminalActionTargetOption<TranslationQueueActionId>[],
): DerivedTagTerminalLine[] {
  return buildDerivedTagTerminalActionTargetHelpLines({
    orientation: "horizontal",
    visibility: "onDemand",
    actions: actionEntries,
    contentHelpText: "Use the action rail to cycle category and translation-status filters.",
  });
}

function createTranslationQueueInteractions(
  actionEntries: DerivedTagTerminalActionTargetOption<TranslationQueueActionId>[],
): TerminalMenuScreenInteractions {
  return {
    actions: getTranslationQueueInteractionActions(),
    footerBindings: [
      { kind: "action", action: { id: "actions" } },
      { kind: "action", action: { id: "help" } },
      createMergedReturnFooterBinding("tag refinement"),
    ],
    help: {
      title: "Ontology Translation Review Help",
      sections: [
        {
          title: "Navigation",
          actions: [
            { id: "move", helpText: "move between unresolved translation rows" },
            { id: "jump", helpText: "jump through the list" },
            { id: "page", helpText: "page through the list" },
            { id: "edge", helpText: "jump to the first or last row" },
          ],
        },
        {
          title: "Actions",
          actions: getTranslationQueueInteractionActions(),
        },
      ],
      appendix: buildTranslationQueueHelpLines(actionEntries),
    },
  };
}

function cycleValue<T extends string>(values: readonly T[], current: T): T {
  const currentIndex = values.indexOf(current);
  if (currentIndex < 0) {
    return values[0] ?? current;
  }
  return values[(currentIndex + 1) % values.length] ?? current;
}

function filterTranslationQueueItems(args: {
  items: DerivedTagTranslationRecord[];
  categoryFilter: TranslationQueueCategoryFilter;
  statusFilter: TranslationQueueStatusFilter;
}): DerivedTagTranslationRecord[] {
  return args.items.filter((item) => {
    if (args.categoryFilter !== "all" && item.currentCategory !== args.categoryFilter) {
      return false;
    }
    if (args.statusFilter !== "all" && item.translationStatus !== args.statusFilter) {
      return false;
    }
    return true;
  });
}

export function DerivedTagTranslationQueueScreen({
  items,
  initialCategory = "all",
  initialStatus = "all",
  onBack,
  transitionStatus,
}: {
  items: DerivedTagTranslationRecord[];
  initialCategory?: TranslationQueueCategoryFilter;
  initialStatus?: TranslationQueueStatusFilter;
  onBack: () => void;
  transitionStatus?: RouteTransitionStatus | null;
}): React.JSX.Element {
  const categoryFilterValues = React.useMemo<readonly TranslationQueueCategoryFilter[]>(() => {
    const categories = DERIVED_TAG_MANAGED_CATEGORIES.filter((category) =>
      items.some((item) => item.currentCategory === category),
    );
    return ["all", ...categories];
  }, [items]);
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const [categoryFilter, setCategoryFilter] = React.useState<TranslationQueueCategoryFilter>(initialCategory);
  const [statusFilter, setStatusFilter] = React.useState<TranslationQueueStatusFilter>(initialStatus);

  const filteredItems = React.useMemo(
    () =>
      filterTranslationQueueItems({
        items,
        categoryFilter,
        statusFilter,
      }),
    [categoryFilter, items, statusFilter],
  );
  const menuItems = React.useMemo(() => buildTranslationQueueMenuItems(filteredItems), [filteredItems]);
  const actionEntries = React.useMemo(
    () =>
      buildTranslationQueueActionEntries({
        categoryFilter,
        statusFilter,
      }),
    [categoryFilter, statusFilter],
  );
  const interactions = React.useMemo(() => createTranslationQueueInteractions(actionEntries), [actionEntries]);

  React.useEffect(() => {
    setSelectedIndex((current) => Math.max(0, Math.min(current, Math.max(0, menuItems.length - 1))));
  }, [menuItems.length]);

  return (
    <TerminalActionMenuScreen
      title="Ontology Translation Review"
      subtitle={`${filteredItems.length} unresolved row${filteredItems.length === 1 ? "" : "s"} shown`}
      leftTitle="Unresolved Rows"
      rightTitle="Translation Detail"
      leftWidth={52}
      items={menuItems}
      selectedIndex={selectedIndex}
      interactions={interactions}
      actionEntries={actionEntries}
      buildRightLines={(selectedItem) =>
        buildTranslationDetailLines({
          selectedItem,
          totalCount: items.length,
          visibleCount: filteredItems.length,
          categoryFilter,
          statusFilter,
        })
      }
      buildStatusLine={({ selectedItem }) => ({
        text: selectedItem
          ? `Selected: ${selectedItem.translation.currentCategory} / ${selectedItem.translation.currentTag}`
          : `Filters: ${formatFilterLabel(categoryFilter)} | ${formatFilterLabel(statusFilter)}`,
        tone: "accent",
      })}
      onMove={(delta, itemCount) =>
        setSelectedIndex((current) => {
          if (itemCount <= 0) {
            return 0;
          }
          if (delta === 0) {
            return Math.max(0, Math.min(current, itemCount - 1));
          }
          return moveSelectionWrapped(current, delta, itemCount);
        })
      }
      onSelect={() => {}}
      onBack={onBack}
      onAction={(actionId) => {
        if (actionId === "cycle_category") {
          setCategoryFilter((current) => cycleValue(categoryFilterValues, current));
          return;
        }
        if (actionId === "cycle_status") {
          setStatusFilter((current) => cycleValue(["all", "provisional", "unmapped"] as const, current));
          return;
        }
        setCategoryFilter("all");
        setStatusFilter("all");
      }}
      transitionStatus={transitionStatus}
    />
  );
}
