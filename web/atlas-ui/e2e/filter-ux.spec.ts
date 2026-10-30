import { encodeFilterUrl, decodeFilterUrl } from "../src/shared/filters/filterUrl";
import { expect, test, type Page, type Locator } from "@playwright/test";

async function choose(page: Page, control: Locator, label: string) {
  await control.click();
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText(label, { exact: true })
    .last()
    .click();
}

test("single-condition exclusion stays reversible and singleton operands follow the selected operator", async ({
  page,
}) => {
  await page.goto(
    "/search?filter=" +
      encodeURIComponent(
        encodeFilterUrl({
          kind: "not",
          predicate: { kind: "in", field: "kind", values: ["companion"] },
        }),
      ),
  );
  const open = () =>
    page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await open();
  const drawer = page.getByRole("dialog", { name: "Advanced filters", exact: true });
  const operator = drawer.getByRole("combobox", {
    name: "Condition for Kind",
    exact: true,
  });
  const groups = drawer.locator(".ruleGroup");
  for (let i = 0; i < 2; i++) {
    await choose(page, operator, "is any of");
    await expect(groups).toHaveCount(1);
    await expect
      .poll(() => decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!))
      .toMatchObject({ kind: "in", field: "kind", values: ["companion"] });
    await choose(page, operator, "is none of");
    await expect(groups).toHaveCount(1);
    await expect
      .poll(() => decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!))
      .toMatchObject({
        kind: "not",
        predicate: { kind: "in", field: "kind", values: ["companion"] },
      });
  }
  await page.reload();
  await open();
  await expect(operator.locator("..")).toContainText("is none of");
  await choose(page, operator, "is any of");
  await choose(page, operator, "is not");
  await expect
    .poll(() => decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!))
    .toMatchObject({
      kind: "compare",
      field: "kind",
      op: "neq",
      value: "companion",
    });
  await expect(drawer.getByText("Enter a value", { exact: true })).toHaveCount(0);
  await expect(
    drawer.getByRole("combobox", { name: "Kind", exact: true }),
  ).toBeVisible();
});

test("imported collection availability becomes a safe nested editor and its original link stays restorable", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto(
    "/search?filter=" +
      encodeURIComponent(
        encodeFilterUrl({
          kind: "state_match",
          field: "embedded.equipment",
          state: "missing",
        }),
      ),
  );
  const originalUrl = page.url();
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  const drawer = page.getByRole("dialog", { name: "Advanced filters", exact: true });
  const operator = drawer.locator(".rule-operators").getByRole("combobox").first();
  await operator.click();
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText("has", { exact: true })
    .click();
  await expect(drawer.locator(".ruleGroup")).toHaveCount(2);
  await expect(
    drawer.getByRole("button", { name: "Add condition", exact: true }).nth(1),
  ).toBeVisible();
  await expect
    .poll(() => decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!))
    .toMatchObject({
      kind: "exists",
      collection: "embedded.equipment",
      predicate: { kind: "boolean_constant", value: true },
    });
  await page.reload();
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await expect(drawer.locator(".ruleGroup")).toHaveCount(2);
  await page.goto(originalUrl);
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await expect(
    drawer.getByRole("combobox", { name: /availability$/ }).locator(".."),
  ).toContainText("Missing from source");
  await expect(drawer.locator(".ruleGroup")).toHaveCount(1);
  await expect
    .poll(() => decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!))
    .toMatchObject({
      kind: "state_match",
      field: "embedded.equipment",
      state: "missing",
    });
  expect(errors).toEqual([]);
});

test("wrapped values and validation retain the advanced control grid", async ({
  page,
}) => {
  await page.goto(
    "/search?filter=" +
      encodeURIComponent(
        encodeFilterUrl({
          kind: "all_of",
          children: [
            {
              kind: "in",
              field: "kind",
              values: [
                "creature",
                "character",
                "companion",
                "hazard",
                "equipment",
                "feat",
                "spell",
                "affliction",
              ],
            },
            { kind: "compare", field: "defenses.hp.maximum", op: "gte", value: 80 },
          ],
        }),
      ),
  );
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  const drawer = page.getByRole("dialog", { name: "Advanced filters", exact: true });
  await drawer.getByRole("textbox", { name: "Maximum HP", exact: true }).fill("-");
  await expect(
    drawer.getByText("Enter a finite numeric value", { exact: true }),
  ).toBeVisible();
  const rows = await drawer.locator(".rule").all();
  expect(rows).toHaveLength(2);
  for (const row of rows) {
    await expect(row).toHaveCSS("display", "grid");
    const operator = await row.locator(".rule-operators").boundingBox();
    const value = await row.locator(".filter-rule-value").boundingBox();
    const remove = await row.locator(".rule-remove").boundingBox();
    expect(operator).not.toBeNull();
    expect(value).not.toBeNull();
    expect(remove).not.toBeNull();
    expect(
      Math.max(operator!.y, value!.y, remove!.y) -
        Math.min(operator!.y, value!.y, remove!.y),
    ).toBeLessThanOrEqual(2);
    expect(
      await row.evaluate((element) => element.scrollWidth <= element.clientWidth + 1),
    ).toBe(true);
  }
});

