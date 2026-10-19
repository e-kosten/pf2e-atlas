export type SourceFromSchema<T> = { [K in keyof T]: T[K] extends { source: infer S } ? S : never };
