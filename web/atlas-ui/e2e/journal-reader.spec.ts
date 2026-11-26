import { expect, test } from "@playwright/test";
import { readUrl } from "./readingUrl";

test("card 18 reads inline and its references and copied section still reach their own destinations", async ({
  page,
}) => {
  const path = "/records/criticaldeck:Toz2ZIpy7N4pDmyi";
  await page.goto(path);
  await expect(
    page.getByRole("heading", { name: "Critical Fumble Deck #18", exact: true }),
  ).toHaveCount(1);
  for (const name of [
    "Overextended",
    "Aching Back",
    "Upset Stomach",
    "Critical Backlash",
  ]) {
    await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
  }
  const outcome = page
    .getByRole("navigation", { name: "On this page" })
    .getByRole("link", { name: "Aching Back" });
  const section = new URL((await outcome.getAttribute("href"))!, page.url());
  expect(readUrl(section).embedded_path).toEqual([
    { collection: "/pages", identity: { Stable: "rZHd27zct5D3dN6w" } },
  ]);
  await outcome.focus();
  await outcome.press("Enter");
  await expect(page.getByRole("heading", { name: "Aching Back" })).toBeFocused();
  expect(new URL(page.url()).pathname).toBe(path);
  expect(readUrl(new URL(page.url())).embedded_path).toHaveLength(0);
  const condition = page.getByRole("link", { name: "Fatigued", exact: true });
  await condition.click();
  await expect(
    page.getByRole("dialog", { name: "Fatigued", exact: true }),
  ).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(condition).toBeFocused();
  await page.goto(section.href);
  await expect(page.getByRole("heading", { name: "Aching Back" })).toBeFocused();
});

test("the journal contents keeps ancestry dividers and GM Screen section pages in authored order", async ({
  page,
}) => {
  await page.goto("/records/journals:45SK8rdbbxvEHfMn");
  const contents = page.getByRole("region", { name: "Journal contents" });
  await expect(contents.getByRole("heading")).toHaveText([
    "Contents",
    "Common",
    "Uncommon",
    "Rare",
  ]);
  await expect(contents.getByRole("link", { name: "Common", exact: true })).toHaveCount(
    0,
  );
  await contents.getByRole("textbox", { name: "Find a page" }).fill("Dwarf");
  await expect(contents.getByRole("heading")).toHaveText(["Contents", "Common"]);
  const dwarf = contents.getByRole("link", { name: "Dwarf", exact: true });
  await dwarf.click();
  await expect(page.getByRole("dialog", { name: "Dwarf", exact: true })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dwarf).toBeFocused();
  await expect(contents.getByRole("textbox", { name: "Find a page" })).toHaveValue(
    "Dwarf",
  );

  await page.goto("/records/journals:S55aqwWIzpQRFhcq");
  await expect(contents.getByRole("heading")).toHaveText([
    "Contents",
    "Playing the Game",
    "Running the Game",
    "Subsystems and Variant Rules",
  ]);
  await expect(contents.getByRole("link").nth(0)).toHaveText("GM Screen");
  await expect(contents.getByRole("link").nth(1)).toHaveText("Player Screen");
  await contents.getByRole("textbox", { name: "Find a page" }).fill("Simple DCs");
  await expect(contents.getByRole("link")).toHaveText([
    "Running the Game",
    "Simple DCs",
  ]);
  await contents
    .getByRole("heading", { name: "Running the Game" })
    .getByRole("link")
    .click();
  await expect(
    page.getByRole("dialog", { name: "Running the Game", exact: true }),
  ).toContainText("Pathfinder GM Core");
});