test("group exclusion remains one group through edits, toggles and URL reload", async ({
  page,
}) => {
  const common = {
    kind: "in",
    clause_id: "kind",
    field: "kind",
    values: ["creature"],
  };
  const group = {
    kind: "all_of",
    clause_id: "hp-group",
    children: [
      {
        kind: "compare",
        clause_id: "hp-min",
        field: "defenses.hp.maximum",
        op: "gte",
        value: 80,
      },
      {
        kind: "compare",
        clause_id: "hp-max",
        field: "defenses.hp.maximum",
        op: "lte",
        value: 200,
      },
    ],
  };
  await page.goto(
    "/search?filter=" +
      encodeURIComponent(
        encodeFilterUrl({
          kind: "all_of",
          clause_id: "root",
          children: [common, group],
        }),
      ),
  );
  const open = () =>
    page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await open();
  const drawer = page.getByRole("dialog", { name: "Advanced filters", exact: true });
  const groupMatch = drawer.getByRole("combobox", {
    name: "Match conditions",
    exact: true,
  });
  const nestedMatch = groupMatch.nth(1);
  const read = () => {
    const predicate = decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!);
    if (predicate.kind !== "all_of") throw new Error("Expected conjunction");
    return predicate;
  };
  for (let index = 0; index < 3; index++) {
    await expect(groupMatch).toHaveCount(2);
    await choose(page, nestedMatch, "Not all match");
    await expect(nestedMatch.locator("..")).toContainText("Not all match");
    await drawer
      .getByRole("textbox", { name: "Maximum HP", exact: true })
      .first()
      .fill(String(81 + index));
    await expect
      .poll(() => read().children[1])
      .toMatchObject({
        kind: "not",
        predicate: {
          kind: "all_of",
          children: [{ value: 81 + index }, { value: 200 }],
        },
      });
    await expect(groupMatch).toHaveCount(2);
    await choose(page, nestedMatch, "Match all");
    await expect.poll(() => read().children[1].kind).toBe("all_of");
    expect(read().children[0]).toMatchObject({
      kind: "in",
      field: "kind",
      values: ["creature"],
    });
  }
  await choose(page, nestedMatch, "Not all match");
  await expect.poll(() => read().children[1].kind).toBe("not");
  const before = read();
  await page.reload();
  await open();
  await expect(groupMatch).toHaveCount(2);
  await expect(nestedMatch.locator("..")).toContainText("Not all match");
  expect(read()).toEqual(before);
});

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
    .poll(() => decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!))
    .toMatchObject({ kind: "all_of" });
  const mixed = decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!);
  if (mixed.kind !== "all_of") throw new Error("Expected mixed Kind conjunction");
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
  ).toHaveCount(2);
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: /Edit Level \/ rank range/i }).click();
  await page.getByRole("textbox", { name: /Level \/ rank minimum/i }).fill("1");
  await expect
    .poll(() => page.locator(".result-pane-header").innerText())
    .toMatch(/\d+ records/);
  const validUrl = page.url();
  const validFilter = new URL(validUrl).searchParams.get("filter");
  await page.getByRole("textbox", { name: /Level \/ rank minimum/i }).fill("-");
  await expect(
    page
      .getByText("Enter a finite numeric value", { exact: true })
      .filter({ visible: true }),
  ).toBeVisible();
  expect(page.url()).toBe(validUrl);
  await expect(page.locator(".result-pane-header")).toContainText("records");
  await page.reload();
  await page.getByRole("button", { name: /Edit Level \/ rank range/i }).click();
  await expect(
    page.getByRole("textbox", { name: /Level \/ rank minimum/i }),
  ).toHaveValue("1");
  expect(new URL(page.url()).searchParams.get("filter")).toBe(validFilter);
  await expect(page.getByRole("combobox", { name: "Records per page" })).toBeVisible();
  expect(errors).toEqual([]);
});

