export interface Broken { field: MissingSourceType }
export interface Callable { callback: () => string }
export const unrelatedFailure: number = 'unrelated';
import * as missing from './missing-provider.js';
const missingCatalog = missing.catalog;
export interface IndirectBroken { field: typeof missingCatalog }
