import type * as Atlas from "../generated/atlas";
import { summaryFixture } from "../test/fixtures";
import { getEmbeddedReading, getRecordDetail, getTraitDetail } from "./atlasApi";
import {
  API_CONTRACT,
  API_CONTRACT_HEADER,
  clearCompatibilityFailure,
} from "./compatibility";
import { convertConditionParameters } from "./encounterConversion";
import { convertNumberFact, convertRecordPresentation } from "./presentationConversion";
import type { Wire } from "./wire";

const fetchMock = vi.fn();
const generation = "prepared-context-generation";
const identity = summaryFixture("actors:prepared-context", "Prepared context");
const navigation: Atlas.RecordNavigationView = {
  record_key: identity.record_key,
  embedded_path: [],
  field: null,
  passage: null,
  source_fingerprint: "prepared-snapshot",
};
const source: Atlas.ContentLocationView = {
  record: identity.record_key,
  embedded_path: [],
  field: "system.description.value",
};

beforeEach(() => {
  clearCompatibilityFailure();
  fetchMock.mockReset();
  vi.stubGlobal("fetch", fetchMock);
});
afterEach(() => vi.unstubAllGlobals());

function respond(value: unknown) {
  fetchMock.mockResolvedValue(
    new Response(JSON.stringify(value), {
      headers: {
        "Content-Type": "application/json",
        [API_CONTRACT_HEADER]: API_CONTRACT,
      },
    }),
  );
}

function known<T>(value: T): Atlas.FactView<T> {
  return { state: "value", value };
}
function unavailable(): { state: "not_applicable"; value: null } {
  return { state: "not_applicable", value: null };
}
function numeric(value: number): Wire<Atlas.NumberFactView> {
  return { state: "value", value, adjustment: null };
}
function adjusted(modifier: number): Wire<Atlas.NumberFactView> {
  return {
    state: "value",
    value: 0.5,
    adjustment: {
      authored: -1.5,
      applied: [
        { source: "effect", label: "Effect", modifier_type: "status", value: modifier },
      ],
      suppressed: [
        { source: "other", label: "Other", modifier_type: "status", value: 0 },
      ],
      notes: [],
    },
  };
}
function presentation(
  body: Wire<Atlas.PreparedFieldBodyView>,
): Wire<Atlas.RecordPresentationView> {
  return {
    classification: "Reference",
    identity,
    embedded: [],
    body: { kind: "content" },
    content: [{ locator: source, role: "description", source_fingerprint: null, body }],
  };
}

const rollSelection: Wire<Atlas.RollContentRequest> = {
  content_interpretation_version: "test-content-version",
  source,
  source_fingerprint: "prepared-snapshot",
  ordinal: 0,
  authored_formula: "1d6",
  authored_command: null,
  spell_rank: 3,
};
const preparedBody: Wire<Atlas.PreparedFieldBodyView> = {
  kind: "html",
  html: "<p>Prepared rolls</p>",
  controls: [
    {
      ordinal: 0,
      control: {
        kind: "damage",
        formula: "1d6",
        options: {},
        literal: null,
        roll: {
          ...rollSelection,
          encounter: {
            encounter_ref: "encounter",
            participant_key: "participant",
            participant_revision: Number.MAX_SAFE_INTEGER,
            artifact_generation: generation,
            condition_id: 9,
          },
        },
      },
    },
    {
      ordinal: 1,
      control: {
        kind: "command",
        command: "r",
        arguments: "1d6",
        options: {},
        literal: null,
        roll: rollSelection,
      },
    },
  ],
  inclusions: [
    {
      namespace: 1,
      source,
      references: [],
      controls: [
        {
          ordinal: 0,
          control: {
            kind: "command",
            command: "r",
            arguments: "1d6",
            options: {},
            literal: null,
            roll: {
              ...rollSelection,
              encounter: {
                encounter_ref: "encounter",
                participant_key: "participant",
                participant_revision: 0,
                artifact_generation: generation,
                condition_id: null,
              },
            },
          },
        },
      ],
    },
  ],
};

async function readPreparedBody(
  root: "record" | "embedded" | "trait",
): Promise<Atlas.PreparedFieldBodyView> {
  const content = presentation(preparedBody).content;
  if (root === "record") {
    const response: Wire<Atlas.RecordDetailView> = {
      artifact_generation: generation,
      record: identity,
      selected: navigation,
      relationships: [],
      relationships_truncated: false,
      presentation: presentation(preparedBody),
    };
    respond(response);
    return (await getRecordDetail(identity.record_key)).presentation.content[0].body;
  }
  if (root === "embedded") {
    const response: Wire<Atlas.EmbeddedReadingBatchView> = {
      artifact_generation: generation,
      parent: navigation,
      availability: "value",
      total: 1,
      start: 0,
      end: 1,
      previous: null,
      next: null,
      entries: [{ navigation, content, relationships: [] }],
    };
    respond(response);
    const result = await getEmbeddedReading({
      parent: navigation,
      artifact_generation: generation,
      selection: { kind: "actor_content" },
    });
    expect(result.total).toBe(1);
    return result.entries[0].content[0].body;
  }
  const response: Wire<Atlas.TraitDetailView> = {
    artifact_generation: generation,
    trait_key: "trait:fire",
    label: "Fire",
    content: { body: preparedBody, references: [] },
    catalogs: ["traits"],
    source_revision: null,
    indexing_locale: "en",
  };
  respond(response);
  const result = await getTraitDetail({
    trait_key: response.trait_key,
    expected_artifact_generation: generation,
  });
  expect(result.content).not.toBeNull();
  return result.content!.body;
}

