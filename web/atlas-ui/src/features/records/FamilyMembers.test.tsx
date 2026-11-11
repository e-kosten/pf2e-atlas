import { fireEvent, render, screen } from "@testing-library/react";
import { ConfigProvider } from "antd";
import { detailFixture } from "../../test/fixtures";
import { FamilyMembers } from "./FamilyMembers";

it("shows each concrete member once in family order in All variants", async () => {
  const older = { ...detailFixture("actors:older").record, title: "Older form" };
  const younger = { ...detailFixture("actors:younger").record, title: "Younger form" };
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <FamilyMembers
        choices={[{ record: older }, { record: younger }]}
        groupings={[
          {
            id: "scenario:party-levels",
            label: "Party levels",
            groups: [
              {
                heading: "Party levels 3–4",
                members: [younger.record_key, older.record_key],
                qualifiers: { [older.record_key]: ["28+ Challenge Points"] },
              },
              {
                heading: "Party levels 5–6",
                members: [older.record_key],
              },
            ],
          },
        ]}
      />
    </ConfigProvider>,
  );
  expect(screen.getByRole("heading", { name: "Party levels 3–4" })).toBeVisible();
  expect(screen.getAllByRole("link", { name: /Older form/ })).toHaveLength(2);
  expect(screen.getByText("28+ Challenge Points")).toBeVisible();

  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Group by" }));
  fireEvent.click(await screen.findByText("All variants"));
  expect(screen.queryByRole("heading", { name: "Party levels 3–4" })).toBeNull();
  expect(screen.queryByText("28+ Challenge Points")).toBeNull();
  expect(screen.getAllByRole("link").map((link) => link.textContent)).toEqual([
    expect.stringContaining("Older form"),
    expect.stringContaining("Younger form"),
  ]);
});

it("uses an authored flat default and can switch to an axis", async () => {
  const record = { ...detailFixture("actors:adult").record, title: "Adult dragon" };
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <FamilyMembers
        choices={[{ record }]}
        groupings={[
          {
            id: "age",
            label: "Age",
            groups: [{ heading: "Adult", members: [record.record_key] }],
          },
        ]}
        allVariantsByDefault
      />
    </ConfigProvider>,
  );

  expect(screen.getByText("All variants")).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Adult" })).toBeNull();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Group by" }));
  fireEvent.click(await screen.findByText("Age"));
  expect(screen.getByRole("heading", { name: "Adult" })).toBeVisible();
});
