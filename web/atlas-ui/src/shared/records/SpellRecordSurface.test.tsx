import { fireEvent, render, screen, within } from "@testing-library/react";
import type {
  RecordSurfaceView,
  SpellAreaView,
  SpellFormView,
  SpellFactView,
  SpellFormResultView,
  SpellResolvedDefinitionView,
  SpellRuleView,
} from "../../generated/atlas";

type AvailableValue<T> = T extends { state: "available"; value: infer V } ? V : never;
type ResolvedSpellField = Exclude<
  keyof SpellResolvedDefinitionView,
  "applied_fixed_ranks" | "ritual"
>;
type SpellDefinitionFixture = {
  [K in ResolvedSpellField]: AvailableValue<SpellResolvedDefinitionView[K]>;
} & Pick<SpellResolvedDefinitionView, "ritual">;
import { RecordSurface } from "./RecordSurface";

const missing = { state: "missing" as const };
const known = <T,>(value: T) => ({ state: "known" as const, value });

describe("SpellRecordSurface", () => {
  it.each([["damage"], ["healing"], ["damage", "healing"]])(
    "omits only the damage-only kind already in the heading: %j",
    (...kinds) => {
      const definition = plainSpellDefinition();
      definition.damage = known([
        {
          label: kinds.includes("healing") ? "Healing" : "Damage",
          formula: known("1d8"),
          damage_type: known("vitality"),
          category: missing,
          kinds: known(kinds),
          materials: missing,
          apply_modifier: known(false),
        },
      ]);
      const surface = spellSurface("Test", definition, [baseForm("base")]);
      const before = JSON.stringify(surface);
      render(<RecordSurface surface={surface} onReference={vi.fn()} />);
      if (kinds.length === 1 && kinds[0] === "damage")
        expect(screen.queryByText("Effect type")).not.toBeInTheDocument();
      else expect(screen.getByText("Effect type")).toBeVisible();
      expect(JSON.stringify(surface)).toBe(before);
    },
  );

  it.each<SpellFactView<boolean>>([
    known(true),
    known(false),
    missing,
    { state: "null" },
    { state: "unsupported" },
  ])(
    "renders only a known-true counteract note, preserving optional states: %j",
    (counteraction) => {
      const definition = plainSpellDefinition();
      definition.casting = known({
        time: known("2"),
        cost: missing,
        requirements: missing,
        counteraction,
      });
      const surface = spellSurface("Test", definition, [baseForm("base")]);
      if (counteraction.state === "unsupported")
        surface.issues = [
          {
            code: "unsupported",
            placement: "rules",
            message: "Counteract field unavailable",
          },
        ];
      const before = JSON.stringify(surface);
      render(<RecordSurface surface={surface} onReference={vi.fn()} />);
      expect(screen.queryAllByText("Uses a counteract check")).toHaveLength(
        counteraction.state === "known" && counteraction.value ? 1 : 0,
      );
      expect(screen.queryByText("Counteraction")).not.toBeInTheDocument();
      if (counteraction.state === "unsupported")
        expect(screen.getByText("Counteract field unavailable")).toBeVisible();
      expect(JSON.stringify(surface)).toBe(before);
    },
  );

  it("updates only returned Blazing description damage and heading rank in place", () => {
    const selected = (rank: number, dice: string) => {
      const surface = fireball();
      if (surface.presentation.presentation_type !== "spell") throw new Error("spell");
      surface.metadata.title = "Blazing Blade";
      surface.presentation.body.effective_form.cast_rank = rank;
      const description = surface.presentation.body.content?.[0];
      if (!description) throw new Error("description fixture");
      description.blocks = [
        {
          block_type: "paragraph",
          spans: [
            { span_type: "text", text: "The target takes " },
            { span_type: "text", text: `${dice} persistent spirit` },
            { span_type: "text", text: " damage." },
          ],
        },
      ];
      return surface;
    };
    const { rerender } = render(
      <RecordSurface surface={selected(2, "1d6")} onReference={vi.fn()} />,
    );
    const description = screen.getByRole("heading", { name: "Description" });
    for (const [rank, dice] of [
      [4, "1d6"],
      [6, "2d6"],
      [8, "3d6"],
    ] as const) {
      rerender(<RecordSurface surface={selected(rank, dice)} onReference={vi.fn()} />);
      expect(
        screen.getByText(`The target takes ${dice} persistent spirit damage.`),
      ).toBeVisible();
      expect(screen.getByText(`Rank ${rank}`)).toBeVisible();
      expect(screen.getByRole("heading", { name: "Description" })).toBe(description);
      expect(screen.queryByText(/ternary|@item.level/)).not.toBeInTheDocument();
    }
  });

  it("uses a true-only modifier note in fixed-heightening component summaries", () => {
    const definition = rimeDefinition();
    if (
      definition.heightening.state !== "known" ||
      definition.heightening.value.kind !== "fixed"
    )
      throw new Error("fixed fixture");
    const change = definition.heightening.value.layers[0].changes.find(
      (change) => change.field === "effect",
    );
    if (!change || change.field !== "effect" || !change.value)
      throw new Error("effect fixture");
    change.value.apply_modifier = known(true);
    render(
      <RecordSurface
        surface={spellSurface("Rime", definition, [baseForm("opaque:rime:base")])}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getAllByText(/your spellcasting ability modifier/)).toHaveLength(1);
    expect(screen.queryByText(/apply modifier (Yes|No)/)).not.toBeInTheDocument();
  });

  it("enables Apply only for a valid changed tuple and permits a newer pending tuple", () => {
    const surface = fireball();
    const onSelect = vi.fn();
    const { rerender } = render(
      <RecordSurface
        surface={surface}
        onReference={vi.fn()}
        onSpellFormSelection={onSelect}
      />,
    );
    const apply = screen.getByRole("button", { name: "Apply" });
    const input = screen.getByRole("spinbutton", { name: "Cast rank" });
    expect(apply).toBeDisabled();
    fireEvent.change(input, { target: { value: "" } });
    expect(apply).toBeDisabled();
    fireEvent.change(input, { target: { value: "4" } });
    expect(apply).toBeEnabled();
    fireEvent.click(apply);
    expect(onSelect).toHaveBeenLastCalledWith({
      formId: "opaque:fireball:base",
      castRank: 4,
    });
    rerender(
      <RecordSurface
        surface={surface}
        onReference={vi.fn()}
        onSpellFormSelection={onSelect}
        spellFormSelection={{ formId: "opaque:fireball:base", castRank: 4 }}
        spellFormSelectionLoading
      />,
    );
    expect(apply).toBeDisabled();
    fireEvent.change(input, { target: { value: "3" } });
    expect(apply).toBeEnabled();
    fireEvent.change(input, { target: { value: "5" } });
    expect(apply).toBeEnabled();
  });

  it("puts authored description before one set of mechanics and one action label", () => {
    const surface = fireball();
    if (surface.presentation.presentation_type !== "spell")
      throw new Error("spell fixture");
    const result = surface.presentation.body.effective_form.result;
    if (
      result.state !== "available" ||
      result.definition.casting.state !== "available" ||
      result.definition.casting.value.state !== "known"
    )
      throw new Error("casting fixture");
    result.definition.casting.value.value.action_cost = {
      cost_type: "actions",
      count: 2,
    };
    render(<RecordSurface surface={surface} onReference={vi.fn()} />);
    const description = screen.getByRole("heading", { name: "Description" });
    const mechanics = screen.getByRole("region", { name: "Spell mechanics" });
    expect(
      description.compareDocumentPosition(mechanics) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(screen.getAllByText("Two actions")).toHaveLength(1);
    expect(screen.getAllByText("500 feet")).toHaveLength(1);
    expect(screen.getAllByText("6d6 Fire")).toHaveLength(1);
    expect(screen.queryByLabelText("Spell quick facts")).not.toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Casting" })).not.toBeInTheDocument();
  });

  it("resets a modified returned tuple to the typed base without changing authored rank", () => {
    const surface = rimeSelected(8, [5, 8], "14d4", 60);
    const onSelect = vi.fn();
    const { rerender } = render(
      <RecordSurface
        surface={surface}
        spellCatalog={surface}
        onReference={vi.fn()}
        onSpellFormSelection={onSelect}
      />,
    );
    expect(screen.queryByText(/^Applied rank \d+$/)).not.toBeInTheDocument();
    expect(screen.getByText("Modified from default")).toBeVisible();
    expect(screen.getByText("Rank 8")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(onSelect).toHaveBeenCalledWith({ formId: "opaque:rime:base", castRank: 2 });
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toHaveValue("2");
    expect(screen.queryByText(/^Applied rank \d+$/)).not.toBeInTheDocument();
    const base = rimeSelected(2, [], "2d4", 15);
    rerender(
      <RecordSurface
        surface={base}
        spellCatalog={base}
        onReference={vi.fn()}
        onSpellFormSelection={onSelect}
      />,
    );
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toHaveValue("2");
    expect(screen.queryByRole("button", { name: "Reset" })).not.toBeInTheDocument();
    expect(screen.queryByText(/Showing Base/)).not.toBeInTheDocument();
    expect(screen.queryByText("Modified from default")).not.toBeInTheDocument();
  });

  it("shows typed gameplay qualifiers without an Effect details disclosure", () => {
    const definition = plainSpellDefinition();
    definition.damage = known([
      {
        label: "Damage",
        formula: known("2d6"),
        damage_type: known("fire"),
        category: known("persistent"),
        kinds: known(["damage"]),
        materials: known(["silver"]),
        apply_modifier: known(false),
      },
    ]);
    render(
      <RecordSurface
        onReference={vi.fn()}
        surface={spellSurface("Test", definition, [baseForm("base")])}
      />,
    );
    const qualifiers = screen.getByLabelText("Damage qualifiers");
    expect(qualifiers).toHaveTextContent("Damage categoryPersistent");
    expect(qualifiers).not.toHaveTextContent("Effect type");
    expect(qualifiers).toHaveTextContent("Damage materialsSilver");
    expect(qualifiers).not.toHaveTextContent("Spellcasting ability modifier");
    expect(
      screen.queryByText("+ your spellcasting ability modifier"),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Effect details")).not.toBeInTheDocument();
  });

  it.each<SpellFactView<boolean>>([
    known(true),
    known(false),
    missing,
    { state: "null" },
    { state: "unsupported" },
  ])("shows a modifier note only for known true: %j", (applyModifier) => {
    const definition = plainSpellDefinition();
    definition.damage = known([
      {
        label: "Damage",
        formula: known("2d6"),
        damage_type: known("fire"),
        category: missing,
        kinds: known(["damage"]),
        materials: missing,
        apply_modifier: applyModifier,
      },
    ]);
    render(
      <RecordSurface
        onReference={vi.fn()}
        surface={spellSurface("Test", definition, [baseForm("base")])}
      />,
    );
    const note = screen.queryByText("+ your spellcasting ability modifier");
    if (applyModifier.state === "known" && applyModifier.value)
      expect(note).toBeVisible();
    else expect(note).not.toBeInTheDocument();
    expect(screen.getByText("2d6 Fire")).toBeVisible();
  });

  it("omits sole-form and idle helper text while keeping controls through pending/error", () => {
    const surface = fireball();
    const { rerender } = render(
      <RecordSurface
        surface={surface}
        onReference={vi.fn()}
        onSpellFormSelection={vi.fn()}
      />,
    );
    const controls = screen.getByLabelText("Resolve spell form");
    const rank = screen.getByRole("spinbutton", { name: "Cast rank" });
    expect(within(controls).queryByText("Form")).not.toBeInTheDocument();
    expect(within(controls).queryByText("Base")).not.toBeInTheDocument();
    expect(screen.getByRole("status")).toBeEmptyDOMElement();
    expect(screen.queryByText(/^Applied rank \d+$/)).not.toBeInTheDocument();
    rerender(
      <RecordSurface
        surface={surface}
        onReference={vi.fn()}
        onSpellFormSelection={vi.fn()}
        spellFormSelection={{ formId: "opaque:fireball:base", castRank: 5 }}
        spellFormSelectionLoading
      />,
    );
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toBe(rank);
    expect(screen.getByRole("status")).toHaveTextContent("Resolving");
    rerender(
      <RecordSurface
        surface={surface}
        onReference={vi.fn()}
        onSpellFormSelection={vi.fn()}
        spellFormSelectionError="Try again."
      />,
    );
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toBe(rank);
    expect(screen.getByRole("status")).toHaveTextContent(
      "Unable to resolve the selected form. Try again.",
    );
  });

  it("omits an authored empty range while preserving the area once", () => {
    const definition = plainSpellDefinition();
    definition.targeting = known({
      range: known({ authored_text: "" }),
      area: known({
        area_type: known("emanation"),
        legacy_area_type: missing,
        value: known(30),
        details: missing,
      }),
      target: missing,
    });
    render(
      <RecordSurface
        onReference={vi.fn()}
        surface={spellSurface("Heal", definition, [baseForm("base")])}
      />,
    );
    const summary = screen.getByLabelText("Spell range and targets");
    expect(within(summary).getByText(/30-foot emanation/)).toBeInTheDocument();
    expect(summary).not.toHaveTextContent(/Range/);
  });

  it("renders common outer issues and both reference directions without changing the effective spell", () => {
    const surface = rimeSelected(5, [5], "8d4", 30);
    surface.issues = [
      {
        code: "unavailable",
        placement: "damage",
        message: "One secondary damage member is unavailable.",
      },
    ];
    surface.references = {
      outgoing: {
        state: "available",
        requested_limit: 8,
        records: [{ record_key: "spells:fireball", title: "Fireball", kind: "spell" }],
        edges: [
          {
            from_record_key: "spells:rime-slick",
            to_record_key: "spells:fireball",
            display_text: "Fireball",
            reference_text: "Fireball",
            source: {
              kind: "rich_content",
              visibility: "public",
              relation_kind: "references",
            },
          },
        ],
        total_records: 1,
        total_edges: 1,
        truncated: false,
      },
      backlinks: {
        state: "available",
        requested_limit: 8,
        records: [
          {
            record_key: "creatures:winter-wizard",
            title: "Winter Wizard",
            kind: "creature",
          },
        ],
        edges: [
          {
            from_record_key: "creatures:winter-wizard",
            to_record_key: "spells:rime-slick",
            display_text: "Rime Slick",
            reference_text: "Rime Slick",
            source: {
              kind: "rich_content",
              visibility: "public",
              relation_kind: "references",
            },
          },
        ],
        total_records: 1,
        total_edges: 1,
        truncated: false,
      },
    };
    const onReference = vi.fn();
    const onReferencesOpen = vi.fn();

    render(
      <RecordSurface
        onReference={onReference}
        onReferencesOpen={onReferencesOpen}
        spellCatalog={surface}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 5 }}
        surface={surface}
      />,
    );

    expect(screen.getByText("8d4 Cold")).toBeInTheDocument();
    expect(
      screen.getAllByText("One secondary damage member is unavailable."),
    ).toHaveLength(1);
    fireEvent.click(screen.getByText("References", { exact: true }));
    expect(onReferencesOpen).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("heading", { name: "Referenced by" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Fireball" }));
    expect(onReference).toHaveBeenLastCalledWith("spells:fireball");
    fireEvent.click(screen.getByRole("button", { name: "Winter Wizard" }));
    expect(onReference).toHaveBeenLastCalledWith("creatures:winter-wizard");
    expect(screen.queryByText("opaque:rime:base")).toBeNull();
  });

  it("renders Deity's Strike defenses, authored range, rich checks, Qi rules, and Planar ritual facts", () => {
    const onReference = vi.fn();
    const { rerender } = render(
      <RecordSurface onReference={onReference} surface={deityStrike()} />,
    );

    expect(screen.getByRole("heading", { name: "Deity's Strike" })).toBeInTheDocument();
    expect(screen.getByText("120 feet (emanation)")).toBeInTheDocument();
    expect(screen.getByText("10-foot emanation — centered on you")).toBeInTheDocument();
    expect(screen.queryByText("120-foot query range")).not.toBeInTheDocument();
    expect(screen.getByText("Armor Class")).toBeInTheDocument();
    expect(screen.getByText("Reflex")).toBeInTheDocument();
    expect(screen.getAllByText("No")).toHaveLength(1);
    expect(screen.queryByText("Counteraction")).not.toBeInTheDocument();
    expect(screen.getByText("DC 30 Religion check")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Alarm" }));
    expect(onReference).toHaveBeenCalledWith("spells-srd:alarm");

    rerender(<RecordSurface onReference={vi.fn()} surface={planarRitual()} />);
    expect(
      screen.getByRole("heading", { name: "Planar Displacement" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Spell casting")).toHaveTextContent("Primary check");
    expect(screen.getByText("Religion (master)")).toBeInTheDocument();
    expect(screen.getByText("Arcana or Occultism")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Effect" })).not.toBeInTheDocument();
    expect(screen.queryByText("Duration", { exact: true })).not.toBeInTheDocument();

    rerender(<RecordSurface onReference={vi.fn()} surface={qiBlast()} />);
    expect(screen.getByRole("heading", { name: "Qi Blast" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Rule effects/ }));
    expect(screen.getAllByText(/Electricity/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Toggleable")).toBeInTheDocument();
    expect(screen.getByText("Yes")).toBeInTheDocument();
    expect(screen.queryByText("fire-key")).not.toBeInTheDocument();
    expect(document.querySelector("img[src]")).toBeNull();
    expect(screen.queryByText(/license/i)).not.toBeInTheDocument();
  });

  it("renders an unsupported rule issue once without hiding supported neighboring facts", () => {
    const surface = qiBlast();
    if (surface.presentation.presentation_type !== "spell") {
      throw new Error("spell fixture");
    }
    const result = surface.presentation.body.effective_form.result;
    if (
      result.state !== "available" ||
      result.definition.rules.state !== "available" ||
      result.definition.rules.value.state !== "known"
    ) {
      throw new Error("spell rule fixture");
    }
    result.definition.rules.value.value.push({
      order: 1,
      rule: { kind: "unsupported" },
    });
    surface.issues = [
      {
        code: "unsupported",
        placement: "rules",
        message: "Unsupported rule is not supported for presentation.",
      },
    ];

    render(<RecordSurface onReference={vi.fn()} surface={surface} />);

    expect(
      screen.getAllByText("Unsupported rule is not supported for presentation."),
    ).toHaveLength(1);
    expect(screen.queryByText("Rule details unavailable")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Rule effects/ }));
    expect(screen.getByText("Energy type")).toBeInTheDocument();
    expect(screen.getAllByText(/Electricity/).length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText("unsupported_rule_payload")).not.toBeInTheDocument();
  });

  it("keeps optional area details quiet and unsupported details issue-owned", () => {
    const missingDetails = deityStrike();
    const area = resolvedArea(missingDetails);
    area.details = missing;
    const { rerender } = render(
      <RecordSurface onReference={vi.fn()} surface={missingDetails} />,
    );
    expect(screen.getByText("10-foot emanation")).toBeInTheDocument();
    expect(screen.queryByText("centered on you")).not.toBeInTheDocument();

    const nullDetails = deityStrike();
    resolvedArea(nullDetails).details = { state: "null" };
    rerender(<RecordSurface onReference={vi.fn()} surface={nullDetails} />);
    expect(screen.getByText("10-foot emanation")).toBeInTheDocument();

    const emptyDetails = deityStrike();
    resolvedArea(emptyDetails).details = known("");
    rerender(<RecordSurface onReference={vi.fn()} surface={emptyDetails} />);
    expect(screen.getByText("10-foot emanation")).toBeInTheDocument();

    const unsupportedDetails = deityStrike();
    resolvedArea(unsupportedDetails).details = { state: "unsupported" };
    unsupportedDetails.issues = [
      {
        code: "unsupported",
        placement: "targeting",
        message: "Area details are unavailable.",
      },
    ];
    rerender(<RecordSurface onReference={vi.fn()} surface={unsupportedDetails} />);
    expect(screen.getByText("10-foot emanation")).toBeInTheDocument();
    expect(screen.getAllByText("Area details are unavailable.")).toHaveLength(1);
    expect(screen.queryByText("Details", { exact: true })).not.toBeInTheDocument();
    expect(screen.queryByText("Unsupported", { exact: true })).not.toBeInTheDocument();
  });

  it("retains a valid draft rank across forms and explains a required reset", async () => {
    const surface = healForms();
    if (surface.presentation.presentation_type !== "spell") throw new Error("fixture");
    const forms = surface.presentation.body.forms;
    const living = forms.find((form) => form.label === "Living creature")!;
    living.minimum_cast_rank = 4;
    const select = vi.fn();
    render(
      <RecordSurface
        onReference={vi.fn()}
        onSpellFormSelection={select}
        surface={surface}
      />,
    );
    fireEvent.change(screen.getByRole("spinbutton", { name: "Cast rank" }), {
      target: { value: "5" },
    });
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Spell form" }));
    fireEvent.click(await screen.findByRole("option", { name: "Living creature" }));
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toHaveValue("5");
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(select).toHaveBeenLastCalledWith({ formId: living.id, castRank: 5 });
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Spell form" }));
    fireEvent.click(await screen.findByRole("option", { name: "Undead creature" }));
    fireEvent.change(screen.getByRole("spinbutton", { name: "Cast rank" }), {
      target: { value: "2" },
    });
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Spell form" }));
    fireEvent.click(await screen.findByRole("option", { name: "Living creature" }));
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toHaveValue("4");
    expect(screen.getByRole("status")).toHaveTextContent("Cast rank reset to 4");
  });

  it("keeps Heal's compact backend form order without exposing authored patches", async () => {
    render(
      <RecordSurface
        onReference={vi.fn()}
        onSpellFormSelection={vi.fn()}
        surface={healForms()}
      />,
    );

    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Spell form" }));
    await screen.findByRole("listbox");
    const touch = screen.getByRole("option", { name: "1 action — touch" });
    const living = screen.getByRole("option", { name: "Living creature" });
    const undead = screen.getByRole("option", { name: "Undead creature" });
    const emanation = screen.getByRole("option", {
      name: "3 actions — 30-foot emanation",
    });
    expect(
      touch.compareDocumentPosition(living) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      living.compareDocumentPosition(undead) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      undead.compareDocumentPosition(emanation) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();

    expect(screen.queryByText("Authored patch")).not.toBeInTheDocument();
    expect(screen.queryByText("Catalog result")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("option", { name: "opaque:heal:harm" }),
    ).not.toBeInTheDocument();
  });

  it("keeps fixed heightening compact without exposing patch internals", () => {
    render(<RecordSurface onReference={vi.fn()} surface={peacefulRest()} />);

    expect(screen.getByText("At rank 5")).toBeInTheDocument();
    expect(screen.queryByText("fire-key")).not.toBeInTheDocument();
    expect(screen.queryByText("Authored patch")).not.toBeInTheDocument();
  });

  it("renders Rime rank 5 and cumulative rank 8 returned definitions", () => {
    const base = rimeSelected(2, [], "2d4", 15);
    const rank5 = rimeSelected(5, [5], "8d4", 30);
    const onReference = vi.fn();
    base.references = proneReference();
    const { rerender } = render(
      <RecordSurface
        onReference={onReference}
        onReferencesOpen={vi.fn()}
        onSpellFormSelection={vi.fn()}
        spellCatalog={base}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 2 }}
        surface={base}
      />,
    );

    expect(screen.getByText("2d4 Cold")).toBeInTheDocument();
    expect(screen.getByText("15-foot burst")).toBeInTheDocument();
    expect(screen.queryByText(/Applied ranks/)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "References" }));
    fireEvent.click(screen.getByRole("button", { name: "Prone" }));
    expect(onReference).toHaveBeenCalledWith("conditions:prone");

    rerender(
      <RecordSurface
        onReference={vi.fn()}
        spellCatalog={rank5}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 5 }}
        surface={rank5}
      />,
    );

    expect(screen.getByText("8d4 Cold")).toBeInTheDocument();
    expect(screen.getByText("30-foot burst")).toBeInTheDocument();
    expect(screen.getByText("Applied ranks 5th")).toBeInTheDocument();

    const rank8 = rimeSelected(8, [5, 8], "14d4", 60);
    rerender(
      <RecordSurface
        onReference={vi.fn()}
        spellCatalog={rank5}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 8 }}
        surface={rank8}
      />,
    );
    expect(screen.getByText("14d4 Cold")).toBeInTheDocument();
    expect(screen.getByText("60-foot burst")).toBeInTheDocument();
    expect(screen.getByText("Applied ranks 5th, 8th")).toBeInTheDocument();
  });

  it("renders Fireball semantic damage and heightening without exposing authored key 0", () => {
    render(<RecordSurface onReference={vi.fn()} surface={fireball()} />);

    expect(screen.getByText("Rank 3")).toBeInTheDocument();
    expect(screen.getByText("2 actions")).toBeInTheDocument();
    expect(screen.getByText("500 feet")).toBeInTheDocument();
    expect(screen.getByText("20-foot burst")).toBeInTheDocument();
    expect(screen.getByText("Basic Reflex")).toBeInTheDocument();
    expect(screen.getByText("6d6 Fire")).toBeInTheDocument();
    expect(screen.getByText("Heightened (+1)")).toBeInTheDocument();
    expect(screen.getByText("Fire damage increases by 2d6")).toBeInTheDocument();
    expect(screen.queryByText(/^0$/)).not.toBeInTheDocument();
    expect(screen.queryByText(/^0:/)).not.toBeInTheDocument();
    expect(screen.queryByText(/Level 99/)).not.toBeInTheDocument();
    expect(screen.queryByText("Legacy type")).not.toBeInTheDocument();
    expect(screen.queryByText("Details")).not.toBeInTheDocument();
    expect(screen.queryByText("Duration", { exact: true })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Source & provenance/ }));
    expect(screen.getByText("Pathfinder Core")).toBeInTheDocument();
    expect(screen.queryByText("spells:fireball")).not.toBeInTheDocument();
    expect(screen.queryByText("Record ID")).not.toBeInTheDocument();
    expect(screen.queryByText("Source path")).not.toBeInTheDocument();
  });

  it("keeps all five supported rule variants readable without internal identities or ordinals", () => {
    render(<RecordSurface onReference={vi.fn()} surface={allRuleVariants()} />);

    fireEvent.click(screen.getByRole("button", { name: /Rule effects/ }));
    for (const heading of [
      "Damage dice",
      "Ephemeral effect",
      "Damage alteration",
      "Roll option",
      "Item alteration",
    ]) {
      expect(screen.getByText(new RegExp(`^${heading}$`, "i"))).toBeInTheDocument();
    }
    expect(screen.getByText("Selected damage scope")).toBeInTheDocument();
    expect(screen.getAllByText(/typed condition/).length).toBeGreaterThanOrEqual(5);
    expect(screen.getByText("Typed effect reference retained")).toBeInTheDocument();
    expect(screen.getByText("Typed effect identifier retained")).toBeInTheDocument();
    expect(screen.getByText("Typed item reference retained")).toBeInTheDocument();
    expect(screen.getAllByText("Electricity").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Toggleable")).toBeInTheDocument();
    for (const internal of [
      "damage-key-0",
      "Compendium.pf2e.spell-effects.Item.private-effect",
      "private-alteration-slug",
      "private-item-id",
      "Rule 1",
      "Rule 2",
    ]) {
      expect(screen.queryByText(internal)).not.toBeInTheDocument();
    }
  });

  it("does not fall back to generic spell classification and labels rituals by family", () => {
    const unavailable = rimeSelected(5, [5], "8d4", 30, undefined, false, true);
    unavailable.metadata.level = 99;
    unavailable.metadata.traits = ["stale-query-trait"];
    const { rerender } = render(
      <RecordSurface onReference={vi.fn()} surface={unavailable} />,
    );

    expect(screen.queryByText("Level 99")).not.toBeInTheDocument();
    expect(screen.queryByText("Stale Query Trait")).not.toBeInTheDocument();
    expect(screen.getByText("8d4 Cold")).toBeInTheDocument();
    expect(screen.getByText("30-foot burst")).toBeInTheDocument();

    const baseUnsupported = rimeSelected(2, [], "2d4", 15, undefined, false, true);
    baseUnsupported.metadata.level = 99;
    baseUnsupported.metadata.traits = ["stale-query-trait"];
    if (baseUnsupported.presentation.presentation_type !== "spell") {
      throw new Error("spell fixture");
    }
    const baseDefinition = baseUnsupported.presentation.body.effective_form.result;
    if (
      baseDefinition.state !== "available" ||
      baseDefinition.definition.classification.state !== "unavailable"
    ) {
      throw new Error("base classification fixture");
    }
    baseDefinition.definition.classification.unavailable.source = "base";
    rerender(<RecordSurface onReference={vi.fn()} surface={baseUnsupported} />);
    expect(screen.queryByText("Level 99")).not.toBeInTheDocument();
    expect(screen.queryByText("Stale Query Trait")).not.toBeInTheDocument();
    expect(screen.getByText("2d4 Cold")).toBeInTheDocument();

    const ritual = planarRitual();
    ritual.profile = "search_compact";
    ritual.metadata.kind_label = "Spell";
    rerender(<RecordSurface onReference={vi.fn()} surface={ritual} />);
    expect(screen.getByText("Ritual")).toBeInTheDocument();
    expect(screen.queryByText(/^Spell$/)).not.toBeInTheDocument();
  });

  it("shows unavailable spell states once through common outer issues", () => {
    const whole = rimeSelected(1, [], "2d4", 15, {
      state: "unavailable",
      reason: { reason: "cast_rank_below_base", base_rank: 2, cast_rank: 1 },
    });
    if (whole.presentation.presentation_type !== "spell")
      throw new Error("spell fixture");
    whole.presentation.body.form_catalog_unavailable = "unsupported_overlay_root";
    whole.issues = [
      {
        code: "unavailable",
        placement: "forms",
        message: "Spell form catalog is unavailable.",
      },
      {
        code: "unavailable",
        placement: "forms",
        message: "Spell form is unavailable.",
      },
    ];
    const { rerender } = render(
      <RecordSurface
        onReference={vi.fn()}
        spellCatalog={whole}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 1 }}
        surface={whole}
      />,
    );
    expect(screen.getAllByText("Spell form catalog is unavailable.")).toHaveLength(1);
    expect(screen.getAllByText("Spell form is unavailable.")).toHaveLength(1);
    expect(screen.queryByText(/Cast Rank Below Base/)).not.toBeInTheDocument();
    expect(screen.queryByText(/Unsupported Overlay Root/)).not.toBeInTheDocument();

    const localized = rimeSelected(8, [5, 8], "14d4", 60, undefined, true);
    localized.issues = [
      {
        code: "unavailable",
        placement: "damage",
        message: "Damage is unavailable.",
      },
    ];
    rerender(
      <RecordSurface
        onReference={vi.fn()}
        onSpellFormSelection={vi.fn()}
        spellCatalog={localized}
        spellFormSelection={{ formId: "opaque:rime:base", castRank: 8 }}
        surface={localized}
      />,
    );
    expect(screen.getAllByText("Damage is unavailable.")).toHaveLength(1);
    expect(screen.queryByText(/Fixed Rank Key Mismatch/)).not.toBeInTheDocument();
    expect(screen.queryByText(/Fixed Heightening patch/)).not.toBeInTheDocument();
    expect(screen.getByLabelText("Spell range and targets")).toBeInTheDocument();
    expect(
      screen.queryByRole("combobox", { name: "Spell form" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/^Applied rank \d+$/)).not.toBeInTheDocument();

    const unknown = rimeSelected(5, [5], "8d4", 30);
    if (unknown.presentation.presentation_type !== "spell") {
      throw new Error("spell fixture");
    }
    unknown.presentation.body.effective_form.id = "opaque:unknown-form";
    unknown.issues = [
      {
        code: "unavailable",
        placement: "forms",
        message: "Spell form label is unavailable.",
      },
    ];
    rerender(
      <RecordSurface
        onReference={vi.fn()}
        onSpellFormSelection={vi.fn()}
        spellCatalog={unknown}
        spellFormSelection={{ formId: "opaque:unknown-form", castRank: 5 }}
        surface={unknown}
      />,
    );
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toBeVisible();
    expect(screen.queryByText(/^Applied rank \d+$/)).not.toBeInTheDocument();
    expect(screen.getAllByText("Spell form label is unavailable.")).toHaveLength(1);
    expect(screen.queryByText("opaque:unknown-form")).not.toBeInTheDocument();
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
        rank: known(5),
        changes: [
          { field: "casting" },
          { field: "targeting" },
          {
            field: "effect",
            operation: "merge",
            label: "Damage",
            value: {
              formula: known("4d6"),
              damage_type: known("fire"),
              category: missing,
              kinds: known(["damage"]),
              materials: known(["silver"]),
              apply_modifier: known(false),
            },
          },
        ],
      },
    ],
  });
  return spellSurface("Peaceful Rest", definition, [baseForm("opaque:peaceful:base")]);
}