it.each(["record", "embedded", "trait"] as const)(
  "converts direct and included roll contexts through the %s response adapter",
  async (root) => {
    const result = await readPreparedBody(root);
    if (result.kind !== "html") throw new Error("Expected prepared HTML");
    const direct = result.controls[0].control;
    const absent = result.controls[1].control;
    const included = result.inclusions[0].controls[0].control;
    if (
      direct.kind !== "damage" ||
      absent.kind !== "command" ||
      included.kind !== "command"
    )
      throw new Error("Expected prepared roll controls");
    expect(direct.roll?.encounter).toEqual({
      encounter_ref: "encounter",
      participant_key: "participant",
      participant_revision: 9007199254740991n,
      artifact_generation: generation,
      condition_id: 9n,
    });
    expect(included.roll?.encounter).toMatchObject({
      participant_revision: 0n,
      condition_id: null,
    });
    expect(absent.roll).not.toHaveProperty("encounter");
    expect(direct.roll?.spell_rank).toBe(3);
    expect(result.inclusions[0].namespace).toBe(1);
    expect(result.controls[0].ordinal).toBe(0);
    expect(fetchMock).toHaveBeenCalledOnce();
  },
);

it.each([Number.MIN_SAFE_INTEGER, -2, 0, Number.MAX_SAFE_INTEGER])(
  "converts modifier %s without changing number-valued effective/authored facts",
  (modifier) => {
    const result = convertNumberFact(adjusted(modifier));
    expect(result.value).toBe(0.5);
    expect(result.adjustment?.authored).toBe(-1.5);
    expect(result.adjustment?.applied[0].value).toBe(BigInt(modifier));
    expect(result.adjustment?.suppressed[0].value).toBe(0n);
  },
);

it("converts nested kit adjustments through physical and recursive item branches", () => {
  const input: Wire<Atlas.RecordPresentationView> = {
    ...presentation({ kind: "plain", text: "A kit" }),
    body: {
      kind: "physical_reference",
      value: {
        base_type: known({ value: "kit", label: "Kit" }),
        details: {
          kind: "kit",
          value: {
            items: known([
              {
                name: known("Pack"),
                quantity: adjusted(-2),
                items: known([
                  { name: known("Rope"), quantity: adjusted(0), items: unavailable() },
                ]),
              },
            ]),
          },
        },
        quantity: numeric(1),
        material: unavailable(),
        size: known("Medium"),
        consumable: null,
        usage: known("worn"),
        bulk: numeric(0.1),
        price: unavailable(),
        price_per: numeric(1),
      },
    },
  };
  const result = convertRecordPresentation(input);
  if (
    result.body.kind !== "physical_reference" ||
    result.body.value.details?.kind !== "kit"
  )
    throw new Error("Expected kit presentation");
  const pack = result.body.value.details.value.items.value?.[0];
  const rope = pack?.items.value?.[0];
  expect(pack?.quantity.adjustment?.applied[0].value).toBe(-2n);
  expect(rope?.quantity.adjustment?.applied[0].value).toBe(0n);
  expect(rope?.items.value).toBeNull();
  expect(result.body.value.bulk.value).toBe(0.1);
  expect(result.content[0].body).toEqual({ kind: "plain", text: "A kit" });
});

it("keeps table-result weight numeric through its presentation branch", () => {
  const input: Wire<Atlas.RecordPresentationView> = {
    ...presentation({ kind: "plain", text: "A lantern flashes." }),
    body: {
      kind: "table_result",
      value: { range: known("1–2"), weight: numeric(1), result_type: known("text") },
    },
  };
  const result = convertRecordPresentation(input);
  if (result.body.kind !== "table_result") throw new Error("Expected table result");
  expect(result.body.value.weight.value).toBe(1);
});

it("preserves absent condition parameters while converting zero and nested union integers", () => {
  type Parameters = Atlas.EncounterParticipantConditionView["parameters"];
  const absent: Wire<Parameters> = { kind: "sickened" };
  const zero: Wire<Parameters> = { kind: "sickened", recovery_dc: 0 };
  const frightened: Wire<Parameters> = {
    kind: "frightened",
    recovery: { kind: "end_turn", minimum: 0 },
  };
  const reminder: Wire<Parameters> = {
    kind: "persistent_damage_reminder",
    formula: null,
    damage_type: null,
    multiplier: "normal",
    recovery: { kind: "specified", dc: 15 },
  };
  expect(convertConditionParameters(absent)).not.toHaveProperty("recovery_dc");
  expect(convertConditionParameters(zero)).toEqual({
    kind: "sickened",
    recovery_dc: 0n,
  });
  expect(convertConditionParameters(frightened)).toEqual({
    kind: "frightened",
    recovery: { kind: "end_turn", minimum: 0n },
  });
  expect(convertConditionParameters(reminder)).toMatchObject({
    formula: null,
    recovery: { kind: "specified", dc: 15n },
  });
});
