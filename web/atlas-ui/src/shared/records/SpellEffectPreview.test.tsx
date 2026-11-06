import {
  fireEvent,
  render as renderWithContext,
  screen,
  waitFor,
} from "@testing-library/react";
import { RouterTestContext } from "../../test/router";
import { ConfigProvider } from "antd";
import { ReferenceHelpProvider } from "./ReferenceHelp";
import { SpellEffectPreview } from "./SpellEffectPreview";
import type { SpellEffectPreviewView } from "../../generated/atlas";
const render = (ui: React.ReactNode) =>
  renderWithContext(ui, { wrapper: RouterTestContext });

const preview: SpellEffectPreviewView = {
  rank: 4,
  base_rank: 2,
  choices: [],
  strikes: [],
  unavailable: null,
  facts: [
    {
      label: "Temporary HP",
      value: { state: "value", value: "15" },
      base_value: "5",
      qualifier: null,
    },
    {
      label: "Athletics",
      value: { state: "value", value: "+16" },
      base_value: "+9",
      qualifier: "Use your own modifier if higher.",
    },
    {
      label: "Speed",
      value: { state: "value", value: "30 feet" },
      base_value: null,
      qualifier: null,
    },
  ],
};
it("explains changed source statistics while keeping unchanged facts and qualifiers inline", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <SpellEffectPreview view={preview} />
    </ConfigProvider>,
  );
  expect(screen.getByText("Use your own modifier if higher.")).toBeVisible();
  expect(screen.queryByRole("button", { name: /Speed/ })).toBeNull();
  fireEvent.click(
    screen.getByRole("button", {
      name: "Heightening details for Temporary HP: 15",
    }),
  );
  expect(await screen.findByRole("dialog")).toHaveTextContent("Base rank: 5");
});
it("does not substitute zero or apply an explanation to an unavailable fact", () => {
  render(
    <SpellEffectPreview
      view={{
        ...preview,
        facts: [
          {
            label: "Temporary HP",
            value: { state: "invalid", value: null },
            base_value: "5",
            qualifier: null,
          },
        ],
      }}
    />,
  );
  expect(screen.getByText("Unavailable")).toBeVisible();
  expect(screen.queryByRole("button")).toBeNull();
  expect(screen.queryByText("0")).toBeNull();
});
it("keeps an unavailable rank preview distinct from an empty collection", () => {
  render(
    <SpellEffectPreview
      view={{
        ...preview,
        facts: [],
        unavailable: "The choice depends on the caster's conditions.",
      }}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent(
    "Rank 4 statistics unavailable.",
  );
});

it("links the complete supplied strike damage without parsing it or nesting heightening controls", async () => {
  const help = {
    definition_key: "damage:fire",
    availability: "available" as const,
    destination: {
      kind: "record" as const,
      navigation: {
        record_key: "actions:fire",
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
    },
    related_filters: [],
  };
  const onHelp = vi.fn();
  const { container } = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <ReferenceHelpProvider onHelp={onHelp}>
        <SpellEffectPreview
          view={{
            ...preview,
            facts: [],
            strikes: [
              {
                id: "jaws",
                title: "Jaws",
                traits: [],
                attack: {
                  label: "Attack",
                  value: { state: "value", value: "+16" },
                  base_value: null,
                  qualifier: null,
                },
                damage: {
                  label: "Damage",
                  value: { state: "value", value: "2d8 fire plus custom effect" },
                  base_value: "1d8 fire plus custom effect",
                  qualifier: "Only while transformed.",
                },
                damage_type: {
                  state: "value",
                  value: { value: "fire", label: "Fire", help },
                },
              },
            ],
          }}
        />
      </ReferenceHelpProvider>
    </ConfigProvider>,
  );
  expect(container.querySelector("button a, a button")).toBeNull();
  expect(screen.queryByText("Damage type")).toBeNull();
  expect(screen.getByText("Only while transformed.")).toBeVisible();
  const damage = screen.getByRole("link", { name: "2d8 fire plus custom effect" });
  fireEvent.click(damage);
  expect(onHelp).toHaveBeenCalledWith(help, damage, undefined);
  const explanation = screen.getByRole("button", {
    name: "Heightening details for Damage",
  });
  fireEvent.click(explanation);
  expect(await screen.findByRole("dialog")).toHaveTextContent(
    "Base rank: 1d8 fire plus custom effect",
  );
  fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  await waitFor(() => expect(explanation).toHaveFocus());
});
