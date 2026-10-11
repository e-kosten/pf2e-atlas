import { test, expect } from "@playwright/test";
import type { GraphContextView, SimilarRecordsView } from "../src/generated/atlas";

test("narrow preview restores focus after closing during opening, reopening and nested Escape", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const name = "Preview lifecycle " + Date.now();
  await page.goto("/encounters");
  await page.getByRole("button", { name: "New Encounter", exact: true }).click();
  const create = page.getByRole("dialog");
  await create.getByLabel("Name", { exact: true }).fill(name);
  await create.getByRole("button", { name: "OK", exact: true }).click();
  await expect(page).toHaveURL(/\/encounters\/preview-lifecycle-/);
  const slug = new URL(page.url()).pathname.split("/")[2];
  try {
    await page.getByRole("button", { name: "Add Creature", exact: true }).click();
    const picker = page.getByRole("dialog");
    await picker.getByRole("textbox", { name: "Search", exact: true }).fill("Ghoul");
    const row = picker
      .getByRole("row")
      .filter({ has: page.getByText("Ghoul", { exact: true }) });
    await row.getByRole("radio").check();
    await picker.getByRole("button", { name: "OK", exact: true }).click();
    await expect(picker).not.toBeVisible();
    await page.getByText("Ghoul", { exact: true }).first().click();
    const jaws = page.getByRole("button", { name: "Jaws", exact: true });
    const preview = page.getByRole("dialog", { name: "Reference", exact: true });
    const openPage = page.getByRole("button", {
      name: "Open reference full page",
      exact: true,
    });

    // Close before the opening callback focuses the first preview action.
    await jaws.focus();
    await jaws.press("Enter");
    await expect(preview).toBeVisible();
    await expect(openPage).not.toBeFocused();
    await page.keyboard.press("Escape");
    await expect(preview).toHaveCount(0);
    await expect(jaws).toBeFocused();

    // The same selection must reopen with fresh lifecycle state.
    await jaws.press("Enter");
    await expect(
      preview.getByRole("heading", { name: "Jaws", exact: true }),
    ).toBeVisible();
    await expect(openPage).toBeFocused();
    await preview
      .getByRole("button", { name: "Close reference preview", exact: true })
      .click();
    await expect(preview).toHaveCount(0);
    await expect(jaws).toBeFocused();

    // The compact root preview intentionally omits the full ability roster.
    // Seize links to Grabbed, whose prepared compact description has a real
    // flat-check control; this exercises a modal nested inside the native Drawer.
    await page.getByRole("button", { name: "Add Creature", exact: true }).click();
    const hazardPicker = page.getByRole("dialog");
    await hazardPicker
      .getByRole("textbox", { name: "Search", exact: true })
      .fill("Waking Nightmare");
    const hazardRow = hazardPicker
      .getByRole("row")
      .filter({ has: page.getByText("Waking Nightmare", { exact: true }) });
    await hazardRow.getByRole("radio").check();
    await hazardPicker.getByRole("button", { name: "OK", exact: true }).click();
    await expect(hazardPicker).not.toBeVisible();
    await page.getByText("Waking Nightmare", { exact: true }).first().click();
    await page.getByRole("button", { name: /^Seize\b/ }).click();
    const grabbedReference = page
      .locator(".actor-ability__detail .prepared-content")
      .getByRole("button", { name: "Grabbed", exact: true });
    await grabbedReference.focus();
    await grabbedReference.press("Enter");
    await expect(
      preview.getByRole("heading", { name: "Grabbed", exact: true }),
    ).toBeVisible();
    const control = preview.locator('[data-atlas-interaction][role="button"]').first();
    await control.focus();
    await control.press("Enter");
    const info = page
      .getByRole("dialog")
      .filter({ has: page.getByRole("button", { name: "OK", exact: true }) });
    await expect(info).toBeVisible();
    await info.getByRole("button", { name: "OK", exact: true }).focus();
    await page.keyboard.press("Escape");
    await expect(info).toHaveCount(0);
    await expect(control).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(preview).toHaveCount(0);
    await expect(grabbedReference).toBeFocused();
    await grabbedReference.press("Enter");
    await expect(openPage).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(preview).toHaveCount(0);
    await expect(grabbedReference).toBeFocused();
  } finally {
    const deleted = await page.request.delete(
      `/api/encounters/${encodeURIComponent(slug)}`,
    );
    expect(deleted.ok()).toBeTruthy();
  }
});

