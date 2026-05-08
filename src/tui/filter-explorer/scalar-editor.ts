import type { DerivedTagTerminalApp } from "../framework/types.js";
import type { SearchTerminalPromptAdapters } from "../interaction-context-adapters.js";
import type { SearchNumericMatch } from "../../domain/search-request-types.js";
import {
  SEARCH_FILTER_OPERATOR_VOCABULARY,
  type NumericOperator,
} from "../../domain/search-filter-operators.js";

export type NumericScalarOperator = NumericOperator;

export type NumericScalarClauseDraft =
  | {
      op: "between";
      min: number;
      max: number;
    }
  | {
      op: Exclude<NumericScalarOperator, "between">;
      value: number;
    };

export type LevelRangeDraft = SearchNumericMatch | null;

type ScalarTextPrompts = Pick<SearchTerminalPromptAdapters, "promptTextInput">;
type ScalarEditorPrompts = Pick<SearchTerminalPromptAdapters, "promptTextInput">;
type ScalarEditorTerminal = Pick<DerivedTagTerminalApp, "pauseForAnyKey">;

type ParsedScalarInputOptions<T> = {
  title: string;
  prompt: string;
  defaultValue?: string;
  hint?: string;
  previewTitle?: string;
  buildPreviewLines?: (currentValue: string) => import("../framework/types.js").DerivedTagTerminalLine[];
  parse: (value: string) => T | string;
  whenEmpty: () => T;
};

async function promptParsedScalarInput<T>(
  prompts: ScalarTextPrompts,
  terminal: ScalarEditorTerminal,
  options: ParsedScalarInputOptions<T>,
): Promise<T | undefined> {
  const input = await prompts.promptTextInput({
    title: options.title,
    prompt: options.prompt,
    defaultValue: options.defaultValue,
    hint: options.hint,
    previewTitle: options.previewTitle,
    buildPreviewLines: options.buildPreviewLines,
    presentation: "overlay",
  });

  if (input === undefined) {
    return undefined;
  }

  const trimmed = input.trim();
  if (!trimmed) {
    return options.whenEmpty();
  }

  const parsed = options.parse(trimmed);
  if (typeof parsed === "string") {
    await terminal.pauseForAnyKey(parsed);
    return undefined;
  }
  return parsed;
}

function parseSingleNumericValue(value: string): number | string {
  const numericValue = Number.parseFloat(value);
  return Number.isFinite(numericValue) ? numericValue : "Enter a valid number.";
}

function parseNumericRangeValue(value: string): { min: number; max: number } | string {
  const match = value.match(/^(-?\d+(?:\.\d+)?)\s*-\s*(-?\d+(?:\.\d+)?)$/);
  if (!match) {
    return "Use a numeric range such as `3-8`.";
  }

  return {
    min: Number.parseFloat(match[1]!),
    max: Number.parseFloat(match[2]!),
  };
}

function parseNumericScalarClauseInput(value: string): NumericScalarClauseDraft | string {
  const trimmed = value.trim();
  const between = parseNumericRangeValue(trimmed);
  if (typeof between !== "string") {
    return { op: "between", ...between };
  }

  const prefixedMatch = trimmed.match(/^(=|!=|>=|>|<=|<)\s*(-?\d+(?:\.\d+)?)$/);
  if (prefixedMatch) {
    const parsedValue = Number.parseFloat(prefixedMatch[2]!);
    if (!Number.isFinite(parsedValue)) {
      return "Enter a valid number.";
    }

    const op =
      prefixedMatch[1] === "="
        ? "eq"
        : prefixedMatch[1] === "!="
          ? "notEq"
          : prefixedMatch[1] === ">"
            ? "gt"
            : prefixedMatch[1] === ">="
              ? "gte"
              : prefixedMatch[1] === "<"
                ? "lt"
                : "lte";
    return { op, value: parsedValue };
  }

  const minimumMatch = trimmed.match(/^(-?\d+(?:\.\d+)?)\+$/);
  if (minimumMatch) {
    return { op: "gte", value: Number.parseFloat(minimumMatch[1]!) };
  }

  const parsedValue = parseSingleNumericValue(trimmed);
  if (typeof parsedValue === "string") {
    return "Use `5`, `!=5`, `>5`, `>=5`, `<5`, `<=5`, or `3-8`.";
  }
  return { op: "eq", value: parsedValue };
}

export function parseLevelRangeInput(value: string): LevelRangeDraft | string {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  const betweenMatch = trimmed.match(/^(\d+)\s*-\s*(\d+)$/);
  if (betweenMatch) {
    const min = Number.parseInt(betweenMatch[1]!, 10);
    const max = Number.parseInt(betweenMatch[2]!, 10);
    if (min === max) {
    return { kind: "eq", value: min };
  }
    return { kind: "between", min, max };
  }

  if (/^\d+$/.test(trimmed)) {
    return { kind: "eq", value: Number.parseInt(trimmed, 10) };
  }

  const minMatch = trimmed.match(/^(\d+)\+$/);
  if (minMatch) {
    return { kind: "gte", value: Number.parseInt(minMatch[1]!, 10) };
  }

  const orderedMatch = trimmed.match(/^(>=|>|<=|<)\s*(\d+)$/);
  if (orderedMatch) {
    const value = Number.parseInt(orderedMatch[2]!, 10);
    if (orderedMatch[1] === ">") {
      return { kind: "gt", value };
    }
    if (orderedMatch[1] === ">=") {
      return { kind: "gte", value };
    }
    if (orderedMatch[1] === "<") {
      return { kind: "lt", value };
    }
    return { kind: "lte", value };
  }

  return "Use `3-8`, `5`, `>5`, `>=5`, `5+`, `<10`, or `<=10`.";
}

