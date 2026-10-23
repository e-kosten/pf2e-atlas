import { render, screen } from "@testing-library/react";
import { numberFact, spellFixture } from "../../test/fixtures";
import { SpellReference } from "./SpellReference";

it("renders the supplied runtime spell attack and DC without replacing authored facts", () => {
  const spell = {
    ...spellFixture(),
    runtime_attack: numberFact(9),
    runtime_difficulty_class: numberFact(21),
  };
  const { rerender } = render(<SpellReference spell={spell} />);
  expect(screen.getByText("Spell attack").parentElement).toHaveTextContent("+9");
  expect(screen.getByText("Spell DC").parentElement).toHaveTextContent("21");
  rerender(
    <SpellReference
      spell={{ ...spell, runtime_attack: null, runtime_difficulty_class: null }}
    />,
  );
  expect(screen.queryByText("Spell attack")).toBeNull();
  expect(screen.queryByText("Spell DC")).toBeNull();
});
