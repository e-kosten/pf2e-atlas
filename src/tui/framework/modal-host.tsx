import React from "react";
import { useInput } from "ink";

import { TERMINAL_DIALOG_CONTINUE_FOOTER } from "../interaction-bindings.js";
import {
  createTerminalCommandPaletteInteractionContext,
  createTerminalInteractionContextRouterState,
  createTerminalMultiSelectPromptInteractionContext,
  createTerminalPolicyPromptInteractionContext,
  createTerminalSelectPromptInteractionContext,
  createTerminalTextPromptInteractionContext,
  routeTerminalInteractionContext,
} from "../interaction-context-router.js";
import { createDerivedTagTerminalInputEvent, moveSelectionWrapped } from "./input.js";
import {
  clampPromptSelectionIndex,
  createEmptyPolicySelection,
  filterCommandPaletteEntries,
  filterPromptEntries,
  getFirstEnabledCommandIndex,
  getFilteredPromptSelectionIndex,
  getMultiSelectPromptFilteringEnabled,
  getPolicyPromptFilteringEnabled,
} from "./modal-helpers.js";
import {
  buildPolicySelection,
  cyclePolicyState,
} from "./modal-policy-state.js";
import {
  CommandPaletteBody,
  InlinePromptMessageBody,
  MultiSelectPromptBody,
  PolicyPromptBody,
  SelectPromptBody,
  TextPromptBody,
} from "./modal-prompt-bodies.js";
import { TerminalCenteredOverlayPanel, TerminalInlinePromptPanel } from "./screen-components.js";
import {
  isCenteredModalPresentation,
  type TerminalModalLayoutResult,
} from "../terminal-modal-layout.js";
import type { TerminalModalState } from "./types.js";

function getChoicePromptFilteringEnabled(
  modal: Exclude<TerminalModalState, null | { kind: "dialog" } | { kind: "text" } | { kind: "command" }>,
): boolean {
  if (modal.kind === "select") {
    return modal.options.filtering;
  }
  return modal.kind === "multiselect"
    ? getMultiSelectPromptFilteringEnabled(modal.options)
    : getPolicyPromptFilteringEnabled(modal.options);
}

function getFilteredChoiceEntries(
  modal: Exclude<TerminalModalState, null | { kind: "dialog" } | { kind: "text" } | { kind: "command" }>,
) {
  switch (modal.kind) {
    case "select":
      return !modal.options.filtering || modal.options.choiceLayout === "horizontal"
        ? modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }))
        : filterPromptEntries(modal.options.entries, modal.filterText);
    case "multiselect":
      return getMultiSelectPromptFilteringEnabled(modal.options)
        ? filterPromptEntries(modal.options.entries, modal.filterText)
        : modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));
    case "policy":
      return getPolicyPromptFilteringEnabled(modal.options)
        ? filterPromptEntries(modal.options.entries, modal.filterText)
        : modal.options.entries.map((entry, originalIndex) => ({ entry, originalIndex }));
  }
}

function getChoicePromptSelectionIndex(
  modal: Exclude<TerminalModalState, null | { kind: "dialog" } | { kind: "text" } | { kind: "command" }>,
  filterText = modal.filterText,
): number {
  switch (modal.kind) {
    case "select":
      return getFilteredPromptSelectionIndex(modal.options.entries, modal.selectedIndex, filterText);
    case "multiselect":
      return getFilteredPromptSelectionIndex(modal.options.entries, modal.selectedIndex, filterText);
    case "policy":
      return getFilteredPromptSelectionIndex(modal.options.entries, modal.selectedIndex, filterText);
  }
}

