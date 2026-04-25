import type { TerminalModalPresentation } from "../terminal-modal-layout.js";

export type DerivedTagTerminalPromptPresentation = TerminalModalPresentation | "overlay" | "blanked";
export type TerminalCenteredPromptBackgroundTreatment = "overlay" | "blanked";
export type NormalizedTerminalPromptPresentation = "inline" | "screen" | TerminalCenteredPromptBackgroundTreatment;

export function normalizeTerminalPromptPresentation(
  presentation: DerivedTagTerminalPromptPresentation | undefined,
): NormalizedTerminalPromptPresentation | undefined {
  switch (presentation) {
    case "centered":
    case "overlay":
      return "overlay";
    case "centered-screen":
    case "blanked":
      return "blanked";
    default:
      return presentation;
  }
}

export function isCenteredPromptPresentation(
  presentation: DerivedTagTerminalPromptPresentation | NormalizedTerminalPromptPresentation | undefined,
): presentation is TerminalCenteredPromptBackgroundTreatment {
  return presentation === "overlay" || presentation === "blanked" || presentation === "centered" || presentation === "centered-screen";
}

export function isBlankedPromptPresentation(
  presentation: DerivedTagTerminalPromptPresentation | NormalizedTerminalPromptPresentation | undefined,
): presentation is "blanked" | "centered-screen" {
  return presentation === "blanked" || presentation === "centered-screen";
}

export function toLegacyCenteredModalPresentation(
  backgroundTreatment: TerminalCenteredPromptBackgroundTreatment,
): Extract<TerminalModalPresentation, "centered" | "centered-screen"> {
  return backgroundTreatment === "overlay" ? "centered" : "centered-screen";
}
