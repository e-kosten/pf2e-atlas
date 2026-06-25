import { describe, expect, it } from "vitest";
import type { EncounterParticipantView } from "../../generated/atlas";
import {
  clampCurrentHp,
  damageChanges,
  evaluateHpFormula,
  healChanges,
  participantUpdate,
} from "./participantEdits";

describe("participantEdits", () => {
  it("evaluates simple hp formulas", () => {
    expect(evaluateHpFormula("50 + 10")).toBe(60);
    expect(evaluateHpFormula("50 - 7 + 2")).toBe(45);
    expect(evaluateHpFormula("12")).toBe(12);
    expect(evaluateHpFormula("5 - 10")).toBe(0);
  });

  it("rejects invalid hp formulas", () => {
    expect(evaluateHpFormula("")).toBeNull();
    expect(evaluateHpFormula("1d6")).toBeNull();
    expect(evaluateHpFormula("5 +")).toBeNull();
    expect(evaluateHpFormula("-5")).toBeNull();
  });

  it("clamps current hp to zero and max hp", () => {
    const participant = participantFixture({ max_hp: BigInt(40) });
    expect(clampCurrentHp(participant, -5)).toBe(0);
    expect(clampCurrentHp(participant, 20)).toBe(20);
    expect(clampCurrentHp(participant, 45)).toBe(40);
  });

  it("lets current hp exceed max when max hp is unknown", () => {
    expect(clampCurrentHp(participantFixture({ max_hp: undefined }), 45)).toBe(45);
  });

  it("applies damage to temporary hp before current hp", () => {
    expect(
      damageChanges(
        participantFixture({ current_hp: BigInt(30), temporary_hp: BigInt(5) }),
        12,
      ),
    ).toMatchObject({
      current_hp: BigInt(23),
      temporary_hp: BigInt(0),
      defeated: false,
    });
  });

  it("marks defeated when damage reaches zero hp", () => {
    expect(
      damageChanges(participantFixture({ current_hp: BigInt(5) }), 10),
    ).toMatchObject({
      current_hp: BigInt(0),
      defeated: true,
    });
  });

  it("heals without exceeding max hp", () => {
    expect(healChanges(participantFixture({ current_hp: BigInt(35) }), 10)).toEqual({
      current_hp: BigInt(40),
    });
  });

  it("preserves unchanged participant update fields", () => {
    expect(
      participantUpdate(participantFixture(), { current_hp: BigInt(12) }),
    ).toMatchObject({
      participant_key: "participant-1",
      display_name: "Goblin 1",
      side: "enemy",
      participant_variant: "normal",
      initiative: BigInt(18),
      max_hp: BigInt(40),
      current_hp: BigInt(12),
      temporary_hp: BigInt(0),
      defeated: false,
      hidden: false,
      note: "watch",
    });
  });
});

function participantFixture(
  overrides: Partial<EncounterParticipantView> = {},
): EncounterParticipantView {
  return {
    participant_key: "participant-1",
    position: BigInt(1),
    initiative_order: BigInt(1),
    display_name: "Goblin 1",
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
    hidden: false,
    note: "watch",
    note_hint: "watch",
    conditions: [],
    stat_block: undefined,
    record: undefined,
    ...overrides,
  };
}
