import { GenericRule, InheritedRule as OtherRule } from './example.js';
export class RuleElements {
  static readonly builtin: Record<string, unknown> = { Example: GenericRule, Inherited: OtherRule };
}