function planarRitual(): RecordSurfaceView {
  const definition = baseDefinition();
  definition.damage = known([]);
  definition.duration = known({ value: known(""), sustained: known(false) });
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
  return spellSurface("Heal", plainSpellDefinition(), [
    overlayForm("opaque:heal:touch", "1 action — touch", "1d8 vitality", 0),
    overlayForm("opaque:heal:living", "Living creature", "1d8 vitality", 10),
    overlayForm("opaque:heal:undead", "Undead creature", "1d8 vitality", 20),
    overlayForm(
      "opaque:heal:emanation",
      "3 actions — 30-foot emanation",
      "1d8 vitality",
      30,
    ),
  ]);
}

function fireball(): RecordSurfaceView {
  const definition = plainSpellDefinition();
  definition.classification = known({
    rank: known(3),
    traits: known(["concentrate", "fire", "manipulate"]),
    traditions: known(["arcane", "primal"]),
  });
  definition.casting = known({
    time: known("2 actions"),
    cost: missing,
    requirements: missing,
    counteraction: known(false),
  });
  definition.targeting = known({
    target: missing,
    range: known({ authored_text: "500 feet" }),
    area: known({
      value: known(20),
      area_type: known("burst"),
      legacy_area_type: missing,
      details: missing,
    }),
  });
  definition.defense = known({
    passive: missing,
    save: known({ statistic: known("reflex"), basic: known(true) }),
  });
  definition.damage = known([
    {
      label: "Fire damage",
      formula: known("6d6"),
      damage_type: known("fire"),
      category: missing,
      kinds: known(["damage"]),
      materials: known([]),
      apply_modifier: known(false),
    },
  ]);
  definition.duration = missing;
  definition.heightening = known({
    kind: "interval",
    interval: known(1),
    area: missing,
    damage: known([{ label: "Fire damage", value: "2d6" }]),
  });
  return spellSurface("Fireball", definition, [
    baseFormAtRank("opaque:fireball:base", 3),
  ]);
}

