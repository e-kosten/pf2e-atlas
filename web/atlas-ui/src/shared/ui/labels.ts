// Typography only: identifiers and filter values are never changed.
export function sentenceCase(label: string): string {
  const value = label.trim();
  return value ? value[0].toLocaleUpperCase("en") + value.slice(1) : value;
}
