import { render, screen } from "@testing-library/react";
import { actorFixture, knownFact, unavailableFact } from "../../test/fixtures";
import { ActorLanguages } from "./ActorReference";
import { FactLine } from "./presentationFacts";
import { RecordRarity } from "./RecordRarity";
import { SpellChanges } from "./SpellReference";
import type { SpellChangeView } from "../../generated/atlas";

it("renders supplied language labels with communication notes while preserving raw tokens", () => {
  const actor = actorFixture();
  actor.languages = knownFact([
    { value: "common", label: "Common" },
    { value: "chthonian", label: "Chthonian" },
  ]);
  actor.language_details = knownFact("Communicates by signing.");
  render(<ActorLanguages actor={actor} />);
  expect(screen.getByText("Common, Chthonian")).toBeVisible();
  expect(screen.getByText("Communicates by signing.")).toBeVisible();
  expect(actor.languages.value?.map((value) => value.value)).toEqual([
    "common",
    "chthonian",
  ]);
});
it("uses backend labels for scalar facts and rarity without deriving them from tokens", () => {
  render(
    <>
      <FactLine
        label="Sense"
        fact={knownFact({ value: "darkvision", label: "Supplied vision label" })}
      />
      <RecordRarity rarity={{ value: "rare", label: "Supplied rarity label" }} />
    </>,
  );
  expect(screen.getByText("Supplied vision label")).toBeVisible();
  expect(screen.getByLabelText("Rarity: Supplied rarity label")).toBeVisible();
});

it("uses supplied spell change trait labels in authored and effective reading", () => {
  const changes: SpellChangeView = {
    cast: unavailableFact("missing"),
    range: unavailableFact("missing"),
    target: unavailableFact("missing"),
    area: unavailableFact("missing"),
    defense: unavailableFact("missing"),
    duration: unavailableFact("missing"),
    sustained: unavailableFact("missing"),
    traits: knownFact([{ value: "mental-token", label: "Supplied spell trait" }]),
    traditions: unavailableFact("missing"),
    damage: unavailableFact("missing"),
    heightening: unavailableFact("missing"),
  };
  const { rerender } = render(<SpellChanges changes={changes} />);
  expect(screen.getByText("Supplied spell trait")).toBeVisible();
  expect(screen.queryByText("mental-token")).toBeNull();
  rerender(<SpellChanges changes={changes} effective />);
  expect(screen.getByText("Supplied spell trait")).toBeVisible();
  expect(screen.queryByText("mental-token")).toBeNull();
});