test("complex conditions remain advanced with searchable fields and no unavailable-count notices", async ({
  page,
}) => {
  const predicate = {
    kind: "any_of",
    clause_id: "root",
    children: [
      {
        kind: "in",
        clause_id: "kind",
        field: "kind",
        values: ["creature", "spell"],
      },
    ],
  };
  await page.goto("/search?filter=" + encodeURIComponent(encodeFilterUrl(predicate)));
  await expect(
    page.getByRole("button", { name: "Kind: Advanced", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Counts unavailable for this condition")).toHaveCount(0);
  await expect(page.getByRole("alert")).toHaveCount(0);
  const importedFilter = new URL(page.url()).searchParams.get("filter");
  await page.getByRole("button", { name: "Kind: Advanced" }).click();
  expect(new URL(page.url()).searchParams.get("filter")).toBe(importedFilter);
  const field = page.getByRole("combobox", { name: "Filter field" });
  await field.fill("Armor Class");
  await field.press("Enter");
  await expect(page.getByRole("textbox", { name: "Armor Class" })).toBeVisible();
  await expect(page.getByRole("combobox", { name: "Match conditions" })).toBeEnabled();
  await expect(page.getByRole("combobox", { name: "Match conditions" })).toBeVisible();
});

test("narrow advanced filters use a native drawer with keyboard focus return", async ({
  page,
}) => {
  await page.setViewportSize({ width: 650, height: 900 });
  await page.goto("/search");
  const trigger = page.getByRole("button", {
    name: "Advanced filters",
    exact: true,
  });
  await trigger.focus();
  await trigger.press("Enter");
  await expect(page.getByRole("dialog", { name: "Advanced filters" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog", { name: "Advanced filters" })).toBeHidden();
  await expect(trigger).toBeFocused();
});

test("Advanced starts with the existing Kind and adding Kind creates exactly one new row", async ({
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
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  const drawer = page.getByRole("dialog", { name: "Advanced filters", exact: true });
  const fields = drawer.getByRole("combobox", { name: "Filter field", exact: true });
  await expect(fields).toHaveCount(1);
  await drawer.getByRole("button", { name: "Add condition", exact: true }).click();
  await expect(fields).toHaveCount(2);
  await fields.nth(1).fill("Kind");
  await fields.nth(1).press("Enter");
  const values = drawer.getByRole("combobox", { name: "Kind", exact: true });
  await values.nth(1).fill("Creature");
  await values.nth(1).press("Enter");
  await expect(fields).toHaveCount(2);
  await expect
    .poll(() => decodeFilterUrl(new URL(page.url()).searchParams.get("filter")!))
    .toMatchObject({
      kind: "all_of",
      children: [
        { kind: "in", field: "kind", values: ["creature"] },
        { kind: "compare", field: "kind", op: "eq", value: "creature" },
      ],
    });
  await drawer
    .getByRole("button", { name: "Remove condition", exact: true })
    .nth(1)
    .click();
  await expect(fields).toHaveCount(1);
  await drawer.getByRole("button", { name: "Close", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Edit Kind filter", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Edit Kind filter", exact: true }).click();
  await page
    .getByRole("button", {
      name: "Kind: Creature, included; activate to exclude",
      exact: true,
    })
    .click();
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await expect(fields).toHaveCount(1);
  await expect(
    drawer
      .getByRole("combobox", { name: "Condition for Kind", exact: true })
      .locator(".."),
  ).toContainText("is none of");
  const url = page.url();
  await page.reload();
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  await expect(fields).toHaveCount(1);
  await expect(
    drawer
      .getByRole("combobox", { name: "Condition for Kind", exact: true })
      .locator(".."),
  ).toContainText("is none of");
  expect(page.url()).toBe(url);
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
    const menu = page.getByRole("dialog", {
      name: "Kind choices",
      exact: true,
    });
    const input = menu.getByRole("textbox", { name: "Search kind choices" });
    await expect(input).toBeFocused();
    await input.fill("Creature");
    await input.press("Tab");
    // Ant's native clear control is a keyboard stop before the choice rows.
    const clear = menu.getByRole("button", {
      name: "close-circle",
      exact: true,
    });
    await expect(clear).toBeFocused();
    await clear.press("Tab");
    const row = menu.getByRole("button", { name: /^Kind: Creature,/ });
    await expect(row).toBeFocused();
    await expect(row).toHaveAccessibleName(
      "Kind: Creature, neutral; activate to include",
    );
    await row.press("Space");
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
  const trigger = page.getByRole("button", {
    name: "Edit Kind filter",
    exact: true,
  });
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
  await expect(row.getByText("Creature", { exact: true })).toBeVisible();
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
  await page
    .getByRole("button", { name: "Remove included Creature from Kind", exact: true })
    .click();
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
  const row = page.locator("[data-record-key]").first();
  await expect(row).toBeVisible();
  await expect(row.getByRole("radio")).toHaveCount(0);
  const name = row;
  await expect(name).toHaveAttribute("href", /\/records\//);
  await expect(row).not.toHaveAttribute("aria-current");
  await row.focus();
  await row.press("Space");
  await expect(row).toHaveAttribute("aria-current", "true");
  const other = page.locator("[data-record-key]").nth(1);
  await expect(other).toBeVisible();
  await other.hover();
  await expect(row).toHaveAttribute("aria-current", "true");
  await expect(other).not.toHaveAttribute("aria-current");
  const searchUrl = page.url();
  const popup = page.context().waitForEvent("page");
  await name.click({ button: "middle" });
  const fullPage = await popup;
  await expect(fullPage).toHaveURL(/\/records\//);
  expect(page.url()).toBe(searchUrl);
  await fullPage.close();
});
