import { ConfigProvider } from "antd";
import {
  fireEvent,
  render as renderWithContext,
  screen,
  within,
  waitFor,
} from "@testing-library/react";
import { RouterTestContext } from "../../test/router";
import { knownFact, unavailableFact, numberFact } from "../../test/fixtures";
import { ReferenceHelpProvider } from "./ReferenceHelp";
import { ReferenceProfiles } from "./ReferenceProfiles";
import { FactLine } from "./presentationFacts";
import { RecordRarity } from "./RecordRarity";
import { SpellChanges } from "./SpellReference";
import type { SpellChangeView, ReferenceHelpView } from "../../generated/atlas";

const render = (ui: React.ReactNode) =>
  renderWithContext(ui, { wrapper: RouterTestContext });

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

const help: ReferenceHelpView = {
  definition_key: "sense:darkvision",

  destination: {
    kind: "record",
    navigation: {
      record_key: "actions:darkvision",
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    },
  },
};

it("preserves profile choices, speaker notes and exact supplied help without inventing a label-only destination", () => {
  const onHelp = vi.fn();
  const value = (token: string, label: string) => ({
    value: token,
    label,
    help: { ...help, definition_key: token },
  });
  render(
    <ReferenceHelpProvider onHelp={onHelp}>
      <ReferenceProfiles
        body={{
          kind: "ancestry",
          value: {
            hit_points: numberFact(8),
            speed: numberFact(25),
            size: knownFact(value("size:med", "Medium")),
            vision: knownFact(value("sense:darkvision", "Darkvision")),
            hands: numberFact(2),
            reach: numberFact(5),
            boosts: knownFact([
              {
                free_choice: false,
                choices: knownFact([
                  value("ability:str", "Strength"),
                  value("ability:dex", "Dexterity"),
                ]),
              },
              { free_choice: true, choices: knownFact([]) },
            ]),
            flaws: knownFact([]),
            languages: knownFact([
              value("language:common", "Common"),
              { value: "language:custom", label: "Custom speech" },
            ]),
            language_details: knownFact("Uses hand signals underwater."),
            additional_languages: knownFact([value("language:elven", "Elven")]),
            additional_language_count: numberFact(1),
            additional_language_details: knownFact("Chosen at creation."),
            grants: knownFact([]),
          },
        }}
        onReference={vi.fn()}
      >
        <p>Complete ancestry description.</p>
      </ReferenceProfiles>
    </ReferenceHelpProvider>,
  );
  expect(screen.getByText("Attribute boosts").parentElement).toHaveTextContent(
    "Strength or Dexterity, Free",
  );
  expect(screen.getByText("Uses hand signals underwater.")).toBeVisible();
  expect(screen.getByText("Chosen at creation.")).toBeVisible();
  expect(screen.getByText("Complete ancestry description.")).toBeVisible();
  expect(screen.queryByRole("link", { name: "Custom speech" })).toBeNull();
  for (const name of [
    "Medium",
    "Darkvision",
    "Strength",
    "Dexterity",
    "Common",
    "Elven",
  ]) {
    const link = screen.getByRole("link", { name });
    fireEvent.click(link);
    expect(onHelp).toHaveBeenLastCalledWith(
      expect.objectContaining({ definition_key: expect.any(String) }),
      link,
      undefined,
    );
  }
});

it("offers spell categorical help and heightening separately while retaining explicit clears", async () => {
  const changes: SpellChangeView = {
    cast: unavailableFact("missing"),
    range: unavailableFact("missing"),
    target: unavailableFact("missing"),
    area: unavailableFact("missing"),
    defense: unavailableFact("missing"),
    duration: unavailableFact("missing"),
    sustained: unavailableFact("missing"),
    traits: knownFact([{ value: "mental", label: "Mental", help }]),
    traditions: unavailableFact("null"),
    damage: unavailableFact("missing"),
    heightening: unavailableFact("missing"),
  };
  const onHelp = vi.fn();
  const { container } = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <ReferenceHelpProvider onHelp={onHelp}>
        <SpellChanges
          changes={changes}
          rankChanges={[
            {
              fact: { kind: "traits" },
              base_rank: 1,
              base_value: "none",
              explanation: "Supplied change",
            },
          ]}
        />
      </ReferenceHelpProvider>
    </ConfigProvider>,
  );
  expect(container.querySelector("button a, a button")).toBeNull();
  expect(screen.getByText("Traditions").parentElement).toHaveTextContent("Cleared");
  fireEvent.click(screen.getByRole("link", { name: "Mental" }));
  expect(onHelp).toHaveBeenCalledWith(help, expect.any(HTMLElement), undefined);
  fireEvent.click(
    screen.getByRole("button", { name: "Heightening details for Traits" }),
  );
  expect(await screen.findByRole("dialog")).toHaveTextContent("Base rank: none");
  fireEvent.click(
    within(screen.getByRole("dialog")).getByRole("button", {
      name: "Close explanation",
    }),
  );
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});
