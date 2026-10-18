import { AtlasApiError } from "../../api/atlasApi";

/** Client validation/revision errors confirm rejection; a timeout does not. */
export function isDefiniteRollRejection(cause: unknown): cause is AtlasApiError {
  return (
    cause instanceof AtlasApiError &&
    cause.status >= 400 &&
    cause.status < 500 &&
    cause.status !== 408
  );
}

/** Presentation delay only; never delays committing the backend result. */
export function waitForRollReveal(): Promise<void> {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ? Promise.resolve()
    : new Promise((resolve) => setTimeout(resolve, 650));
}