export function DerivedTagTerminalModalHost({
  modal,
  setModal,
  exitApp,
  width,
  layout,
}: {
  modal: TerminalModalState;
  setModal: React.Dispatch<React.SetStateAction<TerminalModalState>>;
  exitApp: () => void;
  width: number;
  layout: TerminalModalLayoutResult;
}): React.JSX.Element | null {
  const routerStateRef = React.useRef(
    createTerminalInteractionContextRouterState<
      "commandPalette" | "multiSelectPrompt" | "policyPrompt" | "selectPrompt" | "textPrompt"
    >(),
  );
  const resolveAfterModalClose = React.useCallback(<T,>(resolver: (value: T) => void, value: T): void => {
    routerStateRef.current = createTerminalInteractionContextRouterState();
    resolver(value);
  }, []);
  const pageSize = Math.max(1, layout.visibleListCapacity || 10);

  useInput(
    (input, key) => {
      const event = createDerivedTagTerminalInputEvent(input, key);
      if (event.systemAction === "interrupt") {
        exitApp();
        return;
      }

      if (!modal) {
        routerStateRef.current = createTerminalInteractionContextRouterState();
        return;
      }

      if (modal.kind === "dialog") {
        const resolver = modal.resolve;
        setModal(null);
        resolver();
        return;
      }

      if (modal.kind === "text") {
        const routed = routeTerminalInteractionContext(
          event,
          createTerminalTextPromptInteractionContext(),
          routerStateRef.current,
        );
        routerStateRef.current = routed.state;

        if (routed.route.textEntryIntent?.kind === "submit") {
          const resolver = modal.resolve;
          const trimmed = modal.value.trim();
          setModal(null);
          resolver(trimmed ? trimmed : undefined);
          return;
        }
        if (routed.route.textEntryIntent?.kind === "cancel") {
          const resolver = modal.resolve;
          setModal(null);
          resolver(undefined);
          return;
        }
        if (routed.route.textEntryIntent?.kind === "deleteBackward") {
          setModal((current) =>
            current?.kind === "text" ? { ...current, value: [...current.value].slice(0, -1).join("") } : current,
          );
          return;
        }
        if (routed.route.textEntryIntent?.kind === "append") {
          const appendText = routed.route.textEntryIntent.text;
          setModal((current) =>
            current?.kind === "text" ? { ...current, value: current.value + appendText } : current,
          );
        }
        return;
      }

      if (modal.kind === "command") {
        const routed = routeTerminalInteractionContext(
          event,
          createTerminalCommandPaletteInteractionContext(pageSize),
          routerStateRef.current,
        );
        routerStateRef.current = routed.state;
        const filteredEntries = filterCommandPaletteEntries(modal.options.entries, modal.filterText);
        const clampedSelectedIndex = clampPromptSelectionIndex(modal.selectedIndex, filteredEntries.length);

        if (routed.route.textEntryIntent?.kind === "deleteBackward") {
          if (modal.filterText.length === 0) {
            const resolver = modal.resolve;
            setModal(null);
            resolveAfterModalClose(resolver, undefined);
            return;
          }
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  filterText: [...current.filterText].slice(0, -1).join(""),
                  selectedIndex: getFirstEnabledCommandIndex(
                    filterCommandPaletteEntries(current.options.entries, [...current.filterText].slice(0, -1).join("")),
                  ),
                }
              : current,
          );
          return;
        }
        if (routed.route.textEntryIntent?.kind === "append") {
          const appendText = routed.route.textEntryIntent.text;
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  filterText: current.filterText + appendText,
                  selectedIndex: getFirstEnabledCommandIndex(
                    filterCommandPaletteEntries(current.options.entries, current.filterText + appendText),
                  ),
                }
              : current,
          );
          return;
        }
        if (routed.route.navigationAction?.kind === "move") {
          const delta = routed.route.navigationAction.delta;
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  selectedIndex: moveSelectionWrapped(clampedSelectedIndex, delta, filteredEntries.length),
                }
              : current,
          );
          return;
        }
        if (routed.route.navigationAction?.kind === "boundary") {
          const boundary = routed.route.navigationAction.boundary;
          setModal((current) =>
            current?.kind === "command"
              ? {
                  ...current,
                  selectedIndex: boundary === "start" ? 0 : Math.max(0, filteredEntries.length - 1),
                }
              : current,
          );
          return;
        }
        if (routed.route.interactionAction?.id === "select") {
          const selectedEntry = filteredEntries[clampedSelectedIndex];
          if (selectedEntry?.disabled) {
            return;
          }
          const resolver = modal.resolve;
          const selected = selectedEntry?.value;
          setModal(null);
          resolveAfterModalClose(resolver, selected);
          return;
        }
        if (routed.route.interactionAction?.id === "back" || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolveAfterModalClose(resolver, undefined);
        }
        return;
      }

      const filteredEntries = getFilteredChoiceEntries(modal);
      const filteredSelectedIndex = getChoicePromptSelectionIndex(modal);

      if (modal.filterMode) {
        if (event.isConfirmKey()) {
          setModal((current) =>
            current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
              ? { ...current, filterMode: false }
              : current,
          );
          return;
        }
        if (event.textInputAction === "cancel" || event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          setModal((current) =>
            current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
              ? { ...current, filterMode: false, filterText: "" }
              : current,
          );
          return;
        }
        if (event.textInputAction === "deleteBackward") {
          setModal((current) =>
            current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
              ? {
                  ...current,
                  filterText: [...current.filterText].slice(0, -1).join(""),
                  selectedIndex: getChoicePromptSelectionIndex(
                    current,
                    [...current.filterText].slice(0, -1).join(""),
                  ),
                }
              : current,
          );
          return;
        }
        if (event.printable) {
          setModal((current) =>
            current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
              ? {
                  ...current,
                  filterText: current.filterText + event.printable,
                  selectedIndex: getChoicePromptSelectionIndex(current, current.filterText + event.printable),
                }
              : current,
          );
        }
        return;
      }

      if (getChoicePromptFilteringEnabled(modal) && event.isSearchKey()) {
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? { ...current, filterMode: true }
            : current,
        );
        return;
      }

      if (filteredEntries.length === 0) {
        if (event.textInputAction === "cancel" || event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          setModal(null);
          if (modal.kind === "select") {
            resolveAfterModalClose(modal.resolve, event.isBackNavigationKey() ? { kind: "back" } : { kind: "cancelled" });
            return;
          }
          if (modal.kind === "multiselect") {
            resolveAfterModalClose(modal.resolve, event.isBackNavigationKey() ? { kind: "back" } : { kind: "cancelled" });
            return;
          }
          resolveAfterModalClose(modal.resolve, createEmptyPolicySelection());
        }
        return;
      }

      if (modal.kind === "select" && modal.options.choiceLayout === "horizontal") {
        const selectedVisibleIndex = Math.max(
          0,
          filteredEntries.findIndex((entry) => entry.originalIndex === filteredSelectedIndex),
        );
        if (event.isMoveLeftKey() || event.isMoveUpKey()) {
          setModal((current) =>
            current?.kind === "select"
              ? {
                  ...current,
                  selectedIndex:
                    filteredEntries[
                      moveSelectionWrapped(selectedVisibleIndex, -1, Math.max(1, filteredEntries.length))
                    ]?.originalIndex ?? current.selectedIndex,
                }
              : current,
          );
          return;
        }
        if (event.isMoveRightKey() || event.isMoveDownKey()) {
          setModal((current) =>
            current?.kind === "select"
              ? {
                  ...current,
                  selectedIndex:
                    filteredEntries[
                      moveSelectionWrapped(selectedVisibleIndex, 1, Math.max(1, filteredEntries.length))
                    ]?.originalIndex ?? current.selectedIndex,
                }
              : current,
          );
          return;
        }
        if (event.isTerminalBoundaryStartKey()) {
          setModal((current) =>
            current?.kind === "select" ? { ...current, selectedIndex: filteredEntries[0]?.originalIndex ?? 0 } : current,
          );
          return;
        }
        if (event.isTerminalBoundaryEndKey()) {
          setModal((current) =>
            current?.kind === "select"
              ? { ...current, selectedIndex: filteredEntries.at(-1)?.originalIndex ?? current.selectedIndex }
              : current,
          );
          return;
        }
        if (event.isCommandPaletteKey() && modal.options.supportsCommands) {
          const resolver = modal.resolve;
          setModal(null);
          resolveAfterModalClose(resolver, { kind: "commands" });
          return;
        }
        if (event.isConfirmKey()) {
          const resolver = modal.resolve;
          const selected = modal.options.entries[filteredSelectedIndex];
          setModal(null);
          if (!selected) {
            resolveAfterModalClose(resolver, { kind: "cancelled" });
            return;
          }
          resolveAfterModalClose(
            resolver,
            selected.kind === "all" ? { kind: "all" } : { kind: "selected", value: selected.value },
          );
          return;
        }
        if (event.textInputAction === "cancel" || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolveAfterModalClose(resolver, { kind: "cancelled" });
        }
        return;
      }

      const choiceContext =
        modal.kind === "multiselect"
          ? createTerminalMultiSelectPromptInteractionContext(pageSize, modal.options.supportsCommands ?? false)
          : modal.kind === "policy"
            ? createTerminalPolicyPromptInteractionContext(pageSize)
            : createTerminalSelectPromptInteractionContext(pageSize, modal.options.supportsCommands);

      if (modal.kind === "select" && event.isCommandPaletteKey() && modal.options.supportsCommands) {
        const resolver = modal.resolve;
        setModal(null);
        resolveAfterModalClose(resolver, { kind: "commands" });
        return;
      }
      if (modal.kind === "multiselect" && event.isCommandPaletteKey() && modal.options.supportsCommands) {
        const resolver = modal.resolve;
        setModal(null);
        resolveAfterModalClose(resolver, { kind: "commands" });
        return;
      }
      if (modal.kind === "select" && event.isBackNavigationKey()) {
        const resolver = modal.resolve;
        setModal(null);
        resolveAfterModalClose(resolver, { kind: "back" });
        return;
      }

      const routed = routeTerminalInteractionContext(event, choiceContext, routerStateRef.current);
      routerStateRef.current = routed.state;

      if (routed.route.navigationAction?.kind === "move") {
        const delta = routed.route.navigationAction.delta;
        const selectedVisibleIndex = Math.max(
          0,
          filteredEntries.findIndex((entry) => entry.originalIndex === filteredSelectedIndex),
        );
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? {
                ...current,
                selectedIndex:
                  filteredEntries[
                    moveSelectionWrapped(selectedVisibleIndex, delta, Math.max(1, filteredEntries.length))
                  ]?.originalIndex ?? current.selectedIndex,
              }
            : current,
        );
        return;
      }
      if (routed.route.navigationAction?.kind === "boundary") {
        const boundary = routed.route.navigationAction.boundary;
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? {
                ...current,
                selectedIndex:
                  boundary === "start"
                    ? (filteredEntries[0]?.originalIndex ?? 0)
                    : (filteredEntries.at(-1)?.originalIndex ?? current.selectedIndex),
              }
            : current,
        );
        return;
      }
      if (modal.kind === "multiselect" && routed.route.interactionAction?.id === "toggle") {
        const selected = modal.options.entries[filteredSelectedIndex]?.value;
        if (!selected) {
          return;
        }
        setModal((current) =>
          current?.kind === "multiselect"
            ? {
                ...current,
                selectedValues: current.selectedValues.includes(selected)
                  ? current.selectedValues.filter((value) => value !== selected)
                  : [...current.selectedValues, selected],
              }
            : current,
        );
        return;
      }
      if (modal.kind === "select" && routed.route.interactionAction?.id === "select") {
        const resolver = modal.resolve;
        const selected = modal.options.entries[filteredSelectedIndex];
        setModal(null);
        if (!selected) {
          resolveAfterModalClose(resolver, { kind: "cancelled" });
          return;
        }
        resolveAfterModalClose(
          resolver,
          selected.kind === "all" ? { kind: "all" } : { kind: "selected", value: selected.value },
        );
        return;
      }
      if (modal.kind === "select" && routed.route.interactionAction?.id === "commands" && modal.options.supportsCommands) {
        const resolver = modal.resolve;
        setModal(null);
        resolveAfterModalClose(resolver, { kind: "commands" });
        return;
      }
      if (
        modal.kind === "multiselect" &&
        routed.route.interactionAction?.id === "commands" &&
        modal.options.supportsCommands
      ) {
        const resolver = modal.resolve;
        setModal(null);
        resolveAfterModalClose(resolver, { kind: "commands" });
        return;
      }
      if (modal.kind === "multiselect" && routed.route.interactionAction?.id === "return") {
        const resolver = modal.resolve;
        const selectedValues = modal.selectedValues;
        setModal(null);
        resolveAfterModalClose(resolver, { kind: "selected", values: selectedValues });
        return;
      }
      if (modal.kind === "policy" && routed.route.cycleDirection) {
        const selected = modal.options.entries[filteredSelectedIndex]?.value;
        if (!selected) {
          return;
        }
        setModal((current) =>
          current?.kind === "policy"
            ? {
                ...current,
                valueStates: {
                  ...current.valueStates,
                  [selected]: cyclePolicyState(
                    current.valueStates[selected],
                    current.options.allowedStates,
                    routed.route.cycleDirection,
                  ),
                },
              }
            : current,
        );
        return;
      }
      if (modal.kind === "policy" && (routed.route.interactionAction?.id === "return" || event.isTerminalQuitKey())) {
        const resolver = modal.resolve;
        const selection = buildPolicySelection(modal.options.entries, modal.valueStates);
        setModal(null);
        resolveAfterModalClose(resolver, selection);
        return;
      }
      if (modal.kind === "select" && (routed.route.interactionAction?.id === "back" || event.isTerminalQuitKey())) {
        const resolver = modal.resolve;
        setModal(null);
        resolveAfterModalClose(
          resolver,
          routed.route.interactionAction?.id === "back" ? { kind: "back" } : { kind: "cancelled" },
        );
        return;
      }
      if (modal.kind === "multiselect" && (routed.route.interactionAction?.id === "back" || event.isTerminalQuitKey())) {
        const resolver = modal.resolve;
        setModal(null);
        resolveAfterModalClose(
          resolver,
          routed.route.interactionAction?.id === "back" ? { kind: "back" } : { kind: "cancelled" },
        );
      }
    },
    { isActive: modal !== null },
  );

  if (!modal) {
    return null;
  }

  const panelWidth =
    layout.presentation && isCenteredModalPresentation(layout.presentation)
      ? (layout.panelWidth ?? Math.max(24, width - 4))
      : width;
  const renderPanel = (panel: React.JSX.Element): React.JSX.Element =>
    layout.presentation && isCenteredModalPresentation(layout.presentation) ? (
      <TerminalCenteredOverlayPanel width={panelWidth} height={layout.totalHeight}>{panel}</TerminalCenteredOverlayPanel>
    ) : (
      panel
    );

  if (modal.kind === "dialog") {
    return renderPanel(
      <TerminalInlinePromptPanel
        title={modal.options.title}
        subtitle={modal.options.subtitle}
        body={<InlinePromptMessageBody width={panelWidth} height={layout.bodyHeight} lines={modal.options.body} />}
        footer={modal.options.footer ?? [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }]}
        width={panelWidth}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />,
    );
  }
  if (modal.kind === "text") {
    return renderPanel(
      <TextPromptBody options={modal.options} currentValue={modal.value} width={panelWidth} layout={layout} />,
    );
  }
  if (modal.kind === "command") {
    return renderPanel(
      <CommandPaletteBody
        options={modal.options}
        filterText={modal.filterText}
        selectedIndex={modal.selectedIndex}
        width={panelWidth}
        layout={layout}
      />,
    );
  }
  if (modal.kind === "multiselect") {
    return renderPanel(
      <MultiSelectPromptBody
        options={modal.options}
        selectedIndex={modal.selectedIndex}
        filterText={modal.filterText}
        filterMode={modal.filterMode}
        selectedValues={modal.selectedValues}
        width={panelWidth}
        layout={layout}
      />,
    );
  }
  if (modal.kind === "policy") {
    return renderPanel(
      <PolicyPromptBody
        options={modal.options}
        selectedIndex={modal.selectedIndex}
        filterText={modal.filterText}
        filterMode={modal.filterMode}
        valueStates={modal.valueStates}
        width={panelWidth}
        layout={layout}
      />,
    );
  }

  return renderPanel(
    <SelectPromptBody
      options={modal.options}
      selectedIndex={modal.selectedIndex}
      filterText={modal.filterText}
      filterMode={modal.filterMode}
      width={panelWidth}
      layout={layout}
    />,
  );
}
