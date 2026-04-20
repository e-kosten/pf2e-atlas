export function clampLimit(value: number | undefined, fallback = 20, max = 100): number {
  if (value === undefined || Number.isNaN(value)) {
    return fallback;
  }

  return Math.max(1, Math.min(max, Math.trunc(value)));
}

export function clampOffset(value: number | undefined): number {
  if (value === undefined || Number.isNaN(value)) {
    return 0;
  }

  return Math.max(0, Math.trunc(value));
}

export function bigramDice(query: string, candidate: string): number {
  if (query === candidate) {
    return 1;
  }

  if (query.length < 2 || candidate.length < 2) {
    return query === candidate ? 1 : 0;
  }

  const queryBigrams = new Map<string, number>();
  for (let index = 0; index < query.length - 1; index += 1) {
    const slice = query.slice(index, index + 2);
    queryBigrams.set(slice, (queryBigrams.get(slice) ?? 0) + 1);
  }

  let overlap = 0;
  for (let index = 0; index < candidate.length - 1; index += 1) {
    const slice = candidate.slice(index, index + 2);
    const count = queryBigrams.get(slice) ?? 0;
    if (count > 0) {
      overlap += 1;
      queryBigrams.set(slice, count - 1);
    }
  }

  return (2 * overlap) / (query.length - 1 + (candidate.length - 1));
}
