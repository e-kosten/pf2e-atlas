import wire from "./__fixtures__/wire-conversion.json";
import {
  AtlasApiError,
  getEncounter,
  getEncounterReference,
  getReadiness,
  getRecordDetail,
  getTraitRecords,
  previewEncounterRecovery,
  rollFlatCheck,
  rollTable,
} from "./atlasApi";
import {
  API_CONTRACT,
  API_CONTRACT_HEADER,
  clearCompatibilityFailure,
} from "./compatibility";
import { totalPages } from "../features/search/pageMetrics";
import type { RecordPresentationView } from "../generated/atlas";
import { toBigInt } from "./wire";

const fetchMock = vi.fn();
beforeEach(() => {
  clearCompatibilityFailure();
  vi.stubGlobal("fetch", fetchMock);
});
afterEach(() => {
  vi.unstubAllGlobals();
  fetchMock.mockReset();
});
function respond(value: unknown) {
  fetchMock.mockResolvedValue(
    new Response(JSON.stringify(value), {
      headers: { [API_CONTRACT_HEADER]: API_CONTRACT },
    }),
  );
}
const traitRequest = {
  trait_key: "trait:fire",
  kind: "members" as const,
  page: { number: 1, size: 25 },
  expected_artifact_generation: "fixture-generation",
  search_profile: null,
  ranking: null,
};
const actorKey = "actors:cccccccccccccccc";
const referenceRequest = {
  artifact_generation: "fixture-generation",
  selection: {
    record_key: actorKey,
    embedded_path: [],
    fields: [],
    passage: null,
    source_fingerprint: null,
  },
};

function expectRuntimePresentation(presentation: RecordPresentationView) {
  const body = presentation.body;
  if (body.kind !== "creature") throw new Error("fixture must remain a creature");
  const ac = body.value.armor_class;
  expect(ac.value).toBe(17);
  expect(ac.adjustment?.authored).toBe(16);
  expect(ac.adjustment?.applied.map((modifier) => modifier.value)).toEqual([2n, -1n]);
  const actions = body.value.runtime?.action_budget?.actions;
  expect(actions?.base_value).toBe(3n);
  expect(actions?.adjusted_value).toBe(2n);
  expect(actions?.adjustments.map((modifier) => modifier.value)).toEqual([-1n]);
}

it("decodes Rust/router trait totals for the real bigint paging consumer", async () => {
  respond(wire.trait_records);
  const result = await getTraitRecords(traitRequest);
  expect(result.page.total).toBe(1n);
  expect(result.page.number).toBe(1);
  expect(result.page.size).toBe(25);
  expect(result.page.count).toBe(1);
  expect(totalPages(result.page)).toBe(1n);
  expect(fetchMock).toHaveBeenCalledOnce();
});

it("decodes the actual encounter graph with present counts, pending nulls and omitted HP", async () => {
  respond(wire.encounter);
  const detail = await getEncounter("fixture-encounter");
  const current = detail.participants.find(
    (p) => p.participant_key === "fixture-manual",
  )!;
  const pending = detail.participants.find(
    (p) => p.participant_key === "fixture-zero",
  )!;
  expect(detail.revision).toBe(BigInt(wire.encounter.revision));
  expect(current.turn_resources?.regular_total).toBe(2n);
  expect(current.turn_resources?.round_number).toBe(1n);
  expect(current.turn_resources?.forecast.base_value).toBe(3n);
  expect(current.turn_resources?.action_calculation.kind).toBe("this_turn");
  if (current.turn_resources?.action_calculation.kind === "this_turn")
    expect(current.turn_resources.action_calculation.count?.adjustments[0].value).toBe(
      -1n,
    );
  expect(pending.turn_resources?.regular_total).toBeNull();
  expect(current).not.toHaveProperty("current_hp");
  expect(current).not.toHaveProperty("max_hp");
  expect(pending.current_hp).toBe(0n);
  expect(pending.max_hp).toBe(20n);
  expect(current.conditions.map((c) => c.value)).toEqual([1n]);
  expect(pending.effective_max_hp.value).toBe(20);
});

it("converts an actual participant reference and keeps authored/effective facts numeric", async () => {
  respond(wire.record_detail);
  expectRuntimePresentation(
    (
      await getEncounterReference(
        "fixture-encounter",
        "fixture-actor",
        referenceRequest,
      )
    ).presentation,
  );
});

it("converts the shared serialized presentation nested in a participant response", async () => {
  respond({
    ...wire.encounter,
    participants: [
      {
        ...wire.encounter.participants[0],
        presentation: wire.record_detail.presentation,
      },
    ],
  });
  const detail = await getEncounter("fixture-encounter");
  expectRuntimePresentation(detail.participants[0].presentation!);
});

it.each([false, true])(
  "routes ordinary record detail conversion through GET/POST (selection=%s)",
  async (selected) => {
    respond(wire.record_detail);
    const detail = await getRecordDetail(
      actorKey,
      selected ? referenceRequest.selection : undefined,
    );
    expect(detail.record.record_key).toBe(actorKey);
    expectRuntimePresentation(detail.presentation);
  },
);

