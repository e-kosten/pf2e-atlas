import type {
  RecordNavigationView,
  QueryPredicate,
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

export function sameSelection(left: RecordNavigationView, right: RecordNavigationView) {
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
export function linkPredicate(leaf: LinkLeaf, excluded: boolean): QueryPredicate {
  return excluded ? { kind: "not", predicate: leaf } : leaf;
}

export function hasLinkCondition(
  node: QueryPredicate | null,
  relationship: QueryRelationship,
  excluded: boolean,
  exceptClauseId?: string,
): boolean {
  if (!node) return false;
  if (node.kind === "all_of")
    return node.children.some((child) =>
      hasLinkCondition(child, relationship, excluded, exceptClauseId),
    );
  const leaf = excluded && node.kind === "not" ? node.predicate : node;
  return (
    (!excluded || node.kind === "not") &&
    leaf.kind === "relationship" &&
    (!exceptClauseId || leaf.clause_id !== exceptClauseId) &&
    sameRelationship(leaf.relationship, relationship)
  );
}

export function appendLink(
  filter: QueryPredicate | null,
  leaf: LinkLeaf,
  excluded = false,
): QueryPredicate {
  if (filter && hasLinkCondition(filter, leaf.relationship, excluded)) return filter;
  const condition = linkPredicate(leaf, excluded);
  return filter
    ? {
        kind: "all_of",
        clause_id: crypto.randomUUID(),
        children: [filter, condition],
      }
    : condition;
}

export type LinkLeaf = Extract<
  QueryPredicate,
  {
    kind: "relationship";
  }
>;
export function linkClauses(filter: QueryPredicate | null): LinkLeaf[] {
  if (!filter) return [];
  if (filter.kind === "relationship") return [filter];
  if (filter.kind === "all_of" || filter.kind === "any_of")
    return filter.children.flatMap(linkClauses);
  if (filter.kind === "not" || filter.kind === "exists")
    return linkClauses(filter.predicate);
  return [];
}

export function countLinkConditions(filter: QueryPredicate | null): number {
  if (!filter) return 0;
  if (filter.kind === "relationship" || filter.kind === "related_match") return 1;
  if (filter.kind === "all_of" || filter.kind === "any_of")
    return filter.children.reduce(
      (count, child) => count + countLinkConditions(child),
      0,
    );
  if (filter.kind === "not") return countLinkConditions(filter.predicate);
  return 0;
}
