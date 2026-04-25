import type { TerminalModalPresentation } from "../terminal-modal-layout.js";

export type DerivedTagTerminalPromptPresentation = TerminalModalPresentation | "overlay" | "blanked";
export type TerminalCenteredPromptBackgroundTreatment = "overlay" | "blanked";
export type NormalizedTerminalPromptPresentation = "inline" | "screen" | TerminalCenteredPromptBackgroundTreatment;

export function normalizeTerminalPromptPresentation(
  presentation: DerivedTagTerminalPromptPresentation | undefined,
): NormalizedTerminalPromptPresentation | undefined {
  switch (presentation) {
    case "overlay":
      return "overlay";
    case "blanked":
      return "blanked";
    default:
      return presentation;
  }
}

export function isCenteredPromptPresentation(
  presentation: DerivedTagTerminalPromptPresentation | NormalizedTerminalPromptPresentation | undefined,
): presentation is TerminalCenteredPromptBackgroundTreatment {
  return presentation === "overlay" || presentation === "blanked";
}

export function isBlankedPromptPresentation(
  presentation: DerivedTagTerminalPromptPresentation | NormalizedTerminalPromptPresentation | undefined,
): presentation is "blanked" {
  return presentation === "blanked";
}
