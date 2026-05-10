import React from "react";

import type { SearchCategory } from "../domain/search-types.js";
import { DERIVED_TAG_MANAGED_CATEGORIES } from "../tags/manifest.js";
import {
  createDerivedTagTranslationReviewSession,
  buildEffectiveDerivedTagTranslationRecord,
  cloneDerivedTagTranslationOverride,
  importDerivedTagTranslationReviewSession,
  type DerivedTagTranslationOverride,
  writeDerivedTagTranslationReviewSession,
} from "../tags/editorial.js";
import type {
  DerivedTagTranslationReviewFilterStatus,
  DerivedTagTranslationReviewRow,
  DerivedTagTranslationReviewSession,
} from "../tags/editorial.js";
import type { DerivedTagTranslationRecord } from "../domain/derived-tag-types.js";
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
import { useDerivedTagTerminalApp } from "./framework/context.js";
import { useTerminalInteractionContextAdapters } from "./interaction-context-adapters.js";

type TranslationQueueCategoryFilter = SearchCategory | "all";
type TranslationQueueStatusFilter = DerivedTagTranslationReviewFilterStatus;
type TranslationEditableFieldId =
  | "targetProjectionId"
  | "renameNote"
  | "notes";
type TranslationQueueActionId =
  | "cycle_category"
  | "cycle_status"
  | "set_translation_status"
  | "edit_field"
  | "reset_row"
  | "import";

type TranslationQueueMenuItem = {
  label: string;
  row: DerivedTagTranslationReviewRow;
  effective: DerivedTagTranslationRecord;
  modified: boolean;
};

function formatFilterLabel(value: TranslationQueueCategoryFilter | TranslationQueueStatusFilter): string {
  return value === "all" ? "all" : value;
}

function overridesEqual(left: DerivedTagTranslationOverride, right: DerivedTagTranslationOverride): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function isRowModified(row: DerivedTagTranslationReviewRow): boolean {
  return !overridesEqual(row.currentOverride, row.draftOverride);
}

function buildTranslationQueueMenuItems(rows: DerivedTagTranslationReviewRow[]): TranslationQueueMenuItem[] {
  return rows.map((row) => {
    const effective = buildEffectiveDerivedTagTranslationRecord(row.base, row.draftOverride);
    const modified = isRowModified(row);
    return {
      label: `${modified ? "*" : " "} [${effective.translationStatus}] ${effective.currentCategory} / ${effective.currentFamily} / ${effective.currentTag} -> ${effective.canonicalConceptLabel}`,
      row,
      effective,
      modified,
    };
  });
}

