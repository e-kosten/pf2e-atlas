import React from "react";

import { TERMINAL_DIALOG_RETURN_FOOTER, type TerminalInteractionTone } from "./interaction-bindings.js";
import {
  useDerivedTagTerminalApp,
  type CommandPaletteOptions,
  type DerivedTagTerminalApp,
  type DerivedTagTerminalMultiSelectPromptOptions as MultiSelectPromptOptions,
  type DerivedTagTerminalOptionalSelectPromptOptions as OptionalSelectPromptOptions,
  type DerivedTagTerminalOptionalSelectPromptResult,
  type DerivedTagTerminalPolicyPromptOptions as PolicyPromptOptions,
  type DerivedTagTerminalPolicySelection,
  type DerivedTagTerminalSelectPromptOptions as SelectPromptOptions,
  type DerivedTagTerminalSelectPromptResult,
  type DerivedTagTerminalTextInputOptions as TextPromptOptions,
  type DialogOptions,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";

export type TerminalInteractionContextAdapters = Pick<
  DerivedTagTerminalApp,
  | "promptCommandPalette"
  | "promptOptionalSelectOption"
  | "promptPolicySelectOption"
  | "promptMultiSelectOption"
  | "promptSelectOption"
  | "promptTextInput"
  | "showDialog"
>;

export type SearchTerminalPromptAdapters = Pick<
  TerminalInteractionContextAdapters,
  | "promptCommandPalette"
  | "promptMultiSelectOption"
  | "promptOptionalSelectOption"
  | "promptPolicySelectOption"
  | "promptSelectOption"
  | "promptTextInput"
  | "showDialog"
>;

export function createTerminalInteractionContextAdapters(
  terminal: TerminalInteractionContextAdapters,
): TerminalInteractionContextAdapters {
  return {
    promptCommandPalette: async <T extends string>(options: CommandPaletteOptions<T>) => terminal.promptCommandPalette(options),
    promptOptionalSelectOption: async <T,>(
      options: OptionalSelectPromptOptions<T>,
    ): Promise<DerivedTagTerminalOptionalSelectPromptResult<T>> => terminal.promptOptionalSelectOption(options),
    promptPolicySelectOption: async <T extends string>(
      options: PolicyPromptOptions<T>,
    ): Promise<DerivedTagTerminalPolicySelection<T>> => terminal.promptPolicySelectOption(options),
    promptMultiSelectOption: async <T extends string>(options: MultiSelectPromptOptions<T>): Promise<T[]> =>
      terminal.promptMultiSelectOption(options),
    promptSelectOption: async <T>(options: SelectPromptOptions<T>): Promise<DerivedTagTerminalSelectPromptResult<T>> =>
      terminal.promptSelectOption(options),
    promptTextInput: async (options: TextPromptOptions): Promise<string | undefined> => terminal.promptTextInput(options),
    showDialog: async (options: DialogOptions): Promise<void> => terminal.showDialog(options),
  };
}

export function useTerminalInteractionContextAdapters(): TerminalInteractionContextAdapters {
  const terminal = useDerivedTagTerminalApp();
  return React.useMemo(() => createTerminalInteractionContextAdapters(terminal), [terminal]);
}

export async function showTerminalReturnDialog(
  adapters: Pick<TerminalInteractionContextAdapters, "showDialog">,
  title: string,
  body: DerivedTagTerminalLine[],
  tone: TerminalInteractionTone = "dim",
): Promise<void> {
  await adapters.showDialog({
    title,
    body,
    footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone }],
  });
}
