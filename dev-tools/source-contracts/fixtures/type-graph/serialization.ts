import type { ModifierAdjustment } from './src/module/actor/modifiers.js';
import type { Predicate, RawPredicate } from './src/module/system/predication.js';
import type { PickableThing } from './src/module/apps/pick-a-thing-prompt.js';

export interface SerializedSource {
  adjustments?: ModifierAdjustment[];
  predicate: Predicate;
  rawPredicate: RawPredicate;
  choices?: PickableThing[];
}

export class OtherRuntimeClass { value = 1; toObject() { return { value: this.value }; } }
export interface OtherSource { runtime: OtherRuntimeClass }
