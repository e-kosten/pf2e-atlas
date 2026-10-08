export type PredicateStatement = string | { and: PredicateStatement[] } | { not: PredicateStatement };
export type RawPredicate = PredicateStatement[];
export class Predicate extends Array<PredicateStatement> {
  constructor(...statements: PredicateStatement[] | [PredicateStatement[]]) {
    super(...(Array.isArray(statements[0]) ? statements[0] : statements as PredicateStatement[]));
  }
  readonly isValid = true;
  test(_options: string[]): boolean { return true; }
  toObject(): RawPredicate { return [...this]; }
}
