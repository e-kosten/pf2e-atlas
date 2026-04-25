import type { DerivedTagTerminalApp } from "../framework/types.js";
import type { SearchTerminalPromptAdapters } from "../interaction-context-adapters.js";

export type NumericScalarOperator = "eq" | "neq" | "gte" | "lte" | "between";

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

export type LevelRangeDraft = {
  levelMin: number | null;
  levelMax: number | null;
};

type ScalarTextPrompts = Pick<SearchTerminalPromptAdapters, "promptTextInput">;
type ScalarEditorPrompts = Pick<SearchTerminalPromptAdapters, "promptTextInput">;
type ScalarEditorTerminal = Pick<DerivedTagTerminalApp, "pauseForAnyKey">;

type ParsedScalarInputOptions<T> = {
  title: string;
  prompt: string;
  defaultValue?: string;
  hint?: string;
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
    presentation: "centered",
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

  const prefixedMatch = trimmed.match(/^(=|!=|>=|<=)\s*(-?\d+(?:\.\d+)?)$/);
  if (prefixedMatch) {
    const parsedValue = Number.parseFloat(prefixedMatch[2]!);
    if (!Number.isFinite(parsedValue)) {
      return "Enter a valid number.";
    }

    const op = prefixedMatch[1] === "=" ? "eq" : prefixedMatch[1] === "!=" ? "neq" : prefixedMatch[1] === ">=" ? "gte" : "lte";
    return { op, value: parsedValue };
  }

  const minimumMatch = trimmed.match(/^(-?\d+(?:\.\d+)?)\+$/);
  if (minimumMatch) {
    return { op: "gte", value: Number.parseFloat(minimumMatch[1]!) };
  }

  const parsedValue = parseSingleNumericValue(trimmed);
  if (typeof parsedValue === "string") {
    return "Use `5`, `!=5`, `>=5`, `<=5`, or `3-8`.";
  }
  return { op: "eq", value: parsedValue };
}

export function parseLevelRangeInput(value: string): LevelRangeDraft | string {
  const trimmed = value.trim();
  if (!trimmed) {
    return { levelMin: null, levelMax: null };
  }

  const betweenMatch = trimmed.match(/^(\d+)\s*-\s*(\d+)$/);
  if (betweenMatch) {
    return {
      levelMin: Number.parseInt(betweenMatch[1]!, 10),
      levelMax: Number.parseInt(betweenMatch[2]!, 10),
    };
  }

  if (/^\d+$/.test(trimmed)) {
    const level = Number.parseInt(trimmed, 10);
    return { levelMin: level, levelMax: level };
  }

  const minMatch = trimmed.match(/^(\d+)\+$/);
  if (minMatch) {
    return { levelMin: Number.parseInt(minMatch[1]!, 10), levelMax: null };
  }

  const minOrGreaterMatch = trimmed.match(/^>=?\s*(\d+)$/);
  if (minOrGreaterMatch) {
    return { levelMin: Number.parseInt(minOrGreaterMatch[1]!, 10), levelMax: null };
  }

  const maxMatch = trimmed.match(/^<=\s*(\d+)$/);
  if (maxMatch) {
    return { levelMin: null, levelMax: Number.parseInt(maxMatch[1]!, 10) };
  }

  return "Use `3-8`, `5`, `>=5`, `5+`, or `<=10`.";
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
  if (draft.op === "neq") {
    return `!=${draft.value}`;
  }
  if (draft.op === "gte") {
    return `>=${draft.value}`;
  }
  return `<=${draft.value}`;
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
    prompt: "Enter `5`, `!=5`, `>=5`, `<=5`, or `3-8`. Leave blank to clear.",
    defaultValue: formatNumericScalarInput(options.currentClause),
    hint: "Examples: 5, !=5, >=5, <=5, 3-8",
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
    prompt: "Enter `3-8`, `5`, `>=5`, `5+`, or `<=10`. Leave blank to clear.",
    defaultValue: options.defaultValue,
    hint: "Examples: 3-8, >=5, <=5",
    parse: parseLevelRangeInput,
    whenEmpty: () => ({ levelMin: null, levelMax: null }),
  });
}
