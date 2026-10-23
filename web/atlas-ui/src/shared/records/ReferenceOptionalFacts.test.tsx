import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ContentFields } from "./RecordPresentation";
import { SpellChanges, SpellReference } from "./SpellReference";
import type {
  PreparedFieldBodyView,
  SpellChangeView,
  SpellPresentationView,
} from "../../generated/atlas";
import {
  knownFact,
  numberFact,
  spellFixture,
  unavailableFact,
} from "../../test/fixtures";

describe("optional reference content", () => {
  it("keeps supplied damage before prose and omits a zero area increase without losing the scaling rule", () => {
    const spell = spellFixture();
    spell.damage = knownFact([
      {
        id: "0",
        formula: knownFact("10d6"),
        damage_type: knownFact("fire"),
        kinds: knownFact(["damage"]),
        category: unavailableFact("null"),
        materials: knownFact([]),
        apply_modifier: knownFact(false),
      },
    ]);
    spell.heightening = knownFact({
      kind: "interval",
      interval: numberFact(1),
      area: numberFact(0),
      damage: knownFact([{ id: "0", formula: knownFact("2d6") }]),
    });
    render(
      <SpellReference spell={spell}>
        <p>Authored fireball description.</p>
      </SpellReference>,
    );
    const damage = screen.getByText("10d6");
    expect(
      damage.compareDocumentPosition(
        screen.getByText("Authored fireball description."),
      ) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(screen.queryByText("Area increase")).toBeNull();
    expect(screen.queryByText("Damage increase")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Heightening details" }));
    expect(screen.getByText("Damage increase").parentElement).toHaveTextContent("2d6");
    expect(screen.queryByText("Area increase")).toBeNull();
    expect(screen.queryByText(/Some values may remain/)).toBeNull();
    expect(screen.queryByText(/Read with the spell text/)).toBeNull();
  });

  it("retains an explicit unavailable rank preview note", () => {
    render(
      <SpellReference
        spell={{
          ...spellFixture(),
          rank_note: "Showing original mechanics; rank 5 preview is unavailable.",
        }}
      />,
    );
    expect(
      screen.getByText("Showing original mechanics; rank 5 preview is unavailable."),
    ).toBeVisible();
  });
  it("omits empty rank patches while keeping explicit clears, unavailable changes and authored rules", async () => {
    const missing = <T,>() => unavailableFact<T>("missing");
    const changes: SpellChangeView = {
      cast: missing(),
      range: missing(),
      target: missing(),
      area: missing(),
      defense: missing(),
      duration: missing(),
      sustained: missing(),
      traits: missing(),
      traditions: missing(),
      damage: missing(),
      heightening: missing(),
    };
    const spell = spellFixture();
    spell.rank = numberFact(5);
    spell.heightening = knownFact({
      kind: "fixed",
      levels: knownFact([
        { rank: 4, changes: knownFact(changes) },
        { rank: 5, changes: knownFact({ ...changes, target: knownFact("") }) },
        { rank: 6, changes: unavailableFact("null") },
      ]),
    });
    render(
      <SpellReference spell={spell}>
        <p>As 4th rank, plus an additional benefit.</p>
      </SpellReference>,
    );
    expect(screen.getByText("As 4th rank, plus an additional benefit.")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Heightening details" }));
    expect(screen.queryByRole("heading", { name: "Rank 4" })).toBeNull();
    await waitFor(() =>
      expect(screen.getByRole("heading", { name: "Rank 5" })).toBeVisible(),
    );
    expect(
      screen.getByRole("heading", { name: "Rank 5" }).parentElement,
    ).toHaveAttribute("aria-current", "true");
    expect(screen.getByText("Empty")).toBeVisible();
    expect(screen.getByRole("heading", { name: "Rank 6" })).toBeVisible();
    expect(screen.getByText("Cleared")).toBeVisible();
    expect(screen.getByText("As 4th rank, plus an additional benefit.")).toBeVisible();
  });
  it.each(["missing", "null", "invalid"] as const)(
    "retains unavailable heightening/forms parents (%s) while omitting source-embedded inapplicable structures",
    (state) => {
      const spell: SpellPresentationView = {
        ...spellFixture(),
        heightening: unavailableFact(state),
        forms: unavailableFact<never[]>(state),
      };
      const { rerender } = render(<SpellReference spell={spell} />);
      expect(screen.getByText("Heightening").parentElement).toHaveTextContent(
        state === "missing" ? "—" : `[${state}]`,
      );
      expect(screen.getByText("Spell forms").parentElement).toHaveTextContent(
        state === "missing" ? "—" : `[${state}]`,
      );
      rerender(
        <SpellReference
          spell={{
            ...spell,
            heightening: unavailableFact("not_applicable"),
            forms: unavailableFact("not_applicable"),
          }}
        />,
      );
      expect(screen.queryByText("Heightening")).toBeNull();
      expect(screen.queryByText("Spell forms")).toBeNull();
    },
  );
  it("retains a spell-form defense clear independently of the baseline", () => {
    const absent = <T,>() => unavailableFact<T>("missing");
    render(
      <SpellChanges
        changes={{
          cast: absent(),
          range: absent(),
          target: absent(),
          area: absent(),
          defense: unavailableFact("null"),
          duration: absent(),
          sustained: absent(),
          traits: absent(),
          traditions: absent(),
          damage: absent(),
          heightening: absent(),
        }}
      />,
    );
    expect(screen.getByText("Defense").parentElement).toHaveTextContent("Cleared");
    expect(screen.queryByText("Save statistic")).toBeNull();
  });
  it("omits explicit no-defense and raw casting-entry IDs from baseline web reading", () => {
    const spell = {
      ...spellFixture(),
      defense: null,
      casting_entry: knownFact("local-foundry-entry"),
    };
    const { container } = render(<SpellReference spell={spell} />);
    expect(screen.queryByText("Defense")).toBeNull();
    expect(screen.queryByText("Casting entry")).toBeNull();
    expect(container).not.toHaveTextContent("local-foundry-entry");
    expect(spell.casting_entry.value).toBe("local-foundry-entry");
  });

  it.each(["missing", "null", "invalid"] as const)(
    "retains supplied unavailable baseline defense (%s)",
    (state) => {
      render(
        <SpellReference
          spell={{ ...spellFixture(), defense: unavailableFact(state) }}
        />,
      );
      expect(screen.getByText("Defense").parentElement).toHaveTextContent(
        state === "missing" ? "—" : `[${state}]`,
      );
    },
  );
  it.each<PreparedFieldBodyView>([
    { kind: "plain", text: " \n" },
    { kind: "html", html: "", controls: [] },
  ])("omits only known-empty Notes content ($kind)", (body) => {
    render(
      <ContentFields
        fields={[
          {
            locator: { record: "actors:ghoul", embedded_path: [], field: "notes" },
            role: "notes",
            source_fingerprint: null,
            body,
          },
        ]}
        onReference={vi.fn()}
      />,
    );
    expect(screen.queryByRole("heading", { name: "Notes" })).toBeNull();
  });

  it.each(["missing", "null", "invalid"] as const)(
    "preserves unavailable Notes (%s)",
    (state) => {
      render(
        <ContentFields
          fields={[
            {
              locator: { record: "actors:ghoul", embedded_path: [], field: "notes" },
              role: "notes",
              source_fingerprint: null,
              body: { kind: "unavailable", state },
            },
          ]}
          onReference={vi.fn()}
        />,
      );
      expect(screen.getByRole("heading", { name: "Notes" })).toBeVisible();
    },
  );

  it("preserves nonempty Notes and uses authored or ordered spell-form captions", () => {
    render(
      <ContentFields
        fields={[
          {
            locator: { record: "actors:ghoul", embedded_path: [], field: "notes" },
            role: "notes",
            source_fingerprint: null,
            body: { kind: "plain", text: "void healing" },
          },
        ]}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByText("void healing")).toBeVisible();
    const spell = spellFixture();
    spell.forms = knownFact([
      {
        id: "37gy7l19tik74o4s",
        name: knownFact("Heal (vs. Living)"),
        sort: numberFact(1),
        changes: unavailableFact("invalid"),
      },
      {
        id: "raw-generated-id",
        name: unavailableFact("missing"),
        sort: numberFact(2),
        changes: unavailableFact("invalid"),
      },
      {
        id: "invalid-name-id",
        name: unavailableFact("invalid"),
        sort: numberFact(3),
        changes: unavailableFact("invalid"),
      },
    ]);
    render(<SpellReference spell={spell} />);
    expect(screen.getByRole("heading", { name: "Heal (vs. Living)" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "Spell form 2" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "Spell form 3" })).toBeVisible();
    expect(screen.getByText("Form name").parentElement).toHaveTextContent("[invalid]");
    expect(screen.queryByText("raw-generated-id")).toBeNull();
  });

  it("omits only known blank damage increments, retaining zero and unavailable formulas", () => {
    const spell = spellFixture();
    spell.heightening = knownFact({
      kind: "interval",
      interval: numberFact(1),
      area: { state: "not_applicable", value: null, adjustment: null },
      damage: knownFact([
        { id: "empty", formula: knownFact("") },
        { id: "whitespace", formula: knownFact(" \t") },
        { id: "zero", formula: knownFact("0") },
        { id: "unknown", formula: unavailableFact("missing") },
        { id: "invalid", formula: unavailableFact("invalid") },
      ]),
    });
    render(<SpellReference spell={spell} />);
    fireEvent.click(screen.getByRole("button", { name: "Heightening details" }));
    expect(screen.getByText("Damage increase 1").parentElement).toHaveTextContent("0");
    expect(screen.getByText("Damage increase 2").parentElement).toHaveTextContent("—");
    expect(screen.getByText("Damage increase 3").parentElement).toHaveTextContent(
      "[invalid]",
    );
    expect(screen.queryByText("Damage increase 4")).toBeNull();
  });
});