test("reader follows, closes and promotes exact owned selection through browser history", async ({
  page,
}) => {
  await page.goto("/records/pathfinder-bestiary%3ALHHgGSs0ELCR4CYK");
  await page.getByRole("link", { name: "Reader view", exact: true }).click();
  const primary = page.locator(".record-view__pane").first();
  const preview = page.locator(".record-view__pane").last();
  await primary.getByRole("button", { name: "Jaws", exact: true }).click();
  await expect(
    preview.getByRole("heading", { name: "Jaws", exact: true }),
  ).toBeVisible();
  const ownedPreviewUrl = page.url();
  const ownedSelection = new URL(ownedPreviewUrl).searchParams.get("previewSelection");
  expect(ownedSelection).toBeTruthy();
  await preview.getByRole("button", { name: "From Ghoul", exact: true }).click();
  await expect(
    preview.getByRole("heading", { name: "Ghoul", exact: true }),
  ).toBeVisible();
  await page.goBack();
  await expect(page).toHaveURL(ownedPreviewUrl);
  await expect(
    preview.getByRole("heading", { name: "Jaws", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Close preview", exact: true }).click();
  await expect(
    preview.getByText("Select a linked record to preview it."),
  ).toBeVisible();
  await page.goBack();
  await expect(page).toHaveURL(ownedPreviewUrl);
  await page.getByRole("link", { name: "Open preview as reader", exact: true }).click();
  await expect(
    primary.getByRole("heading", { name: "Jaws", exact: true }),
  ).toBeVisible();
  const ownedReaderUrl = page.url();
  expect(new URL(ownedReaderUrl).searchParams.get("selection")).toBe(ownedSelection);
  await primary.getByRole("button", { name: "From Ghoul", exact: true }).click();
  await expect(
    preview.getByRole("heading", { name: "Ghoul", exact: true }),
  ).toBeVisible();
  expect(new URL(page.url()).searchParams.get("selection")).toBe(ownedSelection);
  await page.getByRole("button", { name: "Close preview", exact: true }).click();
  await expect(page).toHaveURL(ownedReaderUrl);
  await page.getByRole("link", { name: "Detail page", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Jaws", exact: true })).toBeVisible();
  expect(new URL(page.url()).searchParams.get("selection")).toBe(ownedSelection);
  await page.goBack();
  await expect(page).toHaveURL(ownedReaderUrl);
  await page.goForward();
  await expect(page.getByRole("heading", { name: "Jaws", exact: true })).toBeVisible();
});

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
    .locator(".atlas-select-popup:visible")
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
  const reference = page.locator(".encounter-reference");
  for (const [label, value] of [
    ["AC", "16"],
    ["Fortitude", "+4"],
    ["Speed", "30 feet"],
  ]) {
    const fact = reference.locator(".record-fact").filter({
      has: page.getByText(label, { exact: true }),
    });
    await expect(fact).toBeVisible();
    await expect(fact.locator("span").first()).toHaveText(value);
  }
  await expect(
    reference.getByRole("heading", { name: "Strikes", exact: true }),
  ).toBeVisible();
  await expect(
    reference.getByRole("heading", { name: "Abilities", exact: true }),
  ).toBeVisible();
  await expect(
    reference.getByRole("button", { name: "Jaws", exact: true }),
  ).toBeVisible();
  await expect(reference.getByRole("button", { name: /^Ghoul Fever/ })).toBeVisible();
  await reference
    .getByRole("button", { name: "More creature facts", exact: true })
    .click();
  const stealth = reference
    .locator(".record-fact")
    .filter({ has: page.getByText("stealth", { exact: true }) });
  await expect(stealth).toBeVisible();
  await expect(stealth.locator("span").first()).toHaveText("+7");
  await reference
    .getByRole("button", { name: "More creature facts", exact: true })
    .click();
  await page.getByRole("textbox", { name: "HP", exact: true }).fill("13");
  await page.getByRole("button", { name: "Set", exact: true }).first().click();
  await page.getByRole("combobox", { name: "Variant", exact: true }).press("ArrowDown");
  await page
    .locator(".atlas-select-popup:visible")
    .getByText("Elite", { exact: true })
    .click();
  await expect(page.getByRole("textbox", { name: "HP", exact: true })).toHaveValue(
    "23",
  );
  await page.getByRole("button", { name: "Add Condition", exact: true }).click();
  const conditionPicker = page.getByRole("combobox", {
    name: "Add condition",
    exact: true,
  });
  await conditionPicker.fill("Clumsy");
  await conditionPicker.press("Enter");
  await expect(page.locator(".encounter-add-condition__row")).toContainText("Clumsy");
  await page.getByRole("button", { name: "Add", exact: true }).click();
  await expect(page.locator(".encounter-condition-list")).toContainText("Clumsy");
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
  await expect(page.locator(".encounter-condition-list")).toContainText("Clumsy");
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
  await page.goto("/search?q=%22Ghoul%20Fever%22");
  await page.getByRole("link", { name: "Ghoul Fever", exact: true }).first().click();
  await expect(page).toHaveURL(/\/records\/.*selection=/);
  await expect(
    page.getByRole("heading", { name: "Ghoul Fever", exact: true }),
  ).toBeVisible();
  await expect(page.getByText(/rises as a Ghoul the next midnight/)).toBeVisible();
  await page.screenshot({
    path: "../../scratch/browser-validation/ghoul-fever.png",
    fullPage: true,
  });
  await page.goto("/search?q=%22Soul%20Degradation%22");
  await page
    .getByRole("link", { name: "Soul Degradation (Curse 7)", exact: true })
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
  await page.goto("/search?q=transform%20into%20a%20dragon");
  await expect(page.getByText(/candidate records/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Dragon Form", exact: true }).first(),
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
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  const advanced = page.getByRole("region", { name: "Advanced filters", exact: true });
  await expect(
    advanced.getByRole("button", { name: "Add condition", exact: true }),
  ).toBeVisible();
  const importedFilter = JSON.parse(new URL(page.url()).searchParams.get("filter")!);
  expect(importedFilter).toEqual({
    kind: "compare",
    clause_id: "tables",
    field: "source.document_kind",
    op: "eq",
    value: "RollTable",
  });
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
  const relatedRequests: string[] = [];
  page.on("request", (request) => {
    const path = new URL(request.url()).pathname;
    if (
      path === "/api/graph" ||
      /\/api\/records\/[^/]+\/(variants|remaster|similar)$/.test(path)
    ) {
      relatedRequests.push(path);
    }
  });
  await page.goto("/records/equipment-srd%3A7jolv0cuttvjI1JD");
  await expect(
    page.getByRole("heading", { name: "Wovenwood Shield (Minor)", exact: true }),
  ).toBeVisible();
  const related = page.getByRole("region", { name: "Related records", exact: true });
  await expect(related.getByRole("tab")).toHaveCount(0);
  expect(relatedRequests).toEqual([]);
  const graphResponse = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/api/graph" &&
      response.request().method() === "POST",
  );
  await related.getByRole("button", { name: "Related records", exact: true }).click();
  await expect(
    related.getByRole("tab", { name: "References", exact: true }),
  ).toBeVisible();
  const graphResult = await graphResponse;
  expect(graphResult.ok()).toBeTruthy();
  const graph: GraphContextView = await graphResult.json();
  expect(relatedRequests).toEqual(["/api/graph"]);
  const variantsResponse = page.waitForResponse((response) =>
    new URL(response.url()).pathname.endsWith("/variants"),
  );
  await related.getByRole("tab", { name: "Variants", exact: true }).click();
  expect((await variantsResponse).ok()).toBeTruthy();
  expect(
    relatedRequests.some(
      (path) => path.endsWith("/similar") || path.endsWith("/remaster"),
    ),
  ).toBe(false);
  await expect(
    related.getByText(
      /These records share a naming pattern and compatible source facts|These suggestions are uncertain; compare the records/,
    ),
  ).toBeVisible();
  await page.getByRole("combobox", { name: "Compare variant", exact: true }).click();
  await page
    .locator(".atlas-select-popup:visible")
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
  await expect(
    comparison.getByRole("button", { name: "Related records", exact: true }),
  ).toHaveCount(0);
  await expect(comparison.getByText("85 gp", { exact: true })).toBeVisible();
  await expect(comparison.getByText("850 gp", { exact: true })).toBeVisible();
  await page.screenshot({
    path: "../../scratch/browser-validation/variant-comparison.png",
    fullPage: true,
  });
  await comparison.getByRole("button", { name: "Close", exact: true }).click();
  await expect(comparison).not.toBeVisible();
  await page.getByRole("combobox", { name: "Compare variant", exact: true }).click();
  await page
    .locator(".atlas-select-popup:visible")
    .getByText("Wovenwood Shield (Moderate)", { exact: true })
    .click();
  await expect(
    comparison.getByRole("heading", {
      name: "Wovenwood Shield (Moderate)",
      exact: true,
    }),
  ).toBeVisible();
  await comparison.getByRole("button", { name: "Close", exact: true }).click();
  await expect(comparison).not.toBeVisible();
  await related.getByRole("tab", { name: "References", exact: true }).click();
  await expect(
    related.getByRole("heading", { name: "Referenced here", exact: true }),
  ).toBeVisible();
  await expect(
    related.getByRole("heading", { name: "References to this record", exact: true }),
  ).toBeVisible();
  const referenceHrefs = await related
    .locator("a[href]")
    .evaluateAll((links) => links.map((link) => (link as HTMLAnchorElement).href));
  for (const occurrence of graph.backlinks.occurrences) {
    expect(
      referenceHrefs.some((href) => {
        const url = new URL(href);
        const selection = url.searchParams.get("selection");
        return (
          decodeURIComponent(url.pathname) ===
            "/records/" + occurrence.source_navigation.record_key &&
          selection !== null &&
          JSON.stringify(JSON.parse(selection)) ===
            JSON.stringify(occurrence.source_navigation)
        );
      }),
    ).toBeTruthy();
  }
  const similarResponse = page.waitForResponse((response) =>
    new URL(response.url()).pathname.endsWith("/similar"),
  );
  await related.getByRole("tab", { name: "Similar", exact: true }).click();
  const similarResult = await similarResponse;
  expect(similarResult.ok()).toBeTruthy();
  const similar: SimilarRecordsView = await similarResult.json();
  if (!similar.results.coverage.exhaustive) {
    const explanation = related.getByRole("button", {
      name: "About these suggestions",
      exact: true,
    });
    await expect(explanation).toBeVisible();
    await expect(related.getByText("More matches may exist.")).not.toBeVisible();
    await explanation.click();
    await expect(related.getByText("More matches may exist.")).toBeVisible();
  }
  if (similar.results.rows.length) {
    for (const row of similar.results.rows) {
      await expect(
        related.getByRole("link", { name: row.record.title, exact: true }),
      ).toBeVisible();
    }
  } else {
    await expect(
      related.getByText("No suggestions found.", { exact: true }),
    ).toBeVisible();
  }
  await page.screenshot({
    path: "../../scratch/browser-validation/similar.png",
    fullPage: true,
  });
});
test("missing keys and malformed selected routes report errors", async ({ page }) => {
  await page.goto("/records/spells-srd%3A0000000000000000");
  await expect(page.getByRole("alert")).toContainText("Record unavailable");
  await expect(page.getByRole("alert")).toContainText(
    "This record is not present in the current reference collection.",
  );
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toBeVisible();
  await page.goto("/records/spells-srd%3A0000000000000000?selection=invalid");
  await expect(page.getByRole("alert")).toContainText("Invalid record navigation URL");
});
