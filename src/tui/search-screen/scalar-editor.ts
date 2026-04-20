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
type ScalarEditorPrompts = Pick<SearchTerminalPromptAdapters, "promptSelectOption" | "promptTextInput">;
type ScalarEditorTerminal = Pick<DerivedTagTerminalApp, "pauseForAnyKey">;

type ParsedScalarInputOptions<T> = {
  title: string;
  prompt: string;
  defaultValue?: string;
  hint?: string;
  parse: (value: string) => T | string;
  whenEmpty: () => T;
};

const NUMERIC_SCALAR_OPERATOR_OPTIONS: Array<{
  value: NumericScalarOperator;
  label: string;
  description: string;
}> = [
  { value: "eq", label: "Equals", description: "Match exactly one value." },
  { value: "neq", label: "Not Equal", description: "Exclude exactly one value." },
  { value: "gte", label: "At Least", description: "Match values greater than or equal to the target." },
  { value: "lte", label: "At Most", description: "Match values less than or equal to the target." },
  { value: "between", label: "Between", description: "Match values inside an inclusive range." },
];

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

  const maxMatch = trimmed.match(/^<=?\s*(\d+)$/);
  if (maxMatch) {
    return { levelMin: null, levelMax: Number.parseInt(maxMatch[1]!, 10) };
  }

  return "Use `3-8`, `5`, `5+`, or `<=10`.";
}

export function formatNumericScalarInput(draft: NumericScalarClauseDraft | null): string {
  if (!draft) {
    return "";
  }
  return draft.op === "between" ? `${draft.min}-${draft.max}` : String(draft.value);
}

export async function promptNumericScalarClause(
  prompts: ScalarEditorPrompts,
  terminal: ScalarEditorTerminal,
  options: {
    title: string;
    currentClause: NumericScalarClauseDraft | null;
  },
): Promise<NumericScalarClauseDraft | null | undefined> {
  const opResult = await prompts.promptSelectOption({
    title: options.title,
    prompt: "Choose the numeric comparison for this clause",
    entries: NUMERIC_SCALAR_OPERATOR_OPTIONS,
    selectedValue: options.currentClause?.op ?? "eq",
  });

  if (opResult.kind !== "selected") {
    return undefined;
  }

  return promptParsedScalarInput(prompts, terminal, {
    title: `${options.title} Value`,
    prompt:
      opResult.value === "between"
        ? "Enter an inclusive range such as `3-8`. Leave blank to clear."
        : "Enter a numeric value. Leave blank to clear.",
    defaultValue: formatNumericScalarInput(options.currentClause),
    parse: (value) => {
      if (opResult.value === "between") {
        const parsed = parseNumericRangeValue(value);
        return typeof parsed === "string" ? parsed : { op: "between", ...parsed };
      }
      const parsed = parseSingleNumericValue(value);
      return typeof parsed === "string" ? parsed : { op: opResult.value, value: parsed };
    },
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
    prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
    defaultValue: options.defaultValue,
    hint: "Examples: 3-8 or <=5",
    parse: parseLevelRangeInput,
    whenEmpty: () => ({ levelMin: null, levelMax: null }),
  });
}
