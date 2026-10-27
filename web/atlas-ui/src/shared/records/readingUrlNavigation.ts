import type { RecordNavigationView } from "../../generated/atlas";

export function readingNavigationForUrl(
  selected: RecordNavigationView,
  request?: {
    navigation?: RecordNavigationView;
    match?: RecordNavigationView;
  } | null,
): RecordNavigationView {
  const sameDocument = (navigation: RecordNavigationView | undefined) =>
    navigation?.record_key === selected.record_key &&
    JSON.stringify(navigation.embedded_path) === JSON.stringify(selected.embedded_path);
  const match = sameDocument(request?.match) ? request?.match : undefined;
  const navigation = sameDocument(request?.navigation)
    ? request?.navigation
    : undefined;
  const rank = match?.spell_rank ?? navigation?.spell_rank;
  const { spell_rank: _resolvedRank, ...address } = selected;
  return rank === undefined ? address : { ...address, spell_rank: rank };
}
