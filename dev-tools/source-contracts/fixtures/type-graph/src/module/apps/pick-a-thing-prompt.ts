import type { Predicate } from '../system/predication.js';

export interface PickableThing<T = string> {
  value: T;
  label: string;
  predicate?: Predicate;
}
