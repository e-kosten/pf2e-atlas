import { ConfigProvider } from "antd";
import { fireEvent, render, screen, within, waitFor } from "@testing-library/react";
import { numberFact } from "../../test/fixtures";
import { NumberFact } from "./presentationFacts";

it("shows applied and overridden modifiers in the same cause/value calculation", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <NumberFact
        label="Attack"
        signed
        fact={{
          ...numberFact(3),
          adjustment: {
            authored: 6,
            applied: [
              {
                source: "enfeebled",
                label: "Enfeebled",
                modifier_type: "status",
                value: -3n,
              },
            ],
            suppressed: [
              {
                source: "frightened",
                label: "Frightened",
                modifier_type: "status",
                value: -1n,
              },
            ],
            notes: [],
          },
        }}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Show explanation for Attack" }));
  const calculation = within(
    await screen.findByRole("region", { name: "Attack calculation" }),
  );
  await waitFor(() => expect(calculation.getByRole("table")).toBeVisible());
  expect(calculation.getByRole("columnheader", { name: "Cause" })).toBeVisible();
  expect(calculation.getByRole("row", { name: "Original +6" })).toBeVisible();
  const applied = within(calculation.getByRole("row", { name: /Enfeebled/ }));
  expect(applied.getByRole("cell", { name: "-3" })).toBeVisible();
  expect(applied.getByText("status", { exact: true })).toBeVisible();
  const overridden = within(calculation.getByRole("row", { name: /Frightened/ }));
  expect(overridden.getByText("-1 status · Overridden")).toBeVisible();
  expect(overridden.getByRole("cell", { name: "—" })).toBeVisible();
  expect(overridden.getByText("Frightened")).toHaveClass("calculation-table__muted");
  expect(calculation.getByRole("row", { name: "Effective +3" })).toBeVisible();
  expect(calculation.queryByText(/Original:|Effective:|suppressed/)).toBeNull();
});

it("preserves units, positive adjustments, and contextual notes", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <NumberFact
        label="Speed"
        unit=" feet"
        fact={{
          ...numberFact(30),
          adjustment: {
            authored: 25,
            applied: [
              {
                source: "speed",
                label: "Speed bonus",
                modifier_type: "circumstance",
                value: 5n,
              },
            ],
            suppressed: [],
            notes: [
              {
                source: "terrain",
                label: "Terrain",
                reason: "Check difficult terrain.",
              },
            ],
          },
        }}
      />
    </ConfigProvider>,
  );
  expect(screen.getByText("30 feet")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Show explanation for Speed" }));
  const calculation = within(
    await screen.findByRole("region", { name: "Speed calculation" }),
  );
  await waitFor(() => expect(calculation.getByRole("table")).toBeVisible());
  expect(calculation.getByRole("row", { name: "Original 25 feet" })).toBeVisible();
  expect(calculation.getByRole("cell", { name: "+5 feet" })).toBeVisible();
  expect(calculation.getByText("Check difficult terrain.")).toBeVisible();
  expect(calculation.getByRole("row", { name: "Effective 30 feet" })).toBeVisible();
});
