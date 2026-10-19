import { describe, expect, it } from "vitest";
import type { EncounterParticipantView } from "../../generated/atlas";
import { participantUpdate } from "./participantEdits";

describe("participantEdits", () => {
  it("keeps unknown HP unknown and preserves edit origin for unrelated edits", () => {
    const unknown = participantFixture({ current_hp: undefined, max_hp: undefined });
    expect(participantUpdate(unknown, { display_name: "Changed" })).toMatchObject({
      hp_edit: false,
      max_hp_edit: false,
      variant_edit: false,
      use_derived_max: false,
    });
    expect(participantUpdate(unknown, { current_hp: 0n })).toMatchObject({
      hp_edit: true,
    });
    expect(participantUpdate(unknown, { max_hp: 12n })).toMatchObject({
      max_hp_edit: true,
    });
    expect(participantUpdate(unknown, { participant_variant: "normal" })).toMatchObject(
      { variant_edit: true },
    );
    expect(participantUpdate(unknown, { use_derived_max: true })).toMatchObject({
      use_derived_max: true,
      max_hp_edit: false,
    });
  });
  it("preserves unchanged participant update fields", () => {
    expect(
      participantUpdate(participantFixture(), { current_hp: BigInt(12) }),
    ).toMatchObject({
      participant_key: "participant-1",
      expected_revision: 0n,
      display_name: "Goblin 1",
      side: "enemy",
      participant_variant: "normal",
      initiative: BigInt(18),
      max_hp: BigInt(40),
      current_hp: BigInt(12),
      temporary_hp: BigInt(0),
      defeated: false,
      hazard_disabled: false,
      hidden: false,
      note: "watch",
    });
  });
  it("binds replacements to the viewed participant identity and revision", () => {
    const changes = {
      participant_key: "different-participant",
      expected_revision: 999n,
      current_hp: 12n,
    };
    expect(
      participantUpdate(participantFixture({ revision: 7n }), changes),
    ).toMatchObject({
      participant_key: "participant-1",
      expected_revision: 7n,
      current_hp: 12n,
    });
  });
});

function participantFixture(
  overrides: Partial<EncounterParticipantView> = {},
): EncounterParticipantView {
  return {
    revision: 0n,
    participant_key: "participant-1",
    position: BigInt(1),
    initiative_order: BigInt(1),
    display_name: "Goblin 1",
    hp_origin: "explicit",
    variant_origin: "explicit",
    participant_kind: "creature",
    side: "enemy",
    participant_variant: "normal",
    record_key: "pack:goblin",
    status: "active",
    initiative: BigInt(18),
    max_hp: BigInt(40),
    current_hp: BigInt(30),
    temporary_hp: BigInt(0),
    defeated: false,
    hazard_disabled: false,
    cast_unavailable_reason: null,
    hidden: false,
    note: "watch",
    note_hint: "watch",
    conditions: [],
    record: undefined,
    ...overrides,
  };
}
