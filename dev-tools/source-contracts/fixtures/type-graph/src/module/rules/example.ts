interface Field<T> { source: T }
interface ExampleSchema { amount: Field<number>; choices: Field<string[]> }
export class GenericRule<T extends ExampleSchema> {
  static defineSchema(): ExampleSchema { throw new Error('Must never execute upstream code'); }
  preparedOnly(): Date { throw new Error('Runtime method'); }
  declare _source: SourceFromSchema<T>;
}
export class InheritedRule extends GenericRule<ExampleSchema> {}

declare global { type SourceFromSchema<T> = { [K in keyof T]: T[K] extends Field<infer S> ? S : never } }
