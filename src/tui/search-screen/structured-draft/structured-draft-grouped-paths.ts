export function getGroupedFieldChildIndexes(groupPath: number[], fieldMemberPaths: readonly number[][]): number[] {
  const childIndexes = new Set<number>();

  for (const memberPath of fieldMemberPaths) {
    if (memberPath.length <= groupPath.length) {
      continue;
    }
    const isInGroup = groupPath.every((segment, index) => memberPath[index] === segment);
    if (!isInGroup) {
      continue;
    }

    const childIndex = memberPath[groupPath.length];
    if (childIndex !== undefined) {
      childIndexes.add(childIndex);
    }
  }

  return [...childIndexes].sort((left, right) => left - right);
}
