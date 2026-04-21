type MultiValueArgs = Record<string, string[]>;

export function parseCliArgs(argv: string[]): MultiValueArgs {
  const parsed: MultiValueArgs = {};

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (!current || !current.startsWith("--")) {
      continue;
    }

    const [rawKey, inlineValue] = current.slice(2).split("=", 2);
    if (!rawKey) {
      continue;
    }

    const nextValue = inlineValue ?? argv[index + 1];
    if (!nextValue || nextValue.startsWith("--")) {
      parsed[rawKey] = parsed[rawKey] ?? [];
      continue;
    }

    const bucket = parsed[rawKey] ?? [];
    bucket.push(nextValue);
    parsed[rawKey] = bucket;
    if (inlineValue === undefined) {
      index += 1;
    }
  }

  return parsed;
}

export function lastValue(args: MultiValueArgs, key: string): string | undefined {
  return args[key]?.at(-1);
}

export function parseInteger(value: string | undefined, flagName: string): number | undefined {
  if (value === undefined) {
    return undefined;
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Expected ${flagName} to be an integer, received "${value}".`);
  }
  return parsed;
}
