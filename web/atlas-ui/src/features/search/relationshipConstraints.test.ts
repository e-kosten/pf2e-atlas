import type { QueryRelationship } from "../../generated/atlas";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { sameRelationship } from "../../shared/filters/linkPredicates";

const constraint: QueryRelationship = {
  direction: "incoming",
  include_contents: true,
  selection: {
    ...rootNavigation("actors:one"),
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 2 } } }],
    field: "description",
    source_fingerprint: "snapshot-one",
    passage: {
      kind: "html_section",
      prepared_html_sha256: "html",
      canonical_text_sha256: "text",
      selection_version: "v1",
      section_ordinal: 0,
      label: "Effect",
      chunk_bytes: { start: 0, end: 20 },
    },
  },
};

it("compares complete exact selections structurally regardless of object property order", () => {
  const reordered: QueryRelationship = {
    selection: {
      source_fingerprint: "snapshot-one",
      passage: {
        chunk_bytes: { end: 20, start: 0 },
        label: "Effect",
        section_ordinal: 0,
        selection_version: "v1",
        canonical_text_sha256: "text",
        prepared_html_sha256: "html",
        kind: "html_section",
      },
      field: "description",
      embedded_path: [
        { identity: { SnapshotLocal: { index: 2 } }, collection: "items" },
      ],
      record_key: "actors:one",
    },
    include_contents: true,
    direction: "incoming",
  };
  expect(sameRelationship(constraint, reordered)).toBe(true);
});

it.each([
  { ...constraint, direction: "outgoing" as const },
  { ...constraint, include_contents: false },
  { ...constraint, selection: { ...constraint.selection, record_key: "actors:two" } },
  { ...constraint, selection: { ...constraint.selection, embedded_path: [] } },
  {
    ...constraint,
    selection: {
      ...constraint.selection,
      embedded_path: [
        { collection: "items", identity: { SnapshotLocal: { index: 3 } } },
      ],
    },
  },
  {
    ...constraint,
    selection: {
      ...constraint.selection,
      embedded_path: [{ collection: "items", identity: { Stable: "stable-child" } }],
    },
  },
  { ...constraint, selection: { ...constraint.selection, field: "other" } },
  {
    ...constraint,
    selection: { ...constraint.selection, source_fingerprint: "snapshot-two" },
  },
  { ...constraint, selection: { ...constraint.selection, passage: null } },
] satisfies QueryRelationship[])(
  "keeps different exact constraint identities distinct (%#)",
  (other) => {
    expect(sameRelationship(constraint, other)).toBe(false);
  },
);

it("deduplicates only direct positive AND siblings while preserving OR and NOT meaning", async () => {
  const { appendLink, linkLeaf } = await import("../../shared/filters/linkPredicates");
  const leaf = linkLeaf(constraint, "original");
  const duplicate = linkLeaf(constraint, "new");
  expect(appendLink(leaf, duplicate)).toBe(leaf);
  const conjunction = {
    kind: "all_of" as const,
    children: [leaf, { kind: "boolean_constant" as const, value: true }],
  };
  expect(appendLink(conjunction, duplicate)).toBe(conjunction);
  for (const filter of [
    {
      kind: "any_of" as const,
      children: [leaf, { kind: "boolean_constant" as const, value: false }],
    },
    { kind: "not" as const, predicate: leaf },
  ])
    expect(appendLink(filter, duplicate)).toMatchObject({
      kind: "all_of",
      children: [filter, duplicate],
    });
});
