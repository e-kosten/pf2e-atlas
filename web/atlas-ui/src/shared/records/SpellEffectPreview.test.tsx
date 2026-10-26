import { fireEvent, render, screen } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { SpellEffectPreview } from "./SpellEffectPreview";
import type { SpellEffectPreviewView } from "../../generated/atlas";
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