function allRuleVariants(): RecordSurfaceView {
  const surface = qiBlast();
  if (surface.presentation.presentation_type !== "spell") {
    throw new Error("spell fixture");
  }
  const result = surface.presentation.body.effective_form.result;
  if (result.state !== "available") throw new Error("spell fixture");
  const rules: SpellRuleView[] = [
    {
      order: 0,
      rule: {
        kind: "damage_dice",
        value: {
          selector: known("damage-key-0"),
          predicate: known([{ kind: "term", value: "private:damage" }]),
          dice_number: known("1"),
          die_size: known("d6"),
          damage_type: known("electricity"),
          hide_if_disabled: known(true),
        },
      },
    },
    {
      order: 1,
      rule: {
        kind: "ephemeral_effect",
        value: {
          predicate: known([{ kind: "term", value: "private:ephemeral" }]),
          selectors: known(["strike-damage"]),
          uuid: known("Compendium.pf2e.spell-effects.Item.private-effect"),
        },
      },
    },
    {
      order: 2,
      rule: {
        kind: "damage_alteration",
        value: {
          mode: known("add"),
          predicate: known([{ kind: "term", value: "private:alteration" }]),
          property: known("damage-type"),
          selectors: known(["strike-damage"]),
          slug: known("private-alteration-slug"),
          value: known("electricity"),
        },
      },
    },
    {
      order: 3,
      rule: {
        kind: "roll_option",
        value: {
          domain: known("damage"),
          label: known("Energy type"),
          option: known("electricity"),
          placement: known("options"),
          predicate: known([{ kind: "term", value: "private:roll-option" }]),
          suboptions: known([
            { label: known("Electricity"), value: known("private-suboption") },
          ]),
          toggleable: known(true),
        },
      },
    },
    {
      order: 4,
      rule: {
        kind: "item_alteration",
        value: {
          item_id: known("private-item-id"),
          mode: known("override"),
          predicate: known([{ kind: "term", value: "private:item" }]),
          property: known("rarity"),
          value: known("rare"),
        },
      },
    },
  ];
  result.definition.rules = { state: "available", value: known(rules) };
  return surface;
}

