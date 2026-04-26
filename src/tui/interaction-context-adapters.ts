import React from "react";

import { TERMINAL_DIALOG_RETURN_FOOTER, type TerminalInteractionTone } from "./interaction-bindings.js";
import { useDerivedTagTerminalApp } from "./framework/context.js";
import {
  type DerivedTagTerminalApp,
  type DerivedTagTerminalMultiSelectPromptResult,
  type DerivedTagTerminalMultiSelectPromptOptions as MultiSelectPromptOptions,
  type DerivedTagTerminalOptionalSelectPromptOptions as OptionalSelectPromptOptions,
  type DerivedTagTerminalOptionalSelectPromptResult,
  type DerivedTagTerminalPickerSelectPromptResult,
  type DerivedTagTerminalSelectPromptOptions as SelectPromptOptions,
  type DerivedTagTerminalTextInputOptions as TextPromptOptions,
  type DialogOptions,
  type DerivedTagTerminalLine,
} from "./framework/types.js";

export type TerminalInteractionContextAdapters = Pick<
  DerivedTagTerminalApp,
  | "promptOptionalSelectOption"
  | "promptMultiSelectOption"
  | "promptSelectOption"
  | "promptTextInput"
  | "showDialog"
>;

export type SearchTerminalPromptAdapters = Pick<
  TerminalInteractionContextAdapters,
  | "promptMultiSelectOption"
  | "promptOptionalSelectOption"
  | "promptSelectOption"
  | "promptTextInput"
  | "showDialog"
>;

export function createTerminalInteractionContextAdapters(
  terminal: TerminalInteractionContextAdapters,
): TerminalInteractionContextAdapters {
  return {
    promptOptionalSelectOption: async <T>(
      options: OptionalSelectPromptOptions<T>,
    ): Promise<DerivedTagTerminalOptionalSelectPromptResult<T>> => terminal.promptOptionalSelectOption(options),
    promptMultiSelectOption: async <T extends string>(
      options: MultiSelectPromptOptions<T>,
    ): Promise<DerivedTagTerminalMultiSelectPromptResult<T>> => terminal.promptMultiSelectOption(options),
    promptSelectOption: async <T>(options: SelectPromptOptions<T>): Promise<DerivedTagTerminalPickerSelectPromptResult<T>> =>
      terminal.promptSelectOption(options),
    promptTextInput: async (options: TextPromptOptions): Promise<string | undefined> =>
      terminal.promptTextInput(options),
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
