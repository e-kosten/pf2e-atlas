import type { RecordNavigationView } from "../../generated/atlas";
import { readingNavigationForUrl } from "./readingUrlNavigation";

const root: RecordNavigationView = {
  record_key: "spells:fireball",
  embedded_path: [],
  field: null,
  passage: null,
  source_fingerprint: null,
};

it("omits a resolved default rank without changing the service's selected address", () => {
  const selected = {
    ...root,
    field: "/system/description/value",
    passage: { kind: "identity" as const },
    source_fingerprint: "source-a",
    spell_rank: 3,
  };
  expect(readingNavigationForUrl(selected, { navigation: root })).toEqual({
    ...root,
    field: "/system/description/value",
    passage: { kind: "identity" },
    source_fingerprint: "source-a",
  });
  expect(selected.spell_rank).toBe(3);
});

it.each([0, 3, 8])(
  "retains explicitly requested rank %s, including the source default",
  (rank) => {
    expect(
      readingNavigationForUrl(
        { ...root, spell_rank: 3 },
        { navigation: { ...root, spell_rank: rank } },
      ).spell_rank,
    ).toBe(rank);
  },
);

it("retains contextual match rank only for its addressed document", () => {
  const match: RecordNavigationView = {
    ...root,
    record_key: "actors:caster",
    embedded_path: [{ collection: "items", identity: { Stable: "spell" } }],
    spell_rank: 8,
  };
  const navigation = { ...root, record_key: "actors:caster" };
  const request = { navigation, match };
  expect(readingNavigationForUrl({ ...match, spell_rank: 3 }, request).spell_rank).toBe(
    8,
  );
  expect(readingNavigationForUrl(navigation, request)).not.toHaveProperty("spell_rank");
});
