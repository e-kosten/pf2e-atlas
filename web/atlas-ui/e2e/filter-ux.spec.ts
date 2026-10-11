import { expect, test } from "@playwright/test";

test("common filters retain valid results through drafts and exact URL reload", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/search");
  const kindPicker = page.getByRole("button", {
    name: "Edit Kind filter",
    exact: true,
  });
  await kindPicker.click();
  const creature = page.getByRole("button", {
    name: "Kind: Creature, neutral; activate to include",
    exact: true,
  });
  await expect(creature).toBeVisible();
  await expect(page.getByText("Retrieval", { exact: true })).toHaveCount(0);
  await creature.click();
  const spell = page.getByRole("button", { name: /^Kind: Spell,/ });
  await spell.click();
  await spell.click();
  await expect
    .poll(() => new URL(page.url()).searchParams.get("filter"))
    .toContain("creature");
  const mixed = JSON.parse(new URL(page.url()).searchParams.get("filter")!);
  expect(mixed.children).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ kind: "in", values: ["creature"] }),
      expect.objectContaining({
        kind: "not",
        predicate: expect.objectContaining({ values: ["spell"] }),
      }),
    ]),
  );
  await page.keyboard.press("Escape");
  await expect(kindPicker).toBeFocused();
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await expect(
    page.getByRole("combobox", { name: "Filter field", exact: true }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await page.getByRole("textbox", { name: "Level / rank minimum" }).fill("1");
  await expect
    .poll(() => page.locator(".result-pane-header").innerText())
    .toMatch(/\d+ records/);
  const validUrl = page.url();
  const validFilter = new URL(validUrl).searchParams.get("filter");
  await page.getByRole("textbox", { name: "Level / rank minimum" }).fill("-");
  await expect(
    page.getByText("Enter a finite numeric value", { exact: true }),
  ).toBeVisible();
  expect(page.url()).toBe(validUrl);
  await expect(page.locator(".result-pane-header")).toContainText("records");
  await page.reload();
  await expect(page.getByRole("textbox", { name: "Level / rank minimum" })).toHaveValue(
    "1",
  );
  expect(new URL(page.url()).searchParams.get("filter")).toBe(validFilter);
  await expect(page.getByRole("combobox", { name: "Records per page" })).toBeVisible();
  expect(errors).toEqual([]);
});

