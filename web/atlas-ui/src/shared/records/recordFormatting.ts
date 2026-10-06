export function formatSigned(value: number) {
  return value >= 0 ? `+${value}` : value.toString();
}

export function formatSlug(value: string) {
  return value
    .replace(/[_-]+/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

export function formatRank(rank: number | undefined) {
  if (rank === undefined) return "Unranked";
  if (rank === 0) return "Cantrips";
  const suffix =
    rank % 10 === 1 && rank % 100 !== 11
      ? "st"
      : rank % 10 === 2 && rank % 100 !== 12
        ? "nd"
        : rank % 10 === 3 && rank % 100 !== 13
          ? "rd"
          : "th";
  return `${rank}${suffix}`;
}
