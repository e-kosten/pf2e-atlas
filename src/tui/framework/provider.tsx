import React from "react";
import { Box, render as renderInkApp, useApp, useWindowSize } from "ink";

import { TERMINAL_DIALOG_CONTINUE_FOOTER } from "../interaction-bindings.js";
import { DerivedTagTerminalContext } from "./context.js";
import {
  DerivedTagTerminalModalHost,
  buildOptionalSelectModalOptions,
  buildSelectModalOptions,
  createEmptyPolicySelection,
  createValueStateLookup,
  getFirstEnabledCommandIndex,
  getSelectPromptInitialIndex,
  planTerminalModalStateLayout,
} from "./modal.js";
import type {
  CommandPaletteOptions,
  DerivedTagTerminalContextValue,
  DerivedTagTerminalOptionalSelectPromptResult,
  DerivedTagTerminalPolicySelection,
  DerivedTagTerminalSelectPromptResult,
  MultiSelectPromptOptions,
  OptionalSelectPromptOptions,
  PolicyPromptOptions,
  SelectPromptOptions,
  TerminalModalState,
  TextPromptOptions,
} from "./types.js";

export function DerivedTagTerminalProvider({ children }: { children: React.ReactNode }): React.JSX.Element {
  const { exit } = useApp();
  const { columns, rows } = useWindowSize();
  const [modal, setModal] = React.useState<TerminalModalState>(null);
  const modalLayout = React.useMemo(() => planTerminalModalStateLayout(modal, columns, rows), [columns, modal, rows]);
  const availableRows =
    modalLayout?.presentation === "inline" ? Math.max(0, rows - modalLayout.totalHeight) : modalLayout ? 0 : rows;

  const contextValue = React.useMemo<DerivedTagTerminalContextValue>(
    () => ({
      exitApp: exit,
      getTerminalHeight: () => availableRows,
      getTerminalWidth: () => columns,
      modalActive: modal !== null,
      pauseForAnyKey: async (message: string) => {
        await new Promise<void>((resolve) => {
          setModal({
            kind: "dialog",
            options: {
              title: "Derived-Tag Workbench",
              body: message.split("\n").map((line) => ({ text: line })),
              footer: [{ text: TERMINAL_DIALOG_CONTINUE_FOOTER, tone: "dim" }],
            },
            resolve,
          });
        });
      },
      promptCommandPalette: async <T extends string>(options: CommandPaletteOptions<T>) =>
        new Promise<T | undefined>((resolve) => {
          const normalizedOptions = options as CommandPaletteOptions<string>;
          setModal({
            kind: "command",
            options: normalizedOptions,
            filterText: "",
            selectedIndex: getFirstEnabledCommandIndex(normalizedOptions.entries),
            resolve: resolve as (value: string | undefined) => void,
          });
        }),
      promptOptionalSelectOption: async <T,>(options: OptionalSelectPromptOptions<T>) =>
        new Promise<DerivedTagTerminalOptionalSelectPromptResult<T>>((resolve) => {
          const modalOptions = buildOptionalSelectModalOptions(options);
          setModal({
            kind: "select",
            options: modalOptions,
            selectedIndex: getSelectPromptInitialIndex(modalOptions.entries, options.selectedValue),
            resolve: resolve as (
              value:
                | DerivedTagTerminalSelectPromptResult<unknown>
                | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
            ) => void,
          });
        }),
      promptPolicySelectOption: async <T extends string>(options: PolicyPromptOptions<T>) =>
        new Promise<DerivedTagTerminalPolicySelection<T>>((resolve) => {
          const initialSelection = createEmptyPolicySelection<string>();
          initialSelection.any = options.selectedValues?.any ? [...options.selectedValues.any] : [];
          initialSelection.all = options.selectedValues?.all ? [...options.selectedValues.all] : [];
          initialSelection.exclude = options.selectedValues?.exclude ? [...options.selectedValues.exclude] : [];
          const valueStates = createValueStateLookup(initialSelection);
          const selectedIndex = Math.max(
            0,
            options.entries.findIndex((entry) => valueStates[entry.value] !== undefined),
          );
          setModal({
            kind: "policy",
            options: options as PolicyPromptOptions<string>,
            selectedIndex,
            valueStates,
            resolve: resolve as (value: DerivedTagTerminalPolicySelection<string>) => void,
          });
        }),
      promptMultiSelectOption: async <T extends string>(options: MultiSelectPromptOptions<T>) =>
        new Promise<T[]>((resolve) => {
          const selectedIndex = Math.max(
            0,
            options.entries.findIndex((entry) => options.selectedValues?.includes(entry.value)),
          );
          setModal({
            kind: "multiselect",
            options: options as MultiSelectPromptOptions<string>,
            selectedIndex,
            selectedValues: options.selectedValues ? [...options.selectedValues] : [],
            resolve: resolve as (value: string[]) => void,
          });
        }),
      promptSelectOption: async <T,>(options: SelectPromptOptions<T>) =>
        new Promise<DerivedTagTerminalSelectPromptResult<T>>((resolve) => {
          const modalOptions = buildSelectModalOptions(options);
          setModal({
            kind: "select",
            options: modalOptions,
            selectedIndex: getSelectPromptInitialIndex(modalOptions.entries, options.selectedValue),
            resolve: resolve as (
              value:
                | DerivedTagTerminalSelectPromptResult<unknown>
                | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
            ) => void,
          });
        }),
      promptTextInput: async (options: TextPromptOptions) =>
        new Promise<string | undefined>((resolve) => {
          setModal({
            kind: "text",
            options,
            value: options.defaultValue ?? "",
            resolve,
          });
        }),
      showDialog: async (options) =>
        new Promise<void>((resolve) => {
          setModal({
            kind: "dialog",
            options,
            resolve,
          });
        }),
    }),
    [availableRows, columns, exit, modal],
  );

  return (
    <DerivedTagTerminalContext.Provider value={contextValue}>
      {modal && modalLayout?.presentation === "screen" ? (
        <DerivedTagTerminalModalHost
          modal={modal}
          setModal={setModal}
          exitApp={exit}
          width={columns}
          layout={modalLayout}
        />
      ) : null}
      <Box flexDirection="column">{children}</Box>
      {modal && modalLayout?.presentation === "inline" ? (
        <DerivedTagTerminalModalHost
          modal={modal}
          setModal={setModal}
          exitApp={exit}
          width={columns}
          layout={modalLayout}
        />
      ) : null}
    </DerivedTagTerminalContext.Provider>
  );
}

export async function runDerivedTagTerminalApp(node: React.ReactElement): Promise<void> {
  const instance = renderInkApp(<DerivedTagTerminalProvider>{node}</DerivedTagTerminalProvider>, {
    alternateScreen: true,
    exitOnCtrlC: false,
    patchConsole: true,
  });

  try {
    await instance.waitUntilExit();
  } finally {
    instance.cleanup();
  }
}
