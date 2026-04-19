import type { MetadataFilterNode, OntologyDomainModel } from "../types.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "./search-service.js";
import type { DerivedTagTerminalLine, DerivedTagTerminalTwoPaneScreenProps } from "./terminal-ui.js";

export type SearchQueryFieldPickerSession = {
  model: OntologyDomainModel;
  initialSelections: OntologyPickerSelectionMap;
  applySelection: (selection: OntologyPickerSelectionMap) => void;
};

export type SearchQueryFieldBuilderStep = "fieldList" | "ontologyPicker";

export type SearchQueryFieldBuilderItem =
  | {
      kind: "field";
      fieldOption: Pf2eTerminalQueryFieldOption;
      label: string;
    }
  | {
      kind: "finish" | "cancel";
      label: string;
    };

export type SearchQueryFieldBuilderDraft = {
  query: Pf2eTerminalSearchQuery;
  stagedSelections: Pf2eTerminalQueryFieldSelectionMap;
  payload: Record<string, unknown> | null;
};

export type SearchQueryFieldBuilderOutcome =
  | {
      kind: "finish";
      selectedField: Pf2eTerminalQueryFieldOption | null;
      draft: SearchQueryFieldBuilderDraft | null;
    }
  | {
      kind: "cancel";
      selectedField: Pf2eTerminalQueryFieldOption | null;
      draft: SearchQueryFieldBuilderDraft | null;
    };

export type SearchQueryFieldBuilderSession = {
  items: SearchQueryFieldBuilderItem[];
  selectedIndex: number;
  fieldDrafts: Record<string, MetadataFilterNode | null>;
  moveSelection: (delta: number, itemCount: number) => void;
  selectCurrent: () => void;
  finish: () => void;
  cancel: () => void;
  kind?: "queryFieldBuilder";
  title?: string;
  availableFields?: Pf2eTerminalQueryFieldOption[];
  selectedFieldIndex?: number;
  step?: SearchQueryFieldBuilderStep;
  activeChildView?: "none" | "ontologyPicker";
  childSession?: SearchQueryFieldPickerSession | null;
  draft?: SearchQueryFieldBuilderDraft | null;
  onFinish?: (outcome: Extract<SearchQueryFieldBuilderOutcome, { kind: "finish" }>) => void;
  onCancel?: (outcome: Extract<SearchQueryFieldBuilderOutcome, { kind: "cancel" }>) => void;
};

function countConfiguredFields(fieldDrafts: Record<string, MetadataFilterNode | null>): number {
  return Object.values(fieldDrafts).filter((node) => Boolean(node)).length;
}

function getAvailableFields(session: SearchQueryFieldBuilderSession): Pf2eTerminalQueryFieldOption[] {
  return (
    session.availableFields ??
    session.items.flatMap((item) => (item.kind === "field" ? [item.fieldOption] : []))
  );
}

function getSelectedField(session: SearchQueryFieldBuilderSession): Pf2eTerminalQueryFieldOption | null {
  const availableFields = getAvailableFields(session);
  if (availableFields.length === 0) {
    return null;
  }

  const selectedIndex = session.selectedFieldIndex ?? session.selectedIndex;
  const clampedIndex = Math.max(0, Math.min(selectedIndex, availableFields.length - 1));
  return availableFields[clampedIndex] ?? null;
}

function buildDraftSummaryLines(node: MetadataFilterNode | null): DerivedTagTerminalLine[] {
  if (!node) {
    return [{ text: "No staged draft for this field yet.", tone: "dim" }];
  }

  if ("and" in node) {
    return [
      { text: "Staged draft", tone: "section" },
      { text: `${node.and.length} staged clause${node.and.length === 1 ? "" : "s"} in this field.` },
    ];
  }
  if ("or" in node) {
    return [
      { text: "Staged draft", tone: "section" },
      { text: `${node.or.length} OR clause${node.or.length === 1 ? "" : "s"} in this field.` },
    ];
  }
  if ("not" in node) {
    return [
      { text: "Staged draft", tone: "section" },
      { text: "1 negated staged clause in this field." },
    ];
  }

  return [
    { text: "Staged draft", tone: "section" },
    { text: `${node.field} ${node.op}`, tone: "accent" },
  ];
}