function buildTranslationDetailLines(args: {
  selectedItem: TranslationQueueMenuItem | undefined;
  totalCount: number;
  visibleCount: number;
  categoryFilter: TranslationQueueCategoryFilter;
  statusFilter: TranslationQueueStatusFilter;
  persistError: string | null;
}): DerivedTagTerminalLine[] {
  const summaryLines: DerivedTagTerminalLine[] = [
    { text: "Translation review session", tone: "section" },
    { text: `Showing ${args.visibleCount} of ${args.totalCount} session rows.`, indent: 2 },
    { text: `Category filter: ${formatFilterLabel(args.categoryFilter)}`, indent: 2 },
    { text: `Status filter: ${formatFilterLabel(args.statusFilter)}`, indent: 2 },
  ];

  if (args.persistError) {
    summaryLines.push({ text: `Persist error: ${args.persistError}`, indent: 2, tone: "warning" });
  }

  const selected = args.selectedItem;
  if (!selected) {
    return [
      ...summaryLines,
      { text: "" },
      { text: "No translation row is selected for the current filters.", tone: "dim" },
    ];
  }

  const facetLine =
    selected.effective.primaryFacetKind && selected.effective.primaryFacetValue
      ? `${selected.effective.primaryFacetKind} / ${selected.effective.primaryFacetValue}`
      : "(none)";
  const targetProjectionLine = selected.effective.targetProjectionId ?? "(none)";
  const targetLabelLine = selected.effective.canonicalConceptLabel || "(none)";
  const targetIdLine = selected.effective.canonicalConceptId || "(none)";

  const lines: DerivedTagTerminalLine[] = [
    ...summaryLines,
    { text: "" },
    { text: selected.effective.currentTag, tone: "section" },
    { text: `Status: ${selected.effective.translationStatus}`, indent: 2 },
    { text: `Modified: ${selected.modified ? "yes" : "no"}`, indent: 2 },
    { text: `Current category: ${selected.effective.currentCategory}`, indent: 2 },
    { text: `Current axis: ${selected.effective.currentBrowseAxis}`, indent: 2 },
    { text: `Current family: ${selected.effective.currentFamily}`, indent: 2 },
    { text: "" },
    { text: "Proposed canonical target", tone: "section" },
    { text: `Projection: ${targetProjectionLine}`, indent: 2 },
    { text: `Label: ${targetLabelLine}`, indent: 2 },
    { text: `ID: ${targetIdLine}`, indent: 2 },
    { text: `Schema: ${selected.effective.schemaKind}`, indent: 2 },
    { text: `Projection axis: ${selected.effective.projectionAxis}`, indent: 2 },
    { text: `Projection family: ${selected.effective.projectionFamily}`, indent: 2 },
    { text: `Primary facet: ${facetLine}`, indent: 2 },
  ];

  if (selected.effective.domainId !== undefined) {
    lines.push({ text: `Domain: ${selected.effective.domainId || "(blank)"}`, indent: 2 });
  }
  if (selected.effective.operation !== undefined) {
    lines.push({ text: `Operation: ${selected.effective.operation || "(blank)"}`, indent: 2 });
  }
  if (selected.effective.renameNote) {
    lines.push({ text: "" }, { text: "Rename note", tone: "section" }, { text: selected.effective.renameNote, indent: 2 });
  }
  if (selected.effective.notes) {
    lines.push({ text: "" }, { text: "Editorial notes", tone: "section" }, { text: selected.effective.notes, indent: 2 });
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
      description: "Rotate the category filter across the translation review session.",
    },
    {
      id: "cycle_status",
      label: `Cycle Status (${formatFilterLabel(args.statusFilter)})`,
      description: "Rotate between all session rows, provisional only, unmapped only, and mapped only.",
    },
    {
      id: "set_translation_status",
      label: "Set Translation Status",
      description: "Mark the selected row mapped, provisional, unmapped, or dropped.",
    },
    {
      id: "edit_field",
      label: "Edit Target Field",
      description: "Edit the selected row's target projection id or editorial notes.",
    },
    {
      id: "reset_row",
      label: "Reset Row",
      description: "Revert the selected row to the currently authored override.",
    },
    {
      id: "import",
      label: "Lint + Import",
      description: "Apply the session into src/tags/translations/tag-overrides.ts after validation.",
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
    contentHelpText:
      "Use the action rail to filter the session, edit per-tag translation overrides, and import the session.",
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
            { id: "move", helpText: "move between translation rows" },
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

function filterTranslationQueueRows(args: {
  rows: DerivedTagTranslationReviewRow[];
  categoryFilter: TranslationQueueCategoryFilter;
  statusFilter: TranslationQueueStatusFilter;
}): DerivedTagTranslationReviewRow[] {
  return args.rows.filter((row) => {
    const effective = buildEffectiveDerivedTagTranslationRecord(row.base, row.draftOverride);
    if (args.categoryFilter !== "all" && effective.currentCategory !== args.categoryFilter) {
      return false;
    }
    if (args.statusFilter !== "all" && effective.translationStatus !== args.statusFilter) {
      return false;
    }
    return true;
  });
}

function fieldMetadata(field: TranslationEditableFieldId): {
  title: string;
  prompt: string;
  hint?: string;
} {
  switch (field) {
    case "targetProjectionId":
      return {
        title: "Target Projection",
        prompt: "Target projection id",
        hint: "Use a canonical projection id like spell:anti_poison. Leave blank only for dropped/unmapped rows.",
      };
    case "renameNote":
      return { title: "Rename Note", prompt: "Rename note", hint: "Optional editorial note about the canonical rename." };
    case "notes":
      return { title: "Editorial Notes", prompt: "Editorial notes", hint: "Optional mapping note for future review." };
  }
}

function setOverrideField(
  override: DerivedTagTranslationOverride,
  field: TranslationEditableFieldId,
  value: string,
): DerivedTagTranslationOverride {
  const next = cloneDerivedTagTranslationOverride(override);
  if (value) {
    (next as Record<TranslationEditableFieldId, string | undefined>)[field] = value;
  } else {
    delete (next as Partial<Record<TranslationEditableFieldId, string>>)[field];
  }
  return next;
}

export function DerivedTagTranslationQueueScreen({
  items,
  rootPath,
  initialCategory = "all",
  initialStatus = "provisional",
  onBack,
  transitionStatus,
}: {
  items: DerivedTagTranslationRecord[];
  rootPath: string;
  initialCategory?: TranslationQueueCategoryFilter;
  initialStatus?: TranslationQueueStatusFilter;
  onBack: () => void;
  transitionStatus?: RouteTransitionStatus | null;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const categoryFilterValues = React.useMemo<readonly TranslationQueueCategoryFilter[]>(() => {
    const categories = DERIVED_TAG_MANAGED_CATEGORIES.filter((category) =>
      items.some((item) => item.currentCategory === category),
    );
    return ["all", ...categories];
  }, [items]);
  const [session, setSession] = React.useState<DerivedTagTranslationReviewSession>(() =>
    createDerivedTagTranslationReviewSession({
      categoryFilter: initialCategory,
      statusFilter: initialStatus,
      rows: items,
    }),
  );
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const [persistError, setPersistError] = React.useState<string | null>(null);

  const categoryFilter = session.reviewState.categoryFilter;
  const statusFilter = session.reviewState.statusFilter;
  const filteredRows = React.useMemo(
    () =>
      filterTranslationQueueRows({
        rows: session.rows,
        categoryFilter,
        statusFilter,
      }),
    [categoryFilter, session.rows, statusFilter],
  );
  const menuItems = React.useMemo(() => buildTranslationQueueMenuItems(filteredRows), [filteredRows]);
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
    let cancelled = false;
    void writeDerivedTagTranslationReviewSession(rootPath, session)
      .then(() => {
        if (!cancelled) {
          setPersistError(null);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setPersistError((error as Error).message);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [rootPath, session]);

  React.useEffect(() => {
    setSelectedIndex((current) => Math.max(0, Math.min(current, Math.max(0, menuItems.length - 1))));
  }, [menuItems.length]);

  const updateRow = React.useCallback(
    (rowKey: string, nextOverride: DerivedTagTranslationOverride) => {
      setSession((current) => ({
        ...current,
        rows: current.rows.map((row) =>
          row.key === rowKey
            ? {
                ...row,
                draftOverride: cloneDerivedTagTranslationOverride(nextOverride),
              }
            : row,
        ),
        reviewState: {
          ...current.reviewState,
          updatedAt: new Date().toISOString(),
        },
      }));
    },
    [],
  );

  const requestFieldEdit = React.useCallback(
    async (item: TranslationQueueMenuItem) => {
      const fieldResult = await prompts.promptSelectOption<TranslationEditableFieldId>({
        title: "Edit Translation Field",
        prompt: "Field",
        entries: [
          { value: "targetProjectionId", label: "Target projection" },
          { value: "renameNote", label: "Rename note" },
          { value: "notes", label: "Editorial notes" },
        ],
      });
      if (fieldResult.kind !== "selected") {
        return;
      }
      const field = fieldResult.value;
      const metadata = fieldMetadata(field);
      const currentValue =
        field === "targetProjectionId"
          ? String(item.row.draftOverride.targetProjectionId ?? item.row.base.targetProjectionId ?? "")
          : String(item.row.draftOverride[field] ?? "");
      const input = await prompts.promptTextInput({
        title: metadata.title,
        prompt: metadata.prompt,
        defaultValue: currentValue,
        hint: metadata.hint,
        presentation: "overlay",
      });
      if (input === undefined) {
        return;
      }
      const nextValue = input.trim();
      updateRow(item.row.key, setOverrideField(item.row.draftOverride, field, nextValue));
    },
    [prompts, updateRow],
  );

  const requestStatusEdit = React.useCallback(
    async (item: TranslationQueueMenuItem) => {
      const result = await prompts.promptSelectOption<DerivedTagTranslationRecord["translationStatus"]>({
        title: "Set Translation Status",
        prompt: "Status",
        entries: [
          { value: "mapped", label: "mapped" },
          { value: "provisional", label: "provisional" },
          { value: "unmapped", label: "unmapped" },
          { value: "dropped", label: "dropped" },
        ],
      });
      if (result.kind !== "selected") {
        return;
      }
      const nextOverride = cloneDerivedTagTranslationOverride(item.row.draftOverride);
      nextOverride.translationStatus = result.value;
      updateRow(item.row.key, nextOverride);
    },
    [prompts, updateRow],
  );

  const handleImport = React.useCallback(async () => {
    try {
      await importDerivedTagTranslationReviewSession(rootPath, session);
      setSession((current) => ({
        ...current,
        reviewState: {
          ...current.reviewState,
          imported: true,
          updatedAt: new Date().toISOString(),
        },
      }));
      await terminal.pauseForAnyKey(`Imported translation session ${session.manifest.id}.`);
      onBack();
    } catch (error) {
      await terminal.pauseForAnyKey(`Import failed: ${(error as Error).message}`);
    }
  }, [onBack, rootPath, session, terminal]);

  return (
    <TerminalActionMenuScreen
      title="Ontology Translation Review"
      subtitle={`${filteredRows.length} session row${filteredRows.length === 1 ? "" : "s"} shown`}
      leftTitle="Translation Rows"
      rightTitle="Translation Detail"
      leftWidth={60}
      items={menuItems}
      selectedIndex={selectedIndex}
      interactions={interactions}
      actionEntries={actionEntries}
      buildRightLines={(selectedItem) =>
        buildTranslationDetailLines({
          selectedItem,
          totalCount: session.rows.length,
          visibleCount: filteredRows.length,
          categoryFilter,
          statusFilter,
          persistError,
        })
      }
      buildStatusLine={({ selectedItem }) => ({
        text: selectedItem
          ? `Selected: ${selectedItem.row.base.currentCategory} / ${selectedItem.row.base.currentTag}`
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
        const selectedItem = menuItems[selectedIndex];
        if (actionId === "cycle_category") {
          setSession((current) => ({
            ...current,
            reviewState: {
              ...current.reviewState,
              categoryFilter: cycleValue(categoryFilterValues, current.reviewState.categoryFilter),
              updatedAt: new Date().toISOString(),
            },
          }));
          return;
        }
        if (actionId === "cycle_status") {
          setSession((current) => ({
            ...current,
            reviewState: {
              ...current.reviewState,
              statusFilter: cycleValue(
                ["all", "provisional", "unmapped", "mapped"] as const,
                current.reviewState.statusFilter,
              ),
              updatedAt: new Date().toISOString(),
            },
          }));
          return;
        }
        if (!selectedItem) {
          return;
        }
        if (actionId === "set_translation_status") {
          void requestStatusEdit(selectedItem);
          return;
        }
        if (actionId === "edit_field") {
          void requestFieldEdit(selectedItem);
          return;
        }
        if (actionId === "reset_row") {
          updateRow(selectedItem.row.key, selectedItem.row.currentOverride);
          return;
        }
        void handleImport();
      }}
      transitionStatus={transitionStatus}
    />
  );
}
