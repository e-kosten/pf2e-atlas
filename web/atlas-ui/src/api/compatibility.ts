export const API_CONTRACT = __ATLAS_API_CONTRACT__;
export const API_CONTRACT_HEADER = "x-atlas-api-contract";

export class AtlasCompatibilityError extends Error {
  constructor(readonly backendContract: string | null) {
    super(
      "Atlas frontend and backend versions differ. Reload this page. For local development, rebuild and restart the backend from this checkout and check ATLAS_API_PROXY.",
    );
    this.name = "AtlasCompatibilityError";
  }
}

export function checkResponseCompatibility(response: Response): void {
  const actual = response.headers.get(API_CONTRACT_HEADER);
  if (actual !== API_CONTRACT) throw new AtlasCompatibilityError(actual);
}