export function formatNumericScalarInput(draft: NumericScalarClauseDraft | null): string {
  if (!draft) {
    return "";
  }
  if (draft.op === "between") {
    return `${draft.min}-${draft.max}`;
  }
  if (draft.op === "eq") {
    return String(draft.value);
  }
  if (draft.op === SEARCH_FILTER_OPERATOR_VOCABULARY.EQUALITY.NOT_EQ) {
    return `!=${draft.value}`;
  }
  if (draft.op === "gt") {
    return `>${draft.value}`;
  }
  if (draft.op === "gte") {
    return `>=${draft.value}`;
  }
  if (draft.op === "lt") {
    return `<${draft.value}`;
  }
  return `<=${draft.value}`;
}

function buildNumericScalarPreviewLines(currentValue: string): import("../framework/types.js").DerivedTagTerminalLine[] {
  const trimmed = currentValue.trim();
  if (!trimmed) {
    return [{ text: "Clears this numeric matcher.", tone: "dim" }];
  }

  const parsed = parseNumericScalarClauseInput(trimmed);
  if (typeof parsed === "string") {
    return [{ text: parsed, tone: "warning" }];
  }

  if (parsed.op === "between") {
    return [{ text: `Matches values from ${parsed.min} through ${parsed.max}.`, tone: "accent" }];
  }

  const operatorText =
    parsed.op === "eq"
      ? `exactly ${parsed.value}`
      : parsed.op === SEARCH_FILTER_OPERATOR_VOCABULARY.EQUALITY.NOT_EQ
        ? `anything except ${parsed.value}`
        : parsed.op === "gt"
          ? `greater than ${parsed.value}`
        : parsed.op === "gte"
          ? `${parsed.value} or greater`
          : parsed.op === "lt"
            ? `less than ${parsed.value}`
          : `${parsed.value} or lower`;
  return [{ text: `Matches ${operatorText}.`, tone: "accent" }];
}

function buildLevelPreviewLines(currentValue: string): import("../framework/types.js").DerivedTagTerminalLine[] {
  const trimmed = currentValue.trim();
  if (!trimmed) {
    return [{ text: "Clears the level matcher.", tone: "dim" }];
  }

  const parsed = parseLevelRangeInput(trimmed);
  if (typeof parsed === "string") {
    return [{ text: parsed, tone: "warning" }];
  }
  if (parsed === null) {
    return [{ text: "Clears the level matcher.", tone: "dim" }];
  }

  switch (parsed.kind) {
    case "eq":
      return [{ text: `Matches only level ${parsed.value}.`, tone: "accent" }];
    case "gt":
      return [{ text: `Matches levels above ${parsed.value}.`, tone: "accent" }];
    case "gte":
      return [{ text: `Matches level ${parsed.value} and above.`, tone: "accent" }];
    case "lt":
      return [{ text: `Matches levels below ${parsed.value}.`, tone: "accent" }];
    case "lte":
      return [{ text: `Matches level ${parsed.value} and below.`, tone: "accent" }];
    case "between":
      return [{ text: `Matches levels ${parsed.min} through ${parsed.max}.`, tone: "accent" }];
  }
}

export async function promptNumericScalarClause(
  prompts: ScalarEditorPrompts,
  terminal: ScalarEditorTerminal,
  options: {
    title: string;
    currentClause: NumericScalarClauseDraft | null;
  },
): Promise<NumericScalarClauseDraft | null | undefined> {
  return promptParsedScalarInput(prompts, terminal, {
    title: options.title,
    prompt: "Enter `5`, `!=5`, `>5`, `>=5`, `<5`, `<=5`, or `3-8`. Leave blank to clear.",
    defaultValue: formatNumericScalarInput(options.currentClause),
    hint: "Examples: 5, !=5, >5, >=5, <5, <=5, 3-8",
    previewTitle: "Preview",
    buildPreviewLines: buildNumericScalarPreviewLines,
    parse: parseNumericScalarClauseInput,
    whenEmpty: () => null,
  });
}

export async function promptLevelRangeDraft(
  prompts: ScalarTextPrompts,
  terminal: ScalarEditorTerminal,
  options: {
    defaultValue?: string;
  },
): Promise<LevelRangeDraft | undefined> {
  return promptParsedScalarInput(prompts, terminal, {
    title: "Level Range",
    prompt: "Enter `3-8`, `5`, `>5`, `>=5`, `5+`, `<10`, or `<=10`. Leave blank to clear.",
    defaultValue: options.defaultValue,
    hint: "Examples: 3-8, >5, >=5, <10, <=10",
    previewTitle: "Preview",
    buildPreviewLines: buildLevelPreviewLines,
    parse: parseLevelRangeInput,
    whenEmpty: () => null,
  });
}
