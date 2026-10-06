import type { Common, NumericValue, Nested } from '@shared/common.js';
import type * as A from '@shared/left.js';
import type * as B from '@shared/right.js';

export interface Equipment extends Common {
  type: 'equipment';
  contents: Nested<NumericValue>[];
  equipped: { carryType: 'held' | 'worn'; invested: boolean | null };
}
export interface Backpack extends Common {
  type: 'backpack';
  subitems?: never;
  contents: Nested<NumericValue>[];
}
export type Family = Equipment | Backpack;
export type Pair = readonly [NumericValue, string?];
export type Formula = { [key in 'a' | 'b']?: NumericValue };
export type NumericFormula = { [key in 1 | 2]?: NumericValue };
export interface ExplicitEmpty {}
export interface Shared<T> { nested: { value: T } }
export interface Instantiations { first: Shared<string>; second: Shared<number> }
export type AnonymousFamily = { kind: 'a'; payload: string } | { kind: 'b'; payload: number };
export interface SameNamedArguments { first: Shared<A.Foo>; second: Shared<B.Foo> }
export interface SameNamedNullable { first: A.Foo | null; second: B.Foo | null }
