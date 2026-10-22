import type { RelationshipConstraintView } from "../../generated/atlas";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { sameRelationshipConstraint } from "./relationshipConstraints";

const constraint: RelationshipConstraintView = {
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
  const reordered: RelationshipConstraintView = {
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
  expect(sameRelationshipConstraint(constraint, reordered)).toBe(true);
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
] satisfies RelationshipConstraintView[])(
  "keeps different exact constraint identities distinct (%#)",
  (other) => {
    expect(sameRelationshipConstraint(constraint, other)).toBe(false);
  },
);
