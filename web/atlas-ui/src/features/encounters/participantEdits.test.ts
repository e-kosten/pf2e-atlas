import { describe, expect, it } from "vitest";
import type { EncounterParticipantView } from "../../generated/atlas";
import {
  encounterParticipantFixture,
  encounterRuntimeFixture,
} from "../../test/recordFixtures";
import {
  clampCurrentHp,
  damageChanges,
  evaluateHpFormula,
  healChanges,
  participantUpdate,
} from "./participantEdits";

describe("participantEdits", () => {
  it("evaluates safe additive hp formulas and rejects dice or incomplete input", () => {
    expect(evaluateHpFormula("50 + 10")).toBe(60);
    expect(evaluateHpFormula("50 - 7 + 2")).toBe(45);
    expect(evaluateHpFormula("5 - 10")).toBe(0);
    expect(evaluateHpFormula("1d6")).toBeNull();
    expect(evaluateHpFormula("5 +")).toBeNull();
  });

  it("clamps current hp to the typed runtime maximum", () => {
    const participant = participantWithVitals({ maximumHp: 40 });
    expect(clampCurrentHp(participant, -5)).toBe(0);
    expect(clampCurrentHp(participant, 20)).toBe(20);
    expect(clampCurrentHp(participant, 45)).toBe(40);
  });

  it("applies damage to temporary hp before current hp", () => {
    expect(
      damageChanges(
        participantWithVitals({ currentHp: 30, maximumHp: 40, temporaryHp: 5 }),
        12,
      ),
    ).toMatchObject({
      current_hp: 23,
      temporary_hp: 0,
      defeated: false,
    });
  });

  it("marks defeated at zero and caps healing at maximum hp", () => {
    expect(damageChanges(participantWithVitals({ currentHp: 5 }), 10)).toMatchObject({
      current_hp: 0,
      defeated: true,
    });
    expect(healChanges(participantWithVitals({ currentHp: 35 }), 10)).toEqual({
      current_hp: 40,
    });
  });

  it("projects edits through the nested encounter runtime DTO", () => {
    expect(
      participantUpdate(participantWithVitals({}), { current_hp: 12 }),
    ).toMatchObject({
      participant_key: "participant-1",
      display_name: "Goblin Warrior",
      side: "enemy",
      participant_variant: "normal",
      initiative: 12,
      max_hp: 40,
      current_hp: 12,
      temporary_hp: 0,
      defeated: false,
      hidden: false,
    });
  });
});

function participantWithVitals({
  currentHp = 30,
  maximumHp = 40,
  temporaryHp = 0,
}: {
  currentHp?: number;
  maximumHp?: number;
  temporaryHp?: number;
}): EncounterParticipantView {
  const participant = encounterParticipantFixture();
  return {
    ...participant,
    record_view: {
      ...participant.record_view,
      encounter: encounterRuntimeFixture({ currentHp, maximumHp, temporaryHp }),
    },
  };
}
