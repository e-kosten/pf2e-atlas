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
  getFirstEnabledCommandIndex,
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
import { TerminalInlinePromptPanel } from "./screen-components.js";
import type { TerminalModalLayoutResult } from "../terminal-modal-layout.js";
import type { TerminalModalState } from "./types.js";

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
            resolver(undefined);
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
          resolver(selected);
          return;
        }
        if (routed.route.interactionAction?.id === "back" || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver(undefined);
        }
        return;
      }

      if (modal.kind === "select" && modal.options.entries.length === 0) {
        if (event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver({ kind: "cancelled" });
        }
        return;
      }

      if (modal.kind === "multiselect" && modal.options.entries.length === 0) {
        if (event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver([]);
        }
        return;
      }

      if (modal.kind === "policy" && modal.options.entries.length === 0) {
        if (event.isBackNavigationKey() || event.isTerminalQuitKey()) {
          const resolver = modal.resolve;
          setModal(null);
          resolver(createEmptyPolicySelection());
        }
        return;
      }

      const choiceContext =
        modal.kind === "multiselect"
          ? createTerminalMultiSelectPromptInteractionContext(pageSize)
          : modal.kind === "policy"
            ? createTerminalPolicyPromptInteractionContext(pageSize)
            : createTerminalSelectPromptInteractionContext(pageSize);
      const routed = routeTerminalInteractionContext(event, choiceContext, routerStateRef.current);
      routerStateRef.current = routed.state;

      if (routed.route.navigationAction?.kind === "move") {
        const delta = routed.route.navigationAction.delta;
        setModal((current) =>
          current && (current.kind === "select" || current.kind === "multiselect" || current.kind === "policy")
            ? {
                ...current,
                selectedIndex: moveSelectionWrapped(current.selectedIndex, delta, current.options.entries.length),
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
                selectedIndex: boundary === "start" ? 0 : Math.max(0, current.options.entries.length - 1),
              }
            : current,
        );
        return;
      }
      if (modal.kind === "multiselect" && routed.route.interactionAction?.id === "toggle") {
        const selected = modal.options.entries[modal.selectedIndex]?.value;
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
        const selected = modal.options.entries[modal.selectedIndex];
        setModal(null);
        if (!selected) {
          resolver({ kind: "cancelled" });
          return;
        }
        resolver(selected.kind === "all" ? { kind: "all" } : { kind: "selected", value: selected.value });
        return;
      }
      if (modal.kind === "multiselect" && routed.route.interactionAction?.id === "return") {
        const resolver = modal.resolve;
        const selectedValues = modal.selectedValues;
        setModal(null);
        resolver(selectedValues);
        return;
      }
      if (modal.kind === "policy" && routed.route.cycleDirection) {
        const selected = modal.options.entries[modal.selectedIndex]?.value;
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
        resolver(selection);
        return;
      }
      if (modal.kind === "select" && (routed.route.interactionAction?.id === "back" || event.isTerminalQuitKey())) {
        const resolver = modal.resolve;
        setModal(null);
        resolver({ kind: "cancelled" });
      }
    },
    { isActive: modal !== null },
  );

  if (!modal) {
    return null;
  }

  if (modal.kind === "dialog") {
    return (
      <TerminalInlinePromptPanel
        title={modal.options.title}
        subtitle={modal.options.subtitle}
        body={<InlinePromptMessageBody width={width} height={layout.bodyHeight} lines={modal.options.body} />}
        footer={modal.options.footer ?? [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }]}
        width={width}
        height={layout.totalHeight}
        showTopBorder={layout.showTopBorder}
      />
    );
  }
  if (modal.kind === "text") {
    return <TextPromptBody options={modal.options} currentValue={modal.value} width={width} layout={layout} />;
  }
  if (modal.kind === "command") {
    return (
      <CommandPaletteBody
        options={modal.options}
        filterText={modal.filterText}
        selectedIndex={modal.selectedIndex}
        width={width}
        layout={layout}
      />
    );
  }
  if (modal.kind === "multiselect") {
    return (
      <MultiSelectPromptBody
        options={modal.options}
        selectedIndex={modal.selectedIndex}
        selectedValues={modal.selectedValues}
        width={width}
        layout={layout}
      />
    );
  }
  if (modal.kind === "policy") {
    return (
      <PolicyPromptBody
        options={modal.options}
        selectedIndex={modal.selectedIndex}
        valueStates={modal.valueStates}
        width={width}
        layout={layout}
      />
    );
  }

  return <SelectPromptBody options={modal.options} selectedIndex={modal.selectedIndex} width={width} layout={layout} />;
}