it("converts nested table details using the shared serialized DTO and retains numeric dice", async () => {
  respond({
    outcome: wire.flat_check_serialization_control.outcome,
    matches: [wire.record_detail],
    invalid_ranges: [],
  });
  const request = {
    artifact_generation: "fixture-generation",
    selection: { ...referenceRequest.selection, record_key: "tables:dddddddddddddddd" },
  };
  const actual = await rollTable(request);
  expect(actual.outcome.total).toBe(11);
  expectRuntimePresentation(actual.matches[0].presentation);
});

it("preserves recovery null/zero and decodes flat DC without altering dice outcomes", async () => {
  respond(wire.recovery_preview);
  const preview = await previewEncounterRecovery("fixture-encounter", {
    expected_encounter_revision: BigInt(wire.encounter.revision),
    action: {
      kind: "rest_overnight",
      healing_mode: "natural",
      participants: [
        { participant_key: "fixture-zero", healing: 0n },
        { participant_key: "fixture-manual", healing: 0n },
      ],
    },
  });
  const manual = preview.participants.find(
    (p) => p.participant_key === "fixture-manual",
  )!;
  expect(preview.encounter_revision).toBe(BigInt(wire.encounter.revision));
  expect(manual.current_hp).toBeNull();
  expect(manual.resulting_hp).toBeNull();
  expect(manual.healing).toBe(0n);
  expect(
    preview.participants.find((p) => p.participant_key === "fixture-zero")?.current_hp,
  ).toBe(0n);
  expect(
    JSON.parse(fetchMock.mock.calls[0][1].body).action.participants[0].healing,
  ).toBe(0);
  respond(wire.flat_check_serialization_control);
  const flat = await rollFlatCheck({ dc: 11n });
  expect(flat.dc).toBe(11n);
  expect(flat.outcome.total).toBe(11);
  expect(flat.success).toBe(true);
  expect(fetchMock).toHaveBeenCalledTimes(2);
});

it.each([null, 42, "unavailable"])(
  "retains legacy malformed non-array ability states %s",
  async (ability_states) => {
    respond({
      ...wire.encounter,
      participants: [{ ...wire.encounter.participants[0], ability_states }],
    });
    expect(
      (await getEncounter("fixture-encounter")).participants[0].ability_states,
    ).toEqual(ability_states);
  },
);

it.each([null, 42, "invalid"])(
  "rejects present malformed condition parameters %s",
  async (parameters) => {
    respond({
      ...wire.encounter,
      participants: [
        {
          ...wire.encounter.participants[0],
          conditions: [{ ...wire.encounter.participants[0].conditions[0], parameters }],
        },
      ],
    });
    await expect(getEncounter("fixture-encounter")).rejects.toThrow(
      "Invalid condition parameters",
    );
  },
);

it("preserves omitted persistent DC, rejects explicit null and retains object validation", async () => {
  const condition = wire.encounter.participants[0].conditions[0];
  const encounter = (recovery: unknown) => ({
    ...wire.encounter,
    participants: [
      {
        ...wire.encounter.participants[0],
        conditions: [
          {
            ...condition,
            parameters: {
              kind: "persistent_damage",
              formula: "1",
              damage_type: "fire",
              multiplier: "normal",
              recovery,
            },
          },
        ],
      },
    ],
  });
  respond(encounter({ kind: "specified" }));
  const result = await getEncounter("fixture-encounter");
  const parameters = result.participants[0].conditions[0].parameters;
  if (parameters.kind !== "persistent_damage") throw new Error("wrong condition");
  expect(parameters.recovery).not.toHaveProperty("dc");
  respond(encounter({ kind: "specified", dc: null }));
  await expect(getEncounter("fixture-encounter")).rejects.toBeInstanceOf(AtlasApiError);
  respond(encounter(null));
  await expect(getEncounter("fixture-encounter")).rejects.toThrow(
    "Invalid persistent recovery",
  );
});

it.each([
  "{",
  '{"value":1e400}',
  '{"value":9007199254740992}',
  '{"value":-9007199254740992}',
])("rejects malformed/nonfinite/unsafe raw response %s", async (text) => {
  fetchMock.mockResolvedValue(
    new Response(text, { headers: { [API_CONTRACT_HEADER]: API_CONTRACT } }),
  );
  await expect(getReadiness()).rejects.toBeInstanceOf(AtlasApiError);
});

it("retains both safe boundaries, zero and decimal-string helper behavior", () => {
  expect(toBigInt(Number.MAX_SAFE_INTEGER)).toBe(9007199254740991n);
  expect(toBigInt(Number.MIN_SAFE_INTEGER)).toBe(-9007199254740991n);
  expect(toBigInt(0)).toBe(0n);
  expect(toBigInt("-7")).toBe(-7n);
  expect(() => toBigInt(0.1)).toThrow(AtlasApiError);
});