function proneReference(): NonNullable<RecordSurfaceView["references"]> {
  return {
    outgoing: {
      state: "available",
      requested_limit: 8,
      records: [{ record_key: "conditions:prone", title: "Prone", kind: "condition" }],
      edges: [
        {
          from_record_key: "spells-srd:rime-slick",
          to_record_key: "conditions:prone",
          display_text: "Prone",
          reference_text: "Prone",
          source: {
            kind: "rich_content",
            visibility: "public",
            relation_kind: "references",
          },
        },
      ],
      total_records: 1,
      total_edges: 1,
      truncated: false,
    },
    backlinks: { state: "not_requested" },
  };
}

function rimeSelected(
  castRank: number,
  appliedRanks: number[],
  formula: string,
  area: number,
  resultOverride?: SpellFormResultView,
  localizedDamageUnavailable = false,
  localizedClassificationUnavailable = false,
): RecordSurfaceView {
  const result =
    resultOverride ??
    ({
      state: "available",
      definition: resolvedRimeDefinition(
        formula,
        area,
        appliedRanks,
        localizedDamageUnavailable,
        localizedClassificationUnavailable,
      ),
    } satisfies SpellFormResultView);
  const surface = spellSurface("Rime Slick", rimeDefinition(), [
    baseForm("opaque:rime:base"),
  ]);
  if (surface.presentation.presentation_type !== "spell")
    throw new Error("spell fixture");
  surface.presentation.body.effective_form = {
    id: "opaque:rime:base",
    cast_rank: castRank,
    result,
  };
  return surface;
}

