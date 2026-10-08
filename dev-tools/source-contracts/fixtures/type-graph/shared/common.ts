export interface NumericValue { value: number }
export interface Common {
  level: NumericValue;
  usage?: { value: string } | null;
  flags: Record<string, unknown>;
  legacy: object;
  explicit: any;
  provider: ProviderID;
}
export interface Nested<T> { payload: T; children: Nested<T>[] }