export function buildSearchQueryFieldBuilderDetailLines(
  session: SearchQueryFieldBuilderSession,
): DerivedTagTerminalLine[] {
  const selectedItem = session.items[Math.max(0, Math.min(session.selectedIndex, session.items.length - 1))];
  if (!selectedItem) {
    return [{ text: "No query-field builder item is selected.", tone: "dim" }];
  }

  if (selectedItem.kind === "field") {
    const lines: DerivedTagTerminalLine[] = [
      { text: selectedItem.fieldOption.label, tone: "section" },
      { text: selectedItem.fieldOption.description || "No additional field description is available." },
      { text: `Editor: ${selectedItem.fieldOption.editor}`, tone: "accent" },
      { text: "" },
      ...buildDraftSummaryLines(session.fieldDrafts[selectedItem.fieldOption.value] ?? null),
      { text: "" },
      { text: `Step: ${session.step ?? "fieldList"}` },
      { text: `Child view: ${session.activeChildView ?? "none"}` },
      { text: "Selections remain staged in the builder until you finish.", tone: "dim" },
    ];

    if (session.draft?.payload) {
      const payloadKeys = Object.keys(session.draft.payload);
      lines.push({
        text:
          payloadKeys.length > 0
            ? `Payload keys: ${payloadKeys.join(", ")}`
            : "Payload holder is present for session-local collector state.",
        tone: "dim",
      });
    }

    return lines;
  }

  if (selectedItem.kind === "finish") {
    const configuredFields = countConfiguredFields(session.fieldDrafts);
    return [
      { text: "Finish Clause Builder", tone: "section" },
      { text: `${configuredFields} field${configuredFields === 1 ? "" : "s"} currently staged.` },
      { text: "Apply the staged builder draft to the live query and return to the editor.", tone: "accent" },
    ];
  }

  return [
    { text: "Cancel Clause Builder", tone: "section" },
    { text: "Discard the staged builder draft and return to the editor.", tone: "warning" },
  ];
}

export function buildSearchQueryFieldBuilderStatusLine(
  session: SearchQueryFieldBuilderSession,
): DerivedTagTerminalLine {
  const configuredFields = countConfiguredFields(session.fieldDrafts);
  return {
    text: `${configuredFields} staged field${configuredFields === 1 ? "" : "s"} | live query unchanged until finish`,
    tone: "accent",
  };
}

export function buildSearchQueryFieldBuilderScreen(
  session: SearchQueryFieldBuilderSession,
): DerivedTagTerminalTwoPaneScreenProps {
  const configuredFields = countConfiguredFields(session.fieldDrafts);
  const selectedField = getSelectedField(session);
  return {
    title: session.title ?? "Query Field Builder",
    subtitle: `${getAvailableFields(session).length} available query fields | ${configuredFields} staged field${configuredFields === 1 ? "" : "s"}`,
    left: {
      title: "[QUERY FIELDS]",
      lines: session.items.map((item, index) => ({
        text:
          item.kind === "field" && session.fieldDrafts[item.fieldOption.value]
            ? `${item.label} | staged`
            : item.label,
        tone: index === session.selectedIndex ? "selected" : item.kind === "cancel" ? "warning" : "default",
        noWrap: true,
      })),
      active: true,
    },
    right: {
      title: selectedField ? `Builder Detail | ${selectedField.label}` : "Builder Detail",
      lines: buildSearchQueryFieldBuilderDetailLines(session),
      active: false,
    },
    footer: [
      {
        text: "↑/↓ select  Ctrl-U/D jump  b/f page  gg/G edge  Enter/→/Space open  Esc cancel builder",
        tone: "dim",
      },
      {
        text:
          session.step === "ontologyPicker"
            ? "Child ontology picker is active inside the builder session"
            : "Live query unchanged until you finish the clause builder.",
        tone: "accent",
      },
    ],
    leftWidth: 40,
  };
}
