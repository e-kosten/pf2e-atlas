export const API_CONTRACT = __ATLAS_API_CONTRACT__;
export const API_CONTRACT_HEADER = "x-atlas-api-contract";

export class AtlasCompatibilityError extends Error {
  constructor(readonly backendContract: string | null) {
    super(
      "This page and Atlas are running different versions. Reload the page to continue.",
    );
    this.name = "AtlasCompatibilityError";
  }
}

let failure: AtlasCompatibilityError | null = null;
const listeners = new Set<() => void>();
export function getCompatibilityFailure() {
  return failure;
}
export function subscribeCompatibilityFailures(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
export function clearCompatibilityFailure() {
  failure = null;
  for (const listener of listeners) listener();
}

export function checkResponseCompatibility(response: Response): void {
  const actual = response.headers.get(API_CONTRACT_HEADER);
  if (actual !== API_CONTRACT) {
    failure = new AtlasCompatibilityError(actual);
    for (const listener of listeners) listener();
    throw failure;
  }
}
