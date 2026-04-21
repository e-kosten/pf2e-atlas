import type {
  DerivedTagTerminalCommandOption,
  DerivedTagTerminalLine,
  DerivedTagTerminalPolicySelection,
  SelectPromptOptions,
  OptionalSelectPromptOptions,
  TerminalSelectModalEntry,
  TerminalSelectModalOptions,
  TerminalSelectOptionDetails,
} from "./types.js";

export function createEmptyPolicySelection<T extends string>(): DerivedTagTerminalPolicySelection<T> {
  return {
    any: [],
    all: [],
    exclude: [],
  };
}

export function clampInlinePromptWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (itemCount <= visibleCount) {
    return 0;
  }

  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

export function clampPromptSelectionIndex(selectedIndex: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  return Math.max(0, Math.min(selectedIndex, itemCount - 1));
}

export function buildPromptDetailLines(option: TerminalSelectOptionDetails | undefined): DerivedTagTerminalLine[] {
  if (option?.detailLines?.length) {
    return option.detailLines;
  }
  if (option?.description) {
    return [{ text: option.label, tone: "section" }, { text: option.description }];
  }

  return [
    { text: option?.label ?? "(none)", tone: "section" },
    { text: "No additional details.", tone: "dim" },
  ];
}

export function buildSelectModalOptions<T>(options: SelectPromptOptions<T>): TerminalSelectModalOptions {
  return {
    title: options.title,
    subtitle: options.subtitle,
    prompt: options.prompt,
    entries: options.entries.map((entry) => ({
      kind: "selected" as const,
      value: entry.value,
      label: entry.label,
      description: entry.description,
      detailLines: entry.detailLines,
    })),
    presentation: options.presentation,
  };
}

export function buildOptionalSelectModalOptions<T>(
  options: OptionalSelectPromptOptions<T>,
): TerminalSelectModalOptions {
  return {
    title: options.title,
    subtitle: options.subtitle,
    prompt: options.prompt,
    entries: [
      {
        kind: "all" as const,
        label: options.allOption.label,
        description: options.allOption.description,
        detailLines: options.allOption.detailLines,
      },
      ...options.entries.map((entry) => ({
        kind: "selected" as const,
        value: entry.value,
        label: entry.label,
        description: entry.description,
        detailLines: entry.detailLines,
      })),
    ],
    presentation: options.presentation,
  };
}

export function getSelectPromptInitialIndex(entries: TerminalSelectModalEntry[], selectedValue: unknown): number {
  return Math.max(
    0,
    entries.findIndex((entry) =>
      entry.kind === "all" ? selectedValue === null : Object.is(entry.value, selectedValue),
    ),
  );
}

export function filterCommandPaletteEntries(
  entries: DerivedTagTerminalCommandOption<string>[],
  filterText: string,
): DerivedTagTerminalCommandOption<string>[] {
  const normalizedTerms = filterText
    .toLowerCase()
    .split(/\s+/)
    .map((term) => term.trim())
    .filter((term) => term.length > 0);

  if (normalizedTerms.length === 0) {
    return entries;
  }

  return entries.filter((entry) => {
    const searchableText = [entry.label, entry.description ?? "", ...(entry.aliases ?? []), ...(entry.keywords ?? [])]
      .join(" ")
      .toLowerCase();

    return normalizedTerms.every((term) => searchableText.includes(term));
  });
}

export function getFirstEnabledCommandIndex(entries: DerivedTagTerminalCommandOption<string>[]): number {
  const enabledIndex = entries.findIndex((entry) => !entry.disabled);
  return enabledIndex >= 0 ? enabledIndex : 0;
}

export function buildCommandPaletteDetailLines(
  option: DerivedTagTerminalCommandOption<string> | undefined,
  filterText: string,
): DerivedTagTerminalLine[] {
  const lines = option?.detailLines ?? [
    { text: option?.label ?? "(none)", tone: "section" as const },
    { text: option?.description ?? "No additional details." },
    ...(option?.aliases?.length ? [{ text: `Aliases: ${option.aliases.join(", ")}`, tone: "accent" as const }] : []),
  ];

  return [
    ...lines,
    ...(option?.disabled
      ? [
          {
            text: option.disabledReason
              ? `Unavailable: ${option.disabledReason}`
              : "This command is currently unavailable.",
            tone: "warning" as const,
          },
        ]
      : []),
    { text: "" },
    { text: `Filter: ${filterText || "(none)"}`, tone: "accent" as const },
  ];
}