function resolvedArea(surface: RecordSurfaceView): SpellAreaView {
  if (surface.presentation.presentation_type !== "spell") {
    throw new Error("spell fixture");
  }
  const result = surface.presentation.body.effective_form.result;
  if (
    result.state !== "available" ||
    result.definition.targeting.state !== "available" ||
    result.definition.targeting.value.state !== "known" ||
    result.definition.targeting.value.value.area.state !== "known"
  ) {
    throw new Error("spell area fixture");
  }
  return result.definition.targeting.value.value.area.value;
}

function spellSurface(
  title: string,
  definition: ReturnType<typeof baseDefinition>,
  forms: SpellFormView[],
): RecordSurfaceView {
  return {
    metadata: {
      record_key: `spells:${title.toLowerCase().replace(/ /g, "-")}`,
      title,
      kind: "spell",
      kind_label: "Spell",
      level: 99,
      traits: ["stale-query-trait"],
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
        family: definition.ritual.state === "known" ? "ritual" : "spell",
        forms,
        effective_form: {
          id: forms[0]?.id ?? "spell-form:unavailable",
          cast_rank: forms[0]?.minimum_cast_rank ?? 0,
          result: {
            state: "available",
            definition: resolvedFixtureDefinition(definition),
          },
        },
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

function baseDefinition(): SpellDefinitionFixture {
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
        label: "Persistent damage",
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
      damage: known([{ label: "Persistent damage", value: "1d6" }]),
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

function plainSpellDefinition(): SpellDefinitionFixture {
  const definition = baseDefinition();
  definition.ritual = missing;
  definition.rules = missing;
  return definition;
}

function rimeDefinition(): SpellDefinitionFixture {
  const definition = plainSpellDefinition();
  definition.classification = known({
    rank: known(2),
    traits: known(["cold", "concentrate", "manipulate"]),
    traditions: known(["arcane", "primal"]),
  });
  definition.casting = known({
    time: known("2"),
    cost: known(""),
    requirements: known(""),
    counteraction: known(false),
  });
  definition.targeting = known({
    target: known(""),
    range: known({ authored_text: "60 feet" }),
    area: known({
      value: known(15),
      area_type: known("burst"),
      legacy_area_type: missing,
      details: missing,
    }),
  });
  definition.defense = known({
    passive: missing,
    save: known({ statistic: known("reflex"), basic: known(true) }),
  });
  definition.damage = known([
    {
      label: "Damage",
      formula: known("2d4"),
      damage_type: known("cold"),
      category: missing,
      kinds: known(["damage"]),
      materials: known([]),
      apply_modifier: known(false),
    },
  ]);
  definition.duration = known({ value: known("1 minute"), sustained: known(false) });
  definition.heightening = known({
    kind: "fixed",
    layers: [
      {
        rank: known(5),
        changes: [
          { field: "targeting" },
          {
            field: "effect",
            operation: "merge",
            label: "Damage",
            value: {
              formula: known("8d4"),
              damage_type: known("cold"),
              category: missing,
              kinds: missing,
              materials: known([]),
              apply_modifier: known(false),
            },
          },
        ],
      },
      {
        rank: known(8),
        changes: [
          { field: "targeting" },
          {
            field: "effect",
            operation: "merge",
            label: "Damage",
            value: {
              formula: known("14d4"),
              damage_type: known("cold"),
              category: missing,
              kinds: missing,
              materials: known([]),
              apply_modifier: known(false),
            },
          },
        ],
      },
    ],
  });
  return definition;
}

function baseForm(id: string): SpellFormView {
  return baseFormAtRank(id, 2);
}

function baseFormAtRank(id: string, minimumCastRank: number): SpellFormView {
  return {
    id,
    label: "Base spell",
    order: 0,
    minimum_cast_rank: minimumCastRank,
    kind: "base" as const,
  };
}

function overlayForm(
  id: string,
  label: string,
  formula: string,
  order: number,
): SpellFormView {
  void formula;
  return {
    id,
    label,
    order,
    minimum_cast_rank: 2,
    kind: "overlay" as const,
  };
}

function resolvedRimeDefinition(
  formula: string,
  area: number,
  appliedFixedRanks: number[],
  damageUnavailable = false,
  classificationUnavailable = false,
): SpellResolvedDefinitionView {
  const definition = rimeDefinition();
  if (
    definition.targeting.state === "known" &&
    definition.targeting.value.area.state === "known"
  ) {
    const areaValue = definition.targeting.value.area.value.value;
    if (areaValue.state === "known") areaValue.value = area;
  }
  return {
    applied_fixed_ranks: appliedFixedRanks,
    classification: classificationUnavailable
      ? {
          state: "unavailable",
          unavailable: {
            field: "classification",
            source: "overlay",
            reason: "unsupported_patch",
          },
        }
      : { state: "available", value: definition.classification },
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
              label: "Damage",
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
    ritual: definition.ritual,
    rules: { state: "available", value: definition.rules },
  };
}

function resolvedFixtureDefinition(
  definition: ReturnType<typeof baseDefinition>,
): SpellResolvedDefinitionView {
  return {
    applied_fixed_ranks: [],
    classification: { state: "available", value: definition.classification },
    casting: { state: "available", value: definition.casting },
    targeting: { state: "available", value: definition.targeting },
    defense: { state: "available", value: definition.defense },
    damage: { state: "available", value: definition.damage },
    duration: { state: "available", value: definition.duration },
    heightening: { state: "available", value: definition.heightening },
    ritual: definition.ritual,
    rules: { state: "available", value: definition.rules },
  };
}
