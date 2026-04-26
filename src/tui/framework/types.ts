import type React from "react";
import type { Key } from "ink";

import type {
  DerivedTagTerminalPromptPresentation,
} from "./prompt-presentation.js";

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
  href?: string;
};

export type DerivedTagTerminalLine = {
  text: string;
  segments?: DerivedTagTerminalSegment[];
  tone?: DerivedTagTerminalTone;
  indent?: number;
  href?: string;
  plainTextFallback?: string;
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

export type DerivedTagTerminalChoiceLayout = "list" | "horizontal";

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
  presentation?: DerivedTagTerminalPromptPresentation;
};

export type DerivedTagTerminalTextInputOptions = {
  title: string;
  prompt: string;
  defaultValue?: string;
  hint?: string;
  previewTitle?: string;
  buildPreviewLines?: (currentValue: string) => DerivedTagTerminalLine[];
  presentation?: DerivedTagTerminalPromptPresentation;
};

export type DerivedTagTerminalPromptBackResult = { kind: "back" };
export type DerivedTagTerminalSelectPromptResult<T = string> =
  | { kind: "cancelled" }
  | DerivedTagTerminalPromptBackResult
  | { kind: "selected"; value: T };
export type DerivedTagTerminalPickerCommandResult = { kind: "commands" };
export type DerivedTagTerminalPickerSelectPromptResult<T = string> =
  | DerivedTagTerminalSelectPromptResult<T>
  | DerivedTagTerminalPickerCommandResult;

export type DerivedTagTerminalOptionalSelectPromptResult<T = string> =
  | { kind: "cancelled" }
  | DerivedTagTerminalPromptBackResult
  | { kind: "all" }
  | { kind: "selected"; value: T };

export type DerivedTagTerminalSelectPromptOptions<T = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T;
  presentation?: DerivedTagTerminalPromptPresentation;
  choiceLayout?: DerivedTagTerminalChoiceLayout;
  filtering?: boolean;
  supportsCommands?: boolean;
};

export type DerivedTagTerminalOptionalSelectPromptOptions<T = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  allOption: Pick<DerivedTagTerminalSelectOption<string>, "label" | "description" | "detailLines">;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValue?: T | null;
  presentation?: DerivedTagTerminalPromptPresentation;
  choiceLayout?: DerivedTagTerminalChoiceLayout;
  filtering?: boolean;
  supportsCommands?: boolean;
};

export type DerivedTagTerminalMultiSelectPromptOptions<T extends string = string> = {
  title: string;
  subtitle?: string;
  prompt: string;
  entries: DerivedTagTerminalSelectOption<T>[];
  selectedValues?: T[];
  presentation?: DerivedTagTerminalPromptPresentation;
  filtering?: boolean;
  supportsCommands?: boolean;
};
export type DerivedTagTerminalMultiSelectPromptResult<T extends string = string> =
  | { kind: "cancelled" }
  | DerivedTagTerminalPromptBackResult
  | { kind: "selected"; values: T[] }
  | DerivedTagTerminalPickerCommandResult;

export type TextPromptOptions = DerivedTagTerminalTextInputOptions;
export type SelectPromptOptions<T = string> = DerivedTagTerminalSelectPromptOptions<T>;
export type OptionalSelectPromptOptions<T = string> = DerivedTagTerminalOptionalSelectPromptOptions<T>;
export type MultiSelectPromptOptions<T extends string = string> = DerivedTagTerminalMultiSelectPromptOptions<T>;

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
  presentation?: DerivedTagTerminalPromptPresentation;
  choiceLayout: DerivedTagTerminalChoiceLayout;
  filtering: boolean;
  supportsCommands: boolean;
};

export type DerivedTagTerminalPromptSession = Pick<
  DerivedTagTerminalContextValue,
  | "pauseForAnyKey"
  | "promptOptionalSelectOption"
  | "promptMultiSelectOption"
  | "promptSelectOption"
  | "promptTextInput"
  | "showDialog"
>;

export type TerminalModalOwnership = {
  leaseId: number;
  ownerKind: "shared" | "session";
  sessionId: number | null;
};

export type TerminalModalState =
  | null
  | {
      kind: "dialog";
      ownership: TerminalModalOwnership;
      options: DialogOptions;
      resolve: () => void;
    }
  | {
      kind: "text";
      ownership: TerminalModalOwnership;
      options: TextPromptOptions;
      value: string;
      resolve: (value: string | undefined) => void;
    }
  | {
      kind: "select";
      ownership: TerminalModalOwnership;
      options: TerminalSelectModalOptions;
      selectedIndex: number;
      filterText: string;
      filterMode: boolean;
      resolve: (
        value:
          | DerivedTagTerminalPickerSelectPromptResult<unknown>
          | DerivedTagTerminalOptionalSelectPromptResult<unknown>,
      ) => void;
    }
  | {
      kind: "multiselect";
      ownership: TerminalModalOwnership;
      options: MultiSelectPromptOptions<string>;
      selectedIndex: number;
      filterText: string;
      filterMode: boolean;
      selectedValues: string[];
      resolve: (value: DerivedTagTerminalMultiSelectPromptResult<string>) => void;
    };

export type DerivedTagTerminalContextValue = {
  capabilities: DerivedTagTerminalCapabilities;
  backdropActive: boolean;
  exitApp: (result?: unknown) => void;
  getLayoutHeight: () => number;
  getLayoutWidth: () => number;
  getViewportHeight: () => number;
  getViewportWidth: () => number;
  modalActive: boolean;
  pauseForAnyKey: (message: string) => Promise<void>;
  runPromptSession: <T>(runner: (session: DerivedTagTerminalPromptSession) => Promise<T>) => Promise<T>;
  promptOptionalSelectOption: <T>(
    options: OptionalSelectPromptOptions<T>,
  ) => Promise<DerivedTagTerminalOptionalSelectPromptResult<T>>;
  promptMultiSelectOption: <T extends string>(
    options: MultiSelectPromptOptions<T>,
  ) => Promise<DerivedTagTerminalMultiSelectPromptResult<T>>;
  promptSelectOption: <T>(options: SelectPromptOptions<T>) => Promise<DerivedTagTerminalPickerSelectPromptResult<T>>;
  promptTextInput: (options: TextPromptOptions) => Promise<string | undefined>;
  showDialog: (options: DialogOptions) => Promise<void>;
};

export type DerivedTagTerminalApp = DerivedTagTerminalContextValue;

export type DerivedTagTerminalHyperlinkSupport = "supported" | "unsupported";

export type DerivedTagTerminalCapabilities = {
  hyperlinkSupport: DerivedTagTerminalHyperlinkSupport;
};

export type DerivedTagTerminalProviderProps = {
  children: React.ReactNode;
  hyperlinkSupport?: DerivedTagTerminalHyperlinkSupport;
};
