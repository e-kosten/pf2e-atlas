import React from "react";
import { Box, render as renderInkApp, useApp, useWindowSize } from "ink";
import { useStdout } from "ink";

import { TERMINAL_DIALOG_CONTINUE_FOOTER } from "../interaction-bindings.js";
import {
  isCenteredModalPresentation,
  isCenteredScreenModalPresentation,
} from "../terminal-modal-layout.js";
import { DerivedTagTerminalContext } from "./context.js";
import {
  buildOptionalSelectModalOptions,
  buildSelectModalOptions,
  createEmptyPolicySelection,
  getFirstEnabledCommandIndex,
  getSelectPromptInitialIndex,
} from "./modal-helpers.js";
import { DerivedTagTerminalModalHost } from "./modal-host.js";
import { createValueStateLookup } from "./modal-policy-state.js";
import { planTerminalModalStateLayout } from "./modal-planning.js";
import type {
  CommandPaletteOptions,
  DerivedTagTerminalContextValue,
  DerivedTagTerminalHyperlinkSupport,
  DerivedTagTerminalMultiSelectPromptResult,
  DerivedTagTerminalOptionalSelectPromptResult,
  DerivedTagTerminalPolicySelection,
  DerivedTagTerminalProviderProps,
  DerivedTagTerminalPickerSelectPromptResult,
  MultiSelectPromptOptions,
  OptionalSelectPromptOptions,
  PolicyPromptOptions,
  SelectPromptOptions,
  TerminalModalState,
  TextPromptOptions,
} from "./types.js";

function detectHyperlinkSupport(stdout: NodeJS.WriteStream): DerivedTagTerminalHyperlinkSupport {
  const env = process.env;

  if (env.FORCE_HYPERLINK === "1") {
    return "supported";
  }

  if (
    env.FORCE_HYPERLINK === "0" ||
    env.CI ||
    !stdout.isTTY ||
    env.TERM === "dumb" ||
    env.TERM_PROGRAM === "Apple_Terminal"
  ) {
    return "unsupported";
  }

  if (
    env.WT_SESSION ||
    env.KITTY_WINDOW_ID ||
    env.WEZTERM_EXECUTABLE ||
    env.DOMTERM ||
    env.TERMINAL_EMULATOR === "JetBrains-JediTerm" ||
    env.VTE_VERSION ||
    env.TERM_PROGRAM === "iTerm.app" ||
    env.TERM_PROGRAM === "WezTerm"
  ) {
    return "supported";
  }

  return "unsupported";
}

export function DerivedTagTerminalProvider({
  children,
  hyperlinkSupport,
}: DerivedTagTerminalProviderProps): React.JSX.Element {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const { columns, rows } = useWindowSize();
  const [modal, setModal] = React.useState<TerminalModalState>(null);
  const capabilities = React.useMemo(
    () => ({
      hyperlinkSupport: hyperlinkSupport ?? detectHyperlinkSupport(stdout),
    }),
    [hyperlinkSupport, stdout],
  );
  const modalLayout = React.useMemo(() => planTerminalModalStateLayout(modal, columns, rows), [columns, modal, rows]);
  const availableRows =
    modalLayout?.presentation === "inline"
      ? Math.max(0, rows - modalLayout.totalHeight)
      : modalLayout?.presentation && isCenteredModalPresentation(modalLayout.presentation)
        ? isCenteredScreenModalPresentation(modalLayout.presentation)
          ? 0
          : rows
        : modalLayout
          ? 0
          : rows;
  const shouldRenderChildren = !modalLayout?.presentation || !isCenteredScreenModalPresentation(modalLayout.presentation);

  const contextValue = React.useMemo<DerivedTagTerminalContextValue>(
    () => ({
      capabilities,
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
            filterText: "",
            filterMode: false,
            resolve: resolve as (
              value:
                | DerivedTagTerminalPickerSelectPromptResult<unknown>
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
            filterText: "",
            filterMode: false,
            valueStates,
            resolve: resolve as (value: DerivedTagTerminalPolicySelection<string>) => void,
          });
        }),
      promptMultiSelectOption: async <T extends string>(options: MultiSelectPromptOptions<T>) =>
        new Promise<DerivedTagTerminalMultiSelectPromptResult<T>>((resolve) => {
          const selectedIndex = Math.max(
            0,
            options.entries.findIndex((entry) => options.selectedValues?.includes(entry.value)),
          );
          setModal({
            kind: "multiselect",
            options: options as MultiSelectPromptOptions<string>,
            selectedIndex,
            filterText: "",
            filterMode: false,
            selectedValues: options.selectedValues ? [...options.selectedValues] : [],
            resolve: resolve as (value: DerivedTagTerminalMultiSelectPromptResult<string>) => void,
          });
        }),
      promptSelectOption: async <T,>(options: SelectPromptOptions<T>) =>
        new Promise<DerivedTagTerminalPickerSelectPromptResult<T>>((resolve) => {
          const modalOptions = buildSelectModalOptions(options);
          setModal({
            kind: "select",
            options: modalOptions,
            selectedIndex: getSelectPromptInitialIndex(modalOptions.entries, options.selectedValue),
            filterText: "",
            filterMode: false,
            resolve: resolve as (
              value:
                | DerivedTagTerminalPickerSelectPromptResult<unknown>
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
    [availableRows, capabilities, columns, exit, modal],
  );

  return (
    <DerivedTagTerminalContext.Provider value={contextValue}>
      <Box flexDirection="column" width={columns} height={rows}>
        <Box flexDirection="column" width={columns} height={shouldRenderChildren ? rows : 0}>
          {children}
        </Box>
      </Box>
      {modal && modalLayout?.presentation === "inline" ? (
        <DerivedTagTerminalModalHost
          modal={modal}
          setModal={setModal}
          exitApp={exit}
          width={columns}
          layout={modalLayout}
        />
      ) : null}
      {modal &&
      (modalLayout?.presentation === "screen" ||
        (modalLayout?.presentation && isCenteredModalPresentation(modalLayout.presentation))) ? (
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
