export function expandHomePath(pathValue: string): string {
  if (pathValue === "~") {
    return process.env.HOME ?? pathValue;
  }

  if (pathValue.startsWith("~/")) {
    return `${process.env.HOME ?? ""}/${pathValue.slice(2)}`;
  }

  return pathValue;
}
