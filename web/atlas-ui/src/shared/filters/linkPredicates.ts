import type {
  RecordNavigationView,
  QueryRelationship,
  SourcePassageAddress,
} from "../../generated/atlas";

function samePassage(
  left: SourcePassageAddress | null | undefined,
  right: SourcePassageAddress | null | undefined,
): boolean {
  if (left == null || right == null) return left == null && right == null;
  if (left.kind !== right.kind) return false;
  if (left.kind === "identity" || right.kind === "identity") return true;
  return (
    left.selection_version === right.selection_version &&
    left.section_ordinal === right.section_ordinal &&
    left.label === right.label &&
    left.chunk_bytes.start === right.chunk_bytes.start &&
    left.chunk_bytes.end === right.chunk_bytes.end &&
    (left.kind === "html_section" && right.kind === "html_section"
      ? left.prepared_html_sha256 === right.prepared_html_sha256 &&
        left.canonical_text_sha256 === right.canonical_text_sha256
      : left.kind === "plain_section" &&
        right.kind === "plain_section" &&
        left.source_text_sha256 === right.source_text_sha256)
  );
}

function sameSelection(left: RecordNavigationView, right: RecordNavigationView) {
  return (
    left.record_key === right.record_key &&
    (left.field ?? null) === (right.field ?? null) &&
    (left.source_fingerprint ?? null) === (right.source_fingerprint ?? null) &&
    (left.spell_rank ?? null) === (right.spell_rank ?? null) &&
    samePassage(left.passage, right.passage) &&
    left.embedded_path.length === right.embedded_path.length &&
    left.embedded_path.every((step, index) => {
      const other = right.embedded_path[index];
      return (
        step.collection === other.collection &&
        ("Stable" in step.identity && "Stable" in other.identity
          ? step.identity.Stable === other.identity.Stable
          : "SnapshotLocal" in step.identity &&
            "SnapshotLocal" in other.identity &&
            step.identity.SnapshotLocal.index === other.identity.SnapshotLocal.index)
      );
    })
  );
}

export function sameRelationship(left: QueryRelationship, right: QueryRelationship) {
  return (
    left.direction === right.direction &&
    left.include_contents === right.include_contents &&
    sameSelection(left.selection, right.selection)
  );
}

export function linkLeaf(
  relationship: QueryRelationship,
  clause_id: string = crypto.randomUUID(),
): Extract<import("../../generated/atlas").QueryPredicate, { kind: "relationship" }> & {
  clause_id: string;
} {
  return { kind: "relationship", clause_id, relationship };
}
export function appendLink(
  filter: import("../../generated/atlas").QueryPredicate | null,
  leaf: Extract<
    import("../../generated/atlas").QueryPredicate,
    { kind: "relationship" }
  >,
) {
  const direct = filter?.kind === "all_of" ? filter.children : filter ? [filter] : [];
  if (
    direct.some(
      (node) =>
        node.kind === "relationship" &&
        sameRelationship(node.relationship, leaf.relationship),
    )
  )
    return filter;
  return filter
    ? {
        kind: "all_of" as const,
        clause_id: crypto.randomUUID(),
        children: [filter, leaf],
      }
    : leaf;
}
