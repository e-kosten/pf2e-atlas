/** Presentation delay only; never delays committing the backend result. */
export function waitForRollReveal(): Promise<void> {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ? Promise.resolve()
    : new Promise((resolve) => setTimeout(resolve, 650));
}
