/** Compile only authored numeric shapes supported by the encounter evaluator. */
export type ConditionValue = { kind: 'fixed'; value: number }
  | { kind: 'magnitude'; coefficient: number }
  | { kind: 'levelMagnitude'; coefficient: number; minimumLevel: number };
type Expr = { kind: 'number'; value: number } | { kind: 'variable'; name: string }
  | { kind: 'negative'; value: Expr } | { kind: 'multiply' | 'min' | 'max'; left: Expr; right: Expr };
const integer = (n: number): number => {
  if (!Number.isSafeInteger(n)) throw new Error('Condition expressions require safe integers');
  return n;
};
export function conditionValue(value: unknown): ConditionValue {
  if (typeof value === 'number') return { kind: 'fixed', value: integer(value) };
  if (typeof value !== 'string') throw new Error('Unsupported condition value');
  const tokens = value.match(/@(?:item\.badge\.value|actor\.level)|\d+|[()*,-]|min|max|\S/g) ?? [];
  let index = 0;
  const take = (expected: string) => { if (tokens[index++] !== expected) throw new Error('Unsupported expression: ' + value); };
  const primary = (): Expr => {
    const token = tokens[index++];
    if (token === '-') return { kind: 'negative', value: primary() };
    if (token === '(') { const result = product(); take(')'); return result; }
    if (token === 'min' || token === 'max') {
      take('('); const left = product(); take(','); const right = product(); take(')');
      return { kind: token, left, right };
    }
    if (token?.startsWith('@')) return { kind: 'variable', name: token };
    if (token && /^\d+$/.test(token)) return { kind: 'number', value: integer(Number(token)) };
    throw new Error('Unsupported expression: ' + value);
  };
  const product = (): Expr => {
    let result = primary();
    while (tokens[index] === '*') { index++; result = { kind: 'multiply', left: result, right: primary() }; }
    return result;
  };
  const expression = product();
  if (index !== tokens.length) throw new Error('Trailing expression: ' + value);
  const numeric = (expr: Expr): number | undefined => {
    if (expr.kind === 'number') return expr.value;
    if (expr.kind === 'negative') { const n = numeric(expr.value); return n === undefined ? undefined : integer(-n); }
    return undefined;
  };
  const scaled = (expr: Expr, name: string): number | undefined => {
    if (expr.kind === 'variable' && expr.name === name) return 1;
    if (expr.kind === 'negative') { const n = scaled(expr.value, name); return n === undefined ? undefined : -n; }
    if (expr.kind === 'multiply') {
      const a = numeric(expr.left), b = numeric(expr.right), left = scaled(expr.left, name), right = scaled(expr.right, name);
      if (a !== undefined && right !== undefined) return integer(a * right);
      if (b !== undefined && left !== undefined) return integer(b * left);
    }
    return undefined;
  };
  const constant = numeric(expression);
  if (constant !== undefined) return { kind: 'fixed', value: constant };
  const coefficient = scaled(expression, '@item.badge.value');
  if (coefficient !== undefined) return { kind: 'magnitude', coefficient };
  if (expression.kind === 'multiply') {
    for (const [floor, magnitude] of [[expression.left, expression.right], [expression.right, expression.left]]) {
      if (magnitude.kind !== 'variable' || magnitude.name !== '@item.badge.value' || floor.kind !== 'min' && floor.kind !== 'max') continue;
      for (const [level, bound] of [[floor.left, floor.right], [floor.right, floor.left]]) {
        const multiplier = scaled(level, '@actor.level'), limit = numeric(bound);
        if (multiplier === undefined || !multiplier || limit === undefined || (floor.kind === 'min') !== (multiplier < 0)) continue;
        const minimumLevel = limit / multiplier;
        if (Number.isSafeInteger(minimumLevel)) return { kind: 'levelMagnitude', coefficient: multiplier, minimumLevel };
      }
    }
  }
  throw new Error('Unsupported expression shape: ' + value);
}
export type ConditionPredicate = { kind: 'atom'; value: string }
  | { kind: 'all' | 'any'; terms: ConditionPredicate[] } | { kind: 'not'; term: ConditionPredicate };
export const predicateAtoms = new Set(['check:type:skill', 'item:trait:auditory',
  'check:statistic:base:perception', 'check:statistic:initiative']);
export function conditionPredicate(value: unknown): ConditionPredicate {
  if (Array.isArray(value)) return { kind: 'all', terms: value.map(conditionPredicate) };
  if (typeof value === 'string' && predicateAtoms.has(value)) return { kind: 'atom', value };
  if (value && typeof value === 'object') {
    const entries = Object.entries(value);
    if (entries.length === 1) {
      const [key, terms] = entries[0];
      if ((key === 'and' || key === 'or') && Array.isArray(terms)) return { kind: key === 'and' ? 'all' : 'any', terms: terms.map(conditionPredicate) };
      if (key === 'not') return { kind: 'not', term: conditionPredicate(terms) };
    }
  }
  throw new Error('Unsupported predicate: ' + JSON.stringify(value));
}
