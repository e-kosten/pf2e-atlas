import { test, expect } from "@playwright/test";
test("saved list creation and unknown manual participant HP use actual local-state workflows", async ({
  page,
}) => {
  const suffix = Date.now().toString();
  await page.goto("/lists");
  await page.getByRole("button", { name: "New List", exact: true }).click();
  const listDialog = page.getByRole("dialog");
  await listDialog.getByLabel("Name", { exact: true }).fill("Browser List " + suffix);
  await listDialog.getByRole("button", { name: "Create", exact: true }).click();
  await expect(page).toHaveURL(/\/lists\/browser-list-/);
  const listUrl = page.url();
  await expect(
    page.getByText("Browser List " + suffix, { exact: true }).first(),
  ).toBeVisible();
  await page.reload();
  await expect(
    page.getByText("Browser List " + suffix, { exact: true }).first(),
  ).toBeVisible();
  await page.goto("/records/pathfinder-bestiary%3ALHHgGSs0ELCR4CYK");
  await page.getByRole("button", { name: "Add to saved list", exact: true }).click();
  await page.getByRole("dialog").getByRole("combobox").click();
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText("Browser List " + suffix, { exact: true })
    .click();
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Add", exact: true })
    .click();
  await expect(page.getByRole("dialog")).not.toBeVisible();
  await page.goto(listUrl);
  await expect(page.getByText("Ghoul", { exact: true }).first()).toBeVisible();
  await page.reload();
  await expect(page.getByText("Ghoul", { exact: true }).first()).toBeVisible();
  await page.goto("/encounters");
  await page.getByRole("button", { name: /New Encounter/i }).click();
  const encounterDialog = page.getByRole("dialog");
  await encounterDialog
    .getByLabel("Name", { exact: true })
    .fill("Browser Encounter " + suffix);
  await encounterDialog.getByRole("button", { name: "OK", exact: true }).click();
  await expect(page).toHaveURL(/\/encounters\/browser-encounter-/);
  await page.getByRole("button", { name: "Add PC", exact: true }).click();
  const pcDialog = page.getByRole("dialog");
  await pcDialog.getByLabel("Name", { exact: true }).fill("Unknown Hero");
  await pcDialog.getByRole("button", { name: "OK", exact: true }).click();
  const hp = page.getByRole("textbox", { name: "HP", exact: true });
  await expect(hp).toBeVisible();
  await expect(hp).toHaveValue("");
  await expect(
    page.getByRole("button", { name: "Damage", exact: true }),
  ).toBeDisabled();
  await expect(page.getByRole("button", { name: "Heal", exact: true })).toBeDisabled();
  await hp.fill("12");
  await page.getByRole("button", { name: "Set", exact: true }).first().click();
  await expect(hp).toHaveValue("12");
  await expect(page.getByRole("button", { name: "Damage", exact: true })).toBeEnabled();
  await page.reload();
  await page.getByText("Unknown Hero", { exact: true }).first().click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "12",
  );
  await page.screenshot({
    path: "../../scratch/browser-validation/manual-hp.png",
    fullPage: true,
  });
  await page.getByRole("button", { name: "Add Creature", exact: true }).click();
  const creatureDialog = page.getByRole("dialog");
  await creatureDialog
    .getByRole("textbox", { name: "Search", exact: true })
    .fill("Ghoul");
  await creatureDialog.getByText("Ghoul", { exact: true }).click();
  await creatureDialog.getByRole("button", { name: "OK", exact: true }).click();
  await expect(page.getByRole("dialog")).not.toBeVisible();
  await page.getByText("Ghoul", { exact: true }).first().click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "20",
  );
  for (const name of ["Defenses", "Saves", "Skills", "Senses & Movement"])
    await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
  for (const label of ["AC", "fortitude", "stealth", "land"])
    await expect(page.getByText(label, { exact: true }).first()).toBeVisible();
  await page.getByRole("textbox", { name: "HP", exact: true }).fill("13");
  await page.getByRole("button", { name: "Set", exact: true }).first().click();
  await page.getByRole("combobox", { name: "Variant", exact: true }).press("ArrowDown");
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText("Elite", { exact: true })
    .click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "23",
  );
  await page.getByRole("button", { name: "Add Condition", exact: true }).click();
  await page.getByRole("combobox", { name: "Add condition", exact: true }).click();
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText("Clumsy", { exact: true })
    .click();
  await page.getByRole("button", { name: "Add", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "23",
  );
  await page.getByRole("spinbutton", { name: "HP change", exact: true }).fill("3");
  await page.getByRole("button", { name: "Damage", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "20",
  );
  await page.getByRole("spinbutton", { name: "HP change", exact: true }).fill("3");
  await page.getByRole("button", { name: "Heal", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "23",
  );
  await page.reload();
  await page.getByText("Ghoul", { exact: true }).first().click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "23",
  );
  await page.screenshot({
    path: "../../scratch/browser-validation/ghoul-encounter.png",
    fullPage: true,
  });
});
test("owned Ghoul Fever and named GM passage use actual selected detail routes", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/search?q=%22Ghoul%20Fever%22&mode=text_search&retrieval=lexical");
  await page.getByRole("button", { name: "Ghoul Fever", exact: true }).first().click();
  await expect(page).toHaveURL(/\/records\/.*selection=/);
  await expect(
    page.getByRole("heading", { name: "Ghoul Fever", exact: true }),
  ).toBeVisible();
  await expect(page.getByText(/rises as a Ghoul the next midnight/)).toBeVisible();
  await page.screenshot({
    path: "../../scratch/browser-validation/ghoul-fever.png",
    fullPage: true,
  });
  await page.goto(
    "/search?q=%22Soul%20Degradation%22&mode=text_search&retrieval=lexical",
  );
  await page
    .getByRole("button", { name: "Soul Degradation (Curse 7)", exact: true })
    .first()
    .click();
  await expect(page).toHaveURL(/selection=/);
  await expect(
    page.getByRole("heading", { name: /Soul Degradation/ }).first(),
  ).toBeVisible();
  await expect(page.locator(".prepared-content")).toContainText("Soul Degradation");
  await page.screenshot({
    path: "../../scratch/browser-validation/soul-degradation.png",
    fullPage: true,
  });
  expect(errors).toEqual([]);
});
test("semantic coverage, typed catalog and RollTables remain product surfaces", async ({
  page,
}) => {
  await page.goto(
    "/search?q=transform%20into%20a%20dragon&mode=text_search&retrieval=semantic",
  );
  await expect(page.getByText(/candidate records/)).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Dragon Form", exact: true }).first(),
  ).toBeVisible();
  await page.goto(
    "/search?filter=" +
      encodeURIComponent(
        JSON.stringify({
          kind: "compare",
          clause_id: "tables",
          field: "source.document_kind",
          op: "eq",
          value: "RollTable",
        }),
      ),
  );
  await expect(page.getByRole("columnheader", { name: "Name" })).toBeVisible();
  await expect(page.getByText(/Wellspring Surges/).first()).toBeVisible();
  await expect(page.getByRole("button", { name: "+ Rule", exact: true })).toBeVisible();
  await expect(page.getByText("Updating results", { exact: true })).not.toBeVisible();
  await page.screenshot({
    path: "../../scratch/browser-validation/rolltables-filter.png",
    fullPage: true,
  });
  await page.goto("/records/rollable-tables%3AN0pDFNCffTe2Du39");
  await expect(
    page.getByRole("heading", { name: "Wellspring Surges", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Result 1–1", exact: true }).click();
  await expect(page).toHaveURL(/selection=/);
  await expect(page.locator(".prepared-content")).toContainText("Energy Unleashed");
  await page.screenshot({
    path: "../../scratch/browser-validation/rolltable-detail.png",
    fullPage: true,
  });
});
test("suggested variants compare authored details and Similar uses its real route", async ({
  page,
}) => {
  await page.goto("/records/equipment-srd%3A7jolv0cuttvjI1JD");
  await page.getByRole("button", { name: "Suggested variants", exact: true }).click();
  await expect(
    page.getByText(
      "Suggestions share naming and compatible source facts; they do not create aliases.",
    ),
  ).toBeVisible();
  await page.getByRole("combobox", { name: "Compare variant", exact: true }).click();
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText("Wovenwood Shield (Moderate)", { exact: true })
    .click();
  const comparison = page.getByRole("dialog");
  await expect(
    comparison.getByRole("heading", { name: "Wovenwood Shield (Minor)", exact: true }),
  ).toBeVisible();
  await expect(
    comparison.getByRole("heading", {
      name: "Wovenwood Shield (Moderate)",
      exact: true,
    }),
  ).toBeVisible();
  await expect(comparison).not.toHaveClass(/ant-zoom/);
  await page.screenshot({
    path: "../../scratch/browser-validation/variant-comparison.png",
    fullPage: true,
  });
  await comparison.getByRole("button", { name: "Close", exact: true }).click();
  await page.getByRole("button", { name: "Connections", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Outgoing", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Similar", exact: true }).click();
  await expect(page.getByText(/candidate units/)).toBeVisible();
  await page.screenshot({
    path: "../../scratch/browser-validation/similar.png",
    fullPage: true,
  });
});
test("missing keys and malformed selected routes report errors", async ({ page }) => {
  await page.goto("/records/spells-srd%3A0000000000000000");
  await expect(page.getByRole("alert")).toContainText(/not found/i);
  await page.goto("/records/spells-srd%3A0000000000000000?selection=invalid");
  await expect(page.getByRole("alert")).toContainText("Invalid record navigation URL");
});
