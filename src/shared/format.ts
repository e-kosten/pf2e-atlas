const INTEGER_FORMATTER = new Intl.NumberFormat("en-US");

export function formatInteger(value: number): string {
  return INTEGER_FORMATTER.format(value);
}
