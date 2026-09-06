import { fireEvent, render, screen } from "@testing-library/react";
import type {
  RecordSurfaceView,
  SpellDefinitionSurfaceView,
  SpellFormView,
  SpellFormResultView,
  SpellPatchView,
  SpellResolvedDefinitionView,
} from "../../generated/atlas";
import { RecordSurface } from "./RecordSurface";

const missing = { state: "missing" as const };
const known = <T,>(value: T) => ({ state: "known" as const, value });

describe("SpellRecordSurface", () => {
  it("renders Deity's Strike defenses, authored range, rich checks, Qi rules, and Planar ritual facts", () => {
    const onReference = vi.fn();
    const { rerender } = render(
      <RecordSurface onReference={onReference} surface={deityStrike()} />,
    );

    expect(screen.getByRole("heading", { name: "Deity's Strike" })).toBeInTheDocument();
    expect(screen.getByText("120 feet (emanation)")).toBeInTheDocument();
    expect(screen.queryByText("120-foot query range")).not.toBeInTheDocument();
    expect(screen.getByText("Armor Class")).toBeInTheDocument();
    expect(screen.getByText("Reflex")).toBeInTheDocument();
    expect(screen.getAllByText("No").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("DC 30 Religion check")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Alarm" }));
    expect(onReference).toHaveBeenCalledWith("spells-srd:alarm");

    rerender(<RecordSurface onReference={vi.fn()} surface={planarRitual()} />);
    expect(
      screen.getByRole("heading", { name: "Planar Displacement" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Religion (master)")).toBeInTheDocument();
    expect(screen.getByText("Arcana or Occultism")).toBeInTheDocument();

    rerender(<RecordSurface onReference={vi.fn()} surface={qiBlast()} />);
    expect(screen.getByRole("heading", { name: "Qi Blast" })).toBeInTheDocument();
    expect(screen.getAllByText(/Electricity/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Toggleable")).toBeInTheDocument();
    expect(screen.getByText("Yes")).toBeInTheDocument();
    expect(screen.getByText("fire-key")).toBeInTheDocument();
    expect(document.querySelector("img[src]")).toBeNull();
    expect(screen.queryByText(/license/i)).not.toBeInTheDocument();
  });

  it("keeps Heal's opaque backend form order and exposes each authored patch", () => {
    render(<RecordSurface onReference={vi.fn()} surface={healForms()} />);

    const harmful = screen.getByText("Harmful font");
    const healing = screen.getByText("Healing font");
    expect(
      harmful.compareDocumentPosition(healing) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();

    fireEvent.click(harmful);
    expect(screen.getAllByText("2d8 void").length).toBeGreaterThanOrEqual(1);
    fireEvent.click(healing);
    expect(screen.getAllByText("2d8 vitality").length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText("opaque:heal:harm")).not.toBeInTheDocument();
  });

  it("renders complete typed fixed-patch groups without applying them in the client", () => {
    render(<RecordSurface onReference={vi.fn()} surface={peacefulRest()} />);

    fireEvent.click(screen.getByText("Rank 5"));
    expect(screen.getByText("3 actions")).toBeInTheDocument();
    expect(screen.getByText("concentrate and manipulate")).toBeInTheDocument();
    expect(screen.getByText("20")).toBeInTheDocument();
    expect(screen.getAllByText("Burst").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Damage").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("Silver")).toHaveLength(2);
    expect(screen.getAllByText("Apply modifier")).toHaveLength(2);
  });

  it("renders Rime rank 5 and cumulative rank 8 returned definitions", () => {
    const rank5 = rimeSelected(5, [5], "4d4");
    const { rerender } = render(
      <RecordSurface
        onReference={vi.fn()}
        spellCatalog={rank5}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 5 }}
        surface={rank5}
      />,
    );

    expect(screen.getByText("Selected form at 5th rank")).toBeInTheDocument();
    expect(screen.getByText("4d4")).toBeInTheDocument();
    expect(screen.getByText("5th")).toBeInTheDocument();

    const rank8 = rimeSelected(8, [5, 8], "6d4");
    rerender(
      <RecordSurface
        onReference={vi.fn()}
        spellCatalog={rank5}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 8 }}
        surface={rank8}
      />,
    );
    expect(screen.getByText("Selected form at 8th rank")).toBeInTheDocument();
    expect(screen.getByText("6d4")).toBeInTheDocument();
    expect(screen.getByText("5th, 8th")).toBeInTheDocument();
  });

  it("shows whole-form and field-local unavailable results without reconstructing them", () => {
    const whole = rimeSelected(1, [], "2d4", {
      state: "unavailable",
      reason: { reason: "cast_rank_below_base", base_rank: 2, cast_rank: 1 },
    });
    if (whole.presentation.presentation_type !== "spell")
      throw new Error("spell fixture");
    whole.presentation.body.form_catalog_unavailable = "unsupported_overlay_root";
    const { rerender } = render(
      <RecordSurface
        onReference={vi.fn()}
        spellCatalog={whole}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 1 }}
        surface={whole}
      />,
    );
    expect(
      screen.getByText("Form unavailable: Cast Rank Below Base"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Form catalog unavailable: Unsupported Overlay Root"),
    ).toBeInTheDocument();
    expect(screen.getByText("Base rank 2; selected rank 1.")).toBeInTheDocument();

    const localized = rimeSelected(8, [5, 8], "6d4", undefined, true);
    rerender(
      <RecordSurface
        onReference={vi.fn()}
        spellCatalog={localized}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 8 }}
        surface={localized}
      />,
    );
    expect(
      screen.getByText("Damage unavailable: Fixed Rank Key Mismatch"),
    ).toBeInTheDocument();
    expect(screen.getByText("Fixed Heightening patch")).toBeInTheDocument();
    expect(
      screen.getAllByRole("heading", { name: "Targeting" }).length,
    ).toBeGreaterThanOrEqual(1);
  });
});

function deityStrike(): RecordSurfaceView {
  const definition = plainSpellDefinition();
  definition.heightening = missing;
  return spellSurface("Deity's Strike", definition, [baseForm("opaque:deity:base")]);
}

function peacefulRest(): RecordSurfaceView {
  const definition = baseDefinition();
  definition.ritual = missing;
  definition.rules = missing;
  definition.heightening = known({
    kind: "fixed" as const,
    layers: [
      {
        order: 0,
        rank: known(5),
        patch: {
          ...emptyPatch(),
          casting: known({
            time: known("3 actions"),
            cost: missing,
            requirements: known("concentrate and manipulate"),
            counteraction: known(false),
          }),
          targeting: known({
            target: missing,
            range: missing,
            area: known({
              value: known(20),
              area_type: known("burst"),
              legacy_area_type: known("burst"),
              details: known("spreads around corners"),
            }),
          }),
          damage: known({
            members: [
              {
                key: "fire-key",
                order: 0,
                operation: {
                  operation: "merge",
                  value: {
                    formula: known("4d6"),
                    damage_type: known("fire"),
                    category: missing,
                    kinds: known(["damage"]),
                    materials: known(["silver"]),
                    apply_modifier: known(false),
                  },
                },
              },
            ],
          }),
        },
      },
    ],
  });
  return spellSurface("Peaceful Rest", definition, [baseForm("opaque:peaceful:base")]);
}

function planarRitual(): RecordSurfaceView {
  const definition = baseDefinition();
  definition.rules = missing;
  definition.heightening = missing;
  return spellSurface("Planar Displacement", definition, [
    baseForm("opaque:planar:base"),
  ]);
}

function qiBlast(): RecordSurfaceView {
  const definition = baseDefinition();
  definition.ritual = missing;
  definition.heightening = missing;
  return spellSurface("Qi Blast", definition, [baseForm("opaque:qi:base")]);
}

function healForms(): RecordSurfaceView {
  const harmful = overlayForm("opaque:heal:harm", "Harmful font", "2d8 void", 20);
  const healing = overlayForm("opaque:heal:heal", "Healing font", "2d8 vitality", 10);
  return spellSurface("Heal", plainSpellDefinition(), [harmful, healing]);
}

function rimeSelected(
  castRank: number,
  appliedRanks: number[],
  formula: string,
  resultOverride?: SpellFormResultView,
  localizedDamageUnavailable = false,
): RecordSurfaceView {
  const result =
    resultOverride ??
    ({
      state: "available",
      definition: resolvedDefinition(formula, appliedRanks, localizedDamageUnavailable),
    } satisfies SpellFormResultView);
  const surface = spellSurface("Rime Slick", plainSpellDefinition(), [
    baseForm("opaque:rime:base"),
  ]);
  if (surface.presentation.presentation_type !== "spell")
    throw new Error("spell fixture");
  surface.presentation.body.selected_form = {
    id: "opaque:rime:base",
    cast_rank: castRank,
    result,
  };
  return surface;
}

function spellSurface(
  title: string,
  definition: SpellDefinitionSurfaceView,
  forms: SpellFormView[],
): RecordSurfaceView {
  return {
    metadata: {
      record_key: `spells:${title.toLowerCase().replace(/ /g, "-")}`,
      title,
      kind: definition.ritual.state === "known" ? "ritual" : "spell",
      kind_label: definition.ritual.state === "known" ? "Ritual" : "Spell",
      level: 2,
      source: {
        publication_title: "Pathfinder Core",
        pack_label: "Spells",
        document_type: "Item",
        record_type: "spell",
      },
    },
    profile: "record_detail",
    presentation: {
      presentation_type: "spell",
      body: {
        definition,
        forms,
        content: [
          {
            content_key: "description",
            role: "primary_description",
            authored_order: 0,
            blocks: [
              {
                block_type: "paragraph",
                spans: [
                  { span_type: "text", text: "Attempt a " },
                  {
                    span_type: "check",
                    display: "DC 30 Religion check",
                    statistic: "religion",
                    difficulty_class: 30,
                  },
                  { span_type: "text", text: " See " },
                  {
                    span_type: "reference",
                    label: "Alarm",
                    record_key: "spells-srd:alarm",
                    embedded: false,
                  },
                  { span_type: "text", text: "." },
                ],
              },
            ],
            content_hash: "spell-description",
            visibility: "gm_only",
            provenance: {
              source_record_key: "spells:fixture",
              relative_source_path: "fixture.json",
              field_family: "description",
            },
          },
        ],
      },
    },
  };
}

function baseDefinition(): SpellDefinitionSurfaceView {
  return {
    classification: known({
      rank: known(2),
      traits: known(["attack", "concentrate"]),
      traditions: known(["arcane", "divine"]),
    }),
    casting: known({
      time: known("2 actions"),
      cost: known("one silver bell"),
      requirements: known("wielding a deity's favored weapon"),
      counteraction: known(false),
    }),
    targeting: known({
      target: known("1 creature"),
      range: known({ authored_text: "120 feet (emanation)" }),
      area: known({
        value: known(10),
        area_type: known("emanation"),
        legacy_area_type: missing,
        details: known("centered on you"),
      }),
    }),
    defense: known({
      passive: known("Armor Class"),
      save: known({ statistic: known("Reflex"), basic: known(false) }),
    }),
    damage: known([
      {
        key: "fire-key",
        order: 0,
        formula: known("2d6"),
        damage_type: known("fire"),
        category: known("persistent"),
        kinds: known(["damage"]),
        materials: known(["silver"]),
        apply_modifier: known(false),
      },
    ]),
    duration: known({ value: known("1 minute"), sustained: known(false) }),
    heightening: known({
      kind: "interval",
      interval: known(1),
      area: known(5),
      damage: known([{ key: "fire-key", order: 0, value: "1d6" }]),
    }),
    ritual: known({
      primary_check: known("Religion (master)"),
      secondary_casters: known(2),
      secondary_checks: known("Arcana or Occultism"),
    }),
    rules: known([
      {
        order: 0,
        rule: {
          kind: "roll_option",
          value: {
            domain: known("damage"),
            label: known("Energy type"),
            option: known("electricity"),
            placement: known("options"),
            predicate: known([{ kind: "term", value: "self:effect:qi-blast" }]),
            suboptions: known([
              { label: known("Electricity"), value: known("electricity") },
            ]),
            toggleable: known(true),
          },
        },
      },
    ]),
  };
}

function plainSpellDefinition(): SpellDefinitionSurfaceView {
  const definition = baseDefinition();
  definition.ritual = missing;
  definition.rules = missing;
  return definition;
}

function emptyPatch(): SpellPatchView {
  return {
    classification: missing,
    casting: missing,
    targeting: missing,
    defense: missing,
    damage: missing,
    duration: missing,
    heightening: missing,
    rules: missing,
  };
}

function baseForm(id: string): SpellFormView {
  return {
    id,
    label: "Base spell",
    order: 0,
    cast_rank: 2,
    kind: "base" as const,
    result: {
      state: "available" as const,
      definition: resolvedDefinition("2d4", []),
    },
  };
}

function overlayForm(
  id: string,
  label: string,
  formula: string,
  order: number,
): SpellFormView {
  const patch = {
    ...emptyPatch(),
    damage: known({
      members: [
        {
          key: "main",
          order: 0,
          operation: {
            operation: "merge" as const,
            value: {
              formula: known(formula),
              damage_type: missing,
              category: missing,
              kinds: missing,
              materials: missing,
              apply_modifier: missing,
            },
          },
        },
      ],
    }),
  };
  return {
    id,
    label,
    order,
    cast_rank: 2,
    kind: "overlay" as const,
    authored_patch: patch,
    result: {
      state: "available" as const,
      definition: resolvedDefinition(formula, []),
    },
  };
}

function resolvedDefinition(
  formula: string,
  appliedFixedRanks: number[],
  damageUnavailable = false,
): SpellResolvedDefinitionView {
  const definition = baseDefinition();
  return {
    applied_fixed_ranks: appliedFixedRanks,
    classification: { state: "available", value: definition.classification },
    casting: { state: "available", value: definition.casting },
    targeting: { state: "available", value: definition.targeting },
    defense: { state: "available", value: definition.defense },
    damage: damageUnavailable
      ? {
          state: "unavailable",
          unavailable: {
            field: "damage",
            source: "fixed_heightening",
            reason: "fixed_rank_key_mismatch",
          },
        }
      : {
          state: "available",
          value: known([
            {
              key: "cold",
              order: 0,
              formula: known(formula),
              damage_type: known("cold"),
              category: missing,
              kinds: known(["damage"]),
              materials: known([]),
              apply_modifier: known(false),
            },
          ]),
        },
    duration: { state: "available", value: definition.duration },
    heightening: { state: "available", value: definition.heightening },
    rules: { state: "available", value: definition.rules },
  };
}