test("complex conditions remain advanced with searchable fields and local optional counts", async ({
  page,
}) => {
  const predicate = {
    kind: "any_of",
    clause_id: "root",
    children: [
      {
        kind: "in",
        clause_id: "kind",
        field: "record.kind",
        values: ["creature", "spell"],
      },
    ],
  };
  await page.goto("/search?filter=" + encodeURIComponent(JSON.stringify(predicate)));
  await expect(
    page.getByText("Defined in advanced filters", { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByText("Counts unavailable for this condition").first(),
  ).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
  const importedFilter = new URL(page.url()).searchParams.get("filter");
  await page.getByRole("button", { name: "Edit kind" }).click();
  expect(new URL(page.url()).searchParams.get("filter")).toBe(importedFilter);
  const field = page.getByRole("combobox", { name: "Filter field" });
  await field.fill("Armor Class");
  await field.press("Enter");
  await expect(page.getByRole("textbox", { name: "Armor Class" })).toBeVisible();
  await expect(
    page.getByRole("checkbox", { name: "Exclude this group" }),
  ).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Match conditions" })).toBeVisible();
});

test("narrow advanced filters use a native drawer with keyboard focus return", async ({
  page,
}) => {
  await page.setViewportSize({ width: 650, height: 900 });
  await page.goto("/search");
  const trigger = page.getByRole("button", { name: "Advanced filters", exact: true });
  await trigger.focus();
  await trigger.press("Enter");
  await expect(page.getByRole("dialog", { name: "Advanced filters" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog", { name: "Advanced filters" })).toBeHidden();
  await expect(trigger).toBeFocused();
});

for (const width of [1680, 820]) {
  test(`compact choices focus search on first keyboard open and reopen at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 1000 });
    await page.goto("/search");
    const trigger = page.getByRole("button", {
      name: "Edit Kind filter",
      exact: true,
    });
    await trigger.focus();
    await trigger.press("Enter");
    const menu = page.getByRole("dialog", { name: "Kind choices", exact: true });
    const input = menu.getByRole("textbox", { name: "Search kind choices" });
    await expect(input).toBeFocused();
    await input.fill("Creature");
    await input.press("Tab");
    const row = menu.getByRole("button", { name: /^Kind: Creature,/ });
    await expect(row).toBeFocused();
    await expect(row).toHaveAccessibleName(
      "Kind: Creature, neutral; activate to include",
    );
    await row.press("Enter");
    await expect(row).toBeFocused();
    await expect(row).toHaveAccessibleName(
      "Kind: Creature, included; activate to exclude",
    );
    await row.press("Escape");
    await expect(menu).toBeHidden();
    await expect(trigger).toBeFocused();
    await trigger.press("Enter");
    await expect(input).toBeFocused();
    await expect(input).toHaveValue("");
    await input.press("Escape");
    await expect(menu).toBeHidden();
    await expect(trigger).toBeFocused();
  });
}

test("compact choices keep keyboard focus, mix states and dismiss their mobile drawer", async ({
  page,
}) => {
  await page.setViewportSize({ width: 460, height: 900 });
  await page.goto("/search");
  const trigger = page.getByRole("button", { name: "Edit Kind filter", exact: true });
  await trigger.focus();
  await trigger.press("Enter");
  const drawer = page.getByRole("dialog", { name: "Kind", exact: true });
  await expect(drawer).toBeVisible();
  const input = drawer.getByRole("textbox", { name: "Search kind choices" });
  await expect(input).not.toBeFocused();
  await input.fill("Creature");
  const row = drawer.getByRole("button", {
    name: /^Kind: Creature,/,
  });
  await expect(row).toHaveAccessibleName(
    "Kind: Creature, neutral; activate to include",
  );
  await row.focus();
  await row.press("Enter");
  await expect(row).toBeFocused();
  await expect(row).toHaveAccessibleName(
    "Kind: Creature, included; activate to exclude",
  );
  await expect(row).toHaveText("Creature");
  await row.press("Space");
  await expect(row).toBeFocused();
  await expect(row).toHaveAccessibleName("Kind: Creature, excluded; activate to clear");
  await expect(row).not.toHaveAttribute("aria-pressed");
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
  await expect(trigger).toBeFocused();
  await expect(
    page.getByRole("button", {
      name: "Remove excluded Creature from Kind",
      exact: true,
    }),
  ).toBeVisible();
  await expect(page.locator(".common-choice-options")).toBeHidden();
});

test("remembered Kind is only a default for bare search and clear removes it", async ({
  page,
}) => {
  await page.goto("/search");
  await page.getByRole("button", { name: "Edit Kind filter", exact: true }).click();
  await page
    .getByRole("button", {
      name: "Kind: Creature, neutral; activate to include",
      exact: true,
    })
    .click();
  await page.goto("/search");
  await expect(
    page.getByRole("button", {
      name: "Remove included Creature from Kind",
      exact: true,
    }),
  ).toBeVisible();
  await page.goto("/search?q=heal");
  await expect(
    page.getByRole("button", {
      name: "Remove included Creature from Kind",
      exact: true,
    }),
  ).toHaveCount(0);
  await page.goto("/search");
  await page.getByRole("button", { name: "Clear Kind", exact: true }).click();
  await page.goto("/search");
  await expect(
    page.getByRole("button", {
      name: "Remove included Creature from Kind",
      exact: true,
    }),
  ).toHaveCount(0);
});

test("result browsing uses keyboard rows, committed highlight and native name links", async ({
  page,
}) => {
  await page.goto("/search?q=Ghoul");
  const row = page.locator(".result-row").first();
  await expect(row).toBeVisible();
  await expect(row.getByRole("radio")).toHaveCount(0);
  const name = row.getByRole("link").first();
  await expect(name).toHaveAttribute("href", /\/records\//);
  await expect(row).toHaveAttribute("aria-selected", "false");
  await row.focus();
  await row.press("Enter");
  await expect(row).toHaveAttribute("aria-selected", "true");
  const other = page.locator(".result-row").nth(1);
  if (await other.count()) {
    await other.hover();
    await expect(row).toHaveAttribute("aria-selected", "true");
    await expect(other).toHaveAttribute("aria-selected", "false");
  }
  const searchUrl = page.url();
  const popup = page.context().waitForEvent("page");
  await name.click({ button: "middle" });
  const fullPage = await popup;
  await expect(fullPage).toHaveURL(/\/records\//);
  expect(page.url()).toBe(searchUrl);
  await fullPage.close();
});
