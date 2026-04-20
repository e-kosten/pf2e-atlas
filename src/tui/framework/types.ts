import type React from "react";
import type { Key } from "ink";

import type { TerminalModalPresentation } from "../terminal-modal-layout.js";

export type DerivedTagTerminalTone =
  | "default"
  | "heading"
  | "section"
  | "dim"
  | "accent"
  | "success"
  | "warning"
  | "danger"
  | "selected";

export type DerivedTagTerminalSegment = {
  text: string;
  tone?: DerivedTagTerminalTone;
};

export type DerivedTagTerminalLine = {
  text: string;
  segments?: DerivedTagTerminalSegment[];
  tone?: DerivedTagTerminalTone;
  indent?: number;
  noWrap?: boolean;
};

export type DerivedTagTerminalPane = {
  title: string;
  lines: DerivedTagTerminalLine[];
  active?: boolean;
};

export type DerivedTagTerminalSelectOption<T = string> = {
  value: T;
  label: string;
  description?: string;
  detailLines?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalCommandOption<T extends string = string> = DerivedTagTerminalSelectOption<T> & {
  aliases?: string[];
  disabled?: boolean;
  disabledReason?: string;
  keywords?: string[];
};

export type DerivedTagTerminalTwoPaneFocus = "list" | "detail";
export type DerivedTagTerminalTwoPaneLayoutMode = "split" | "detail-only";
export type DerivedTagTerminalTextInputAction = "submit" | "cancel" | "deleteBackward";
export type DerivedTagTerminalSystemAction = "interrupt";

export type DerivedTagTerminalInputEvent = {
  input: string;
  key: Key;
  printable?: string;
  systemAction?: DerivedTagTerminalSystemAction;
  textInputAction?: DerivedTagTerminalTextInputAction;
  isBackNavigationKey: () => boolean;
  isCommandPaletteKey: () => boolean;
  isConfirmKey: () => boolean;
  isConfirmOrToggleKey: () => boolean;
  isExactPrintableKey: (expected: string) => boolean;
  isExecuteKey: () => boolean;
  isFocusToggleKey: () => boolean;
  isHelpKey: () => boolean;
  isLayoutToggleKey: () => boolean;
  isMoveDownKey: () => boolean;
  isMoveLeftKey: () => boolean;
  isMoveRightKey: () => boolean;
  isMoveUpKey: () => boolean;
  isPageDownKey: () => boolean;
  isPageUpKey: () => boolean;
  isSearchKey: () => boolean;
  isTerminalBoundaryEndKey: () => boolean;
  isTerminalBoundaryStartKey: () => boolean;
  isTerminalJumpBackwardKey: () => boolean;
  isTerminalJumpForwardKey: () => boolean;
  isTerminalQuitKey: () => boolean;
  getCycleDirection: () => 1 | -1 | undefined;
};

export type DerivedTagTerminalTextScreenProps = {
  title: string;
  subtitle?: string;
  body: DerivedTagTerminalLine[];
  footer?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalPaneScreenProps = {
  title: string;
  subtitle?: string;
  pane: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
};

export type DerivedTagTerminalTwoPaneScreenProps = {
  title: string;
  subtitle?: string;
  left: DerivedTagTerminalPane;
  right: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
  leftWidth?: number;
};

export type DerivedTagTerminalThreePaneScreenProps = {
  title: string;
  subtitle?: string;
  left: DerivedTagTerminalPane;
  center: DerivedTagTerminalPane;
  right: DerivedTagTerminalPane;
  footer?: DerivedTagTerminalLine[];
  leftWidth?: number;
  centerWidth?: number;
};

export type DerivedTagTerminalInlinePromptPanelProps = {
  title: string;
  subtitle?: string;
  body: React.ReactNode;
  footer?: DerivedTagTerminalLine[];
  width: number;
  height: number;
  showTopBorder?: boolean;
};

export type DialogOptions = {
  title: string;
  subtitle?: string;
  body: DerivedTagTerminalLine[];
  footer?: DerivedTagTerminalLine[];
  presentation?: TerminalModalPresentation;
};

export type DerivedTagTerminalTextInputOptions = {
  title: string;
  prompt: string;
  defaultValue?: string;
  hint?: string;
  presentation?: TerminalModalPresentation;
};

export type DerivedTagTerminalSelectPromptResult<T = string> = { kind: "cancelled" } | { kind: "selected"; value: T };

export type DerivedTagTerminalOptionalSelectPromptResult<T = string> =
  | { kind: "cancelled" }
  | { kind: "all" }
  | { kind: "selected"; value: T };

export type DerivedTagTerminalSelectPromptOptions<T = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T;
  presentation?: TerminalModalPresentation;
};

export type DerivedTagTerminalOptionalSelectPromptOptions<T = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  allOption: Pick<DerivedTagTerminalSelectOption<string>, "label" | "description" | "detailLines">;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T | null;
  presentation?: TerminalModalPresentation;
};

export type DerivedTagTerminalMultiSelectPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValues?: T[];
  presentation?: TerminalModalPresentation;
};

export type DerivedTagTerminalPolicyState = "any" | "all" | "exclude";

export type DerivedTagTerminalPolicySelection<T extends string = string> = {
  any: T[];
  all: T[];
  exclude: T[];
};

export type DerivedTagTerminalPolicyPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  allowedStates: DerivedTagTerminalPolicyState[];
  selectedValues?: Partial<DerivedTagTerminalPolicySelection<T>>;
  presentation?: TerminalModalPresentation;
};

export type CommandPaletteOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalCommandOption<T>[];
  presentation?: TerminalModalPresentation;
};

export type TextPromptOptions = DerivedTagTerminalTextInputOptions;
export type SelectPromptOptions<T = string> = DerivedTagTerminalSelectPromptOptions<T>;
export type OptionalSelectPromptOptions<T = string> = DerivedTagTerminalOptionalSelectPromptOptions<T>;
export type MultiSelectPromptOptions<T extends string = string> = DerivedTagTerminalMultiSelectPromptOptions<T>;
export type PolicyPromptOptions<T extends string = string> = DerivedTagTerminalPolicyPromptOptions<T>;

export type TerminalSelectOptionDetails = Pick<
  DerivedTagTerminalSelectOption<unknown>,
  "label" | "description" | "detailLines"
>;

export type TerminalSelectModalEntry =
  | (TerminalSelectOptionDetails & {
      kind: "selected";
      value: unknown;
    })
  | (TerminalSelectOptionDetails & {
      kind: "all";
    });

export type TerminalSelectModalOptions = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: TerminalSelectModalEntry[];
  presentation?: TerminalModalPresentation;
};

export type TerminalModalState =
  | null
  | {
      kind: "dialog";
      options: DialogOptions;
      resolve: () => void;
    }
  | {
      kind: "text";
      options: TextPromptOptions;
      value: string;
      resolve: (value: string | undefined) => void;
    }
  | {
      kind: "select";
      options: TerminalSelectModalOptions;
      selectedIndex: number;
      resolve: (
        value: DerivedTagTerminalSelectPromptResult<unknown> | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
      ) => void;
    }
  | {
      kind: "multiselect";
      options: MultiSelectPromptOptions<string>;
      selectedIndex: number;
      selectedValues: string[];
      resolve: (value: string[]) => void;
    }
  | {
      kind: "policy";
      options: PolicyPromptOptions<string>;
      selectedIndex: number;
      valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>;
      resolve: (value: DerivedTagTerminalPolicySelection<string>) => void;
    }
  | {
      kind: "command";
      options: CommandPaletteOptions<string>;
      filterText: string;
      selectedIndex: number;
      resolve: (value: string | undefined) => void;
    };

export type DerivedTagTerminalContextValue = {
  exitApp: (result?: unknown) => void;
  getTerminalHeight: () => number;
  getTerminalWidth: () => number;
  modalActive: boolean;
  pauseForAnyKey: (message: string) => Promise<void>;
  promptCommandPalette: <T extends string>(options: CommandPaletteOptions<T>) => Promise<T | undefined>;
  promptOptionalSelectOption: <T>(
    options: OptionalSelectPromptOptions<T>,
  ) => Promise<DerivedTagTerminalOptionalSelectPromptResult<T>>;
  promptPolicySelectOption: <T extends string>(
    options: PolicyPromptOptions<T>,
  ) => Promise<DerivedTagTerminalPolicySelection<T>>;
  promptMultiSelectOption: <T extends string>(options: MultiSelectPromptOptions<T>) => Promise<T[]>;
  promptSelectOption: <T>(options: SelectPromptOptions<T>) => Promise<DerivedTagTerminalSelectPromptResult<T>>;
  promptTextInput: (options: TextPromptOptions) => Promise<string | undefined>;
  showDialog: (options: DialogOptions) => Promise<void>;
};

export type DerivedTagTerminalApp = DerivedTagTerminalContextValue;
