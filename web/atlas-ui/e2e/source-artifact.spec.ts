import { test, expect } from "@playwright/test";
import { encodeFilterUrl, decodeFilterUrl } from "../src/shared/filters/filterUrl";
import type { RelatedRecordsView, SearchResultsPage } from "../src/generated/atlas";

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
    await page.getByRole("button", { name: "Add participants", exact: true }).click();
    await page
      .getByRole("menuitem", { name: "Add creature or hazard…", exact: true })
      .click();
    const picker = page.getByRole("dialog");
    await picker
      .getByRole("searchbox", { name: "Search records", exact: true })
      .fill("Ghoul");
    await picker
      .getByRole("searchbox", { name: "Search records", exact: true })
      .press("Enter");
    await picker.locator("[data-record-key]").filter({ hasText: "Ghoul" }).click();
    await picker.getByRole("button", { name: "Add", exact: true }).click();
    await expect(picker).not.toBeVisible();
    await page.getByText("Ghoul", { exact: true }).first().click();
    const jaws = page.getByRole("button", { name: "Jaws", exact: true });
    const preview = page.getByRole("dialog", {
      name: "Reference",
      exact: true,
    });
    const openPage = page.getByRole("link", {
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

    // Requested Seize prose links to Grabbed, whose prepared description has a real
    // flat-check control; this exercises a modal nested inside the native Drawer.
    await page.getByRole("button", { name: "Add participants", exact: true }).click();
    await page
      .getByRole("menuitem", { name: "Add creature or hazard…", exact: true })
      .click();
    const hazardPicker = page.getByRole("dialog");
    await hazardPicker
      .getByRole("searchbox", { name: "Search records", exact: true })
      .fill("Waking Nightmare");
    await hazardPicker
      .getByRole("searchbox", { name: "Search records", exact: true })
      .press("Enter");
    await hazardPicker
      .locator("[data-record-key]")
      .filter({ hasText: "Waking Nightmare" })
      .click();
    await hazardPicker.getByRole("button", { name: "Add", exact: true }).click();
    await expect(hazardPicker).not.toBeVisible();
    await page.getByText("Waking Nightmare", { exact: true }).first().click();
    const seize = page.getByRole("button", { name: /^Seize\b/ });
    await seize.click();
    await expect(
      preview.getByRole("heading", { name: "Description", exact: true }),
    ).toBeVisible();
    await expect(preview.locator(".prepared-content")).toContainText("Trigger");
    await expect(preview.locator(".prepared-content")).toContainText("Effect");
    const grabbedReference = preview
      .locator(".prepared-content")
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
    await expect(seize).toBeFocused();
    await seize.press("Enter");
    await expect(openPage).toBeFocused();
    const next = preview.getByRole("button", { name: "Grabbed", exact: true });
    await next.focus();
    await next.press("Enter");
    await expect(
      preview.getByRole("heading", { name: "Grabbed", exact: true }),
    ).toBeVisible();
    const content = preview.getByLabel("Reference content", { exact: true });
    await expect(content).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(preview).toHaveCount(0);
    await expect(seize).toBeFocused();
    await seize.press("Enter");
    await expect(openPage).toBeFocused();
    await next.focus();
    await next.press("Enter");
    await expect(
      preview.getByRole("heading", { name: "Grabbed", exact: true }),
    ).toBeVisible();
    await expect(content).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(
      preview.getByRole("button", { name: "DC 5 flat", exact: true }),
    ).toBeFocused();
    await page.keyboard.press("Shift+Tab");
    await expect(
      preview.getByRole("button", {
        name: "Close reference preview",
        exact: true,
      }),
    ).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(preview).toHaveCount(0);
    await expect(seize).toBeFocused();
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
  await preview.getByRole("link", { name: "Return to Ghoul", exact: true }).click();
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
  await primary.getByRole("link", { name: "Return to Ghoul", exact: true }).click();
  await expect(
    preview.getByRole("heading", { name: "Ghoul", exact: true }),
  ).toBeVisible();
  expect(new URL(page.url()).searchParams.get("selection")).toBe(ownedSelection);
  await page.getByRole("button", { name: "Close preview", exact: true }).click();
  const returnedReader = new URL(ownedReaderUrl);
  returnedReader.searchParams.set("landing", ownedSelection!);
  await expect(page).toHaveURL(returnedReader.href);
  expect(new URL(page.url()).searchParams.get("selection")).toBe(ownedSelection);
  expect(new URL(page.url()).searchParams.has("preview")).toBe(false);
  expect(new URL(page.url()).searchParams.has("previewSelection")).toBe(false);
  await page.getByRole("link", { name: "Detail page", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Jaws", exact: true })).toBeVisible();
  expect(new URL(page.url()).searchParams.get("selection")).toBe(ownedSelection);
  await page.goBack();
  await expect(page).toHaveURL(returnedReader.href);
  await page.goForward();
  await expect(page.getByRole("heading", { name: "Jaws", exact: true })).toBeVisible();
});

test("saved list creation and unknown manual participant HP use actual local-state workflows", async ({
  page,
}) => {
  const suffix = Date.now().toString();
  await page.goto("/lists");
  await page.getByRole("button", { name: "New list", exact: true }).click();
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
  // Ant virtualizes long lists; reveal the option rather than assuming it is mounted.
  const listPopup = page.locator(".atlas-select-popup:visible");
  const listOption = listPopup.getByText("Browser List " + suffix, { exact: true });
  await expect
    .poll(async () => {
      if (await listOption.isVisible()) return true;
      await listPopup
        .locator(".ant-select-dropdown-list-holder")
        .evaluate((element) => {
          element.scrollTop += element.clientHeight;
        });
      return false;
    })
    .toBe(true);
  await listOption.click();
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
  await page.getByRole("button", { name: "Add participants", exact: true }).click();
  await page.getByRole("menuitem", { name: "Add PC…", exact: true }).click();
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
  await page.getByRole("button", { name: "Add participants", exact: true }).click();
  await page
    .getByRole("menuitem", { name: "Add creature or hazard…", exact: true })
    .click();
  const creatureDialog = page.getByRole("dialog");
  await creatureDialog
    .getByRole("searchbox", { name: "Search records", exact: true })
    .fill("Ghoul");
  await creatureDialog
    .getByRole("searchbox", { name: "Search records", exact: true })
    .press("Enter");
  await creatureDialog
    .locator("[data-record-key]")
    .filter({ hasText: "Ghoul" })
    .click();
  await creatureDialog.getByRole("button", { name: "Add", exact: true }).click();
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
  await expect(
    page.getByRole("textbox", { name: "Ghoul current HP", exact: true }),
  ).toHaveValue("13");
  const variant = page.getByRole("combobox", { name: "Variant", exact: true });
  await expect(variant).toBeEnabled();
  await variant.press("ArrowDown");
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
  const encounterResponse = await page.request.get(
    `/api/encounters/${encodeURIComponent(new URL(page.url()).pathname.split("/")[2])}`,
  );
  expect(encounterResponse.ok()).toBeTruthy();
  const currentEncounter = await encounterResponse.json();
  const ghoulParticipant = currentEncounter.participants.find(
    (participant: { display_name: string }) => participant.display_name === "Ghoul",
  );
  const jawsActivity = ghoulParticipant.presentation.body.value.activities.value.find(
    (activity: { title: string }) => activity.title === "Jaws",
  );
  await reference.getByRole("button", { name: "Jaws", exact: true }).click();
  const activityPreview = page.getByRole("dialog", {
    name: "Reference preview",
    exact: true,
  });
  await expect(
    activityPreview.getByRole("heading", { name: "Jaws", exact: true }),
  ).toBeVisible();
  await expect(
    activityPreview
      .locator(".record-fact")
      .filter({ has: page.getByText("Attack", { exact: true }) }),
  ).toContainText("+" + jawsActivity.attack.value);
  await expect(
    activityPreview.getByRole("heading", { name: "Description", exact: true }),
  ).toHaveCount(0);
  for (const note of jawsActivity.notes)
    await expect(activityPreview).toContainText(note.reason);
  await page
    .getByRole("button", { name: "Close reference preview", exact: true })
    .click();
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
  const ghoul = page.locator(".reference-result").filter({
    has: page.locator('[data-record-key="pathfinder-bestiary:LHHgGSs0ELCR4CYK"]'),
  });
  await ghoul.getByRole("button", { name: /matches in Ghoul$/ }).click();
  await page
    .locator(".reference-match-list:visible")
    .getByRole("link", { name: "Ghoul Fever", exact: true })
    .first()
    .click();
  await page
    .getByRole("link", { name: "Open Ghoul Fever full reference", exact: true })
    .click();
  await expect(page).toHaveURL(/\/records\/.*selection=/);
  const fever = page.getByRole("heading", { name: /^Ghoul Fever\b/ });
  await expect(fever).toBeVisible();
  await expect(fever).toHaveText("Ghoul Fever passive");
  await expect(fever.locator('.action-cost[title="passive"]')).toHaveCount(1);
  await expect(fever.locator("[data-atlas-action-glyph]")).toHaveCount(0);
  await expect(page.getByText(/rises as a Ghoul the next midnight/)).toBeVisible();
  await page.screenshot({
    path: "../../scratch/browser-validation/ghoul-fever.png",
    fullPage: true,
  });
  await page.goto("/search?q=%22Soul%20Degradation%22");
  await page
    .locator(".reference-result")
    .first()
    .getByRole("button", { name: /matches in/ })
    .click();
  await page
    .locator(".reference-match-list:visible")
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
  await expect(page.getByText(/^Top [\d,]+ matches$/)).toBeVisible();
  await expect(
    page.locator("[data-record-key]").filter({ hasText: "Dragon Form" }).first(),
  ).toBeVisible();
  await page.goto(
    "/search?filter=" +
      encodeURIComponent(
        encodeFilterUrl({
          kind: "compare",
          clause_id: "tables",
          field: "source.document_kind",
          op: "eq",
          value: "RollTable",
        }),
      ),
  );
  await expect(
    page.getByRole("region", { name: "Search results", exact: true }),
  ).toBeVisible();
  await expect(page.getByText(/Wellspring Surges/).first()).toBeVisible();
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  const advanced = page.getByRole("region", {
    name: "Advanced filters",
    exact: true,
  });
  await expect(
    advanced.getByRole("button", { name: "Add condition", exact: true }),
  ).toBeVisible();
  const importedFilter = decodeFilterUrl(
    new URL(page.url()).searchParams.get("filter")!,
  );
  expect(importedFilter).toMatchObject({
    kind: "compare",
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
  await page.getByRole("button", { name: "Result 1", exact: true }).click();
  await expect(page).not.toHaveURL(/selection=/);
  const preview = page.getByRole("dialog", {
    name: "Reference preview",
    exact: true,
  });
  await expect(preview.locator(".prepared-content")).toContainText("Energy Unleashed");
  const fullPage = page.getByRole("link", {
    name: "Open reference full page",
    exact: true,
  });
  await expect(fullPage).toHaveAttribute("href", /selection=/);
  await fullPage.click();
  await expect(page).toHaveURL(/selection=/);
  await expect(page.locator(".prepared-content")).toContainText("Energy Unleashed");
  await page.screenshot({
    path: "../../scratch/browser-validation/rolltable-detail.png",
    fullPage: true,
  });
});
test("versions compare complete records and Similar hands off to Search with browser history", async ({
  page,
}) => {
  const seed = "equipment-srd:7jolv0cuttvjI1JD";
  const relatedRequests: string[] = [];
  page.on("request", (request) => {
    if (new URL(request.url()).pathname === "/api/related")
      relatedRequests.push(request.url());
  });
  await page.goto(`/records/${seed}`);
  await expect(
    page.getByRole("heading", { name: "Wovenwood Shield (Minor)", exact: true }),
  ).toBeVisible();
  expect(relatedRequests).toHaveLength(0);
  const response = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/api/related" &&
      response.request().method() === "POST",
  );
  await page
    .getByRole("button", { name: "Explore related records", exact: true })
    .click();
  const availability: RelatedRecordsView = await (await response).json();
  expect(availability.variants.kind).toBe("content");
  const explore = page.getByRole("region", { name: "Explore record", exact: true });
  await explore.getByRole("tab", { name: "Versions", exact: true }).click();
  const variant = explore.locator(".reference-result").filter({
    has: page.getByRole("link", { name: /^Wovenwood Shield \(Moderate\)/ }),
  });
  const compare = variant.getByRole("button", { name: "Compare", exact: true });
  await compare.click();
  const dialog = page.getByRole("dialog", { name: "Variant comparison", exact: true });
  await expect(
    dialog.getByRole("heading", { name: "Wovenwood Shield (Minor)", exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByRole("heading", { name: "Wovenwood Shield (Moderate)", exact: true }),
  ).toBeVisible();
  await expect(dialog.getByText("85 gp", { exact: true })).toBeVisible();
  await expect(dialog.getByText("850 gp", { exact: true })).toBeVisible();
  await expect(
    dialog.getByRole("button", { name: "Explore related records", exact: true }),
  ).toHaveCount(0);
  await dialog.press("Escape");
  await expect(dialog).not.toBeVisible();
  await expect(compare).toBeFocused();
  await compare.press("Enter");
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Close", exact: true }).click();
  await expect(compare).toBeFocused();
  const similarResponse = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname === "/api/search/page" &&
      response.request().postDataJSON()?.criteria?.kind === "similar_records",
  );
  await explore.getByRole("tab", { name: "Similar", exact: true }).click();
  const similar: SearchResultsPage = await (await similarResponse).json();
  expect(similar.mode.kind).toBe("similar_records");
  expect(similar.page.size).toBe(5);
  expect(similar.rows.every((row) => row.record.record_key !== seed)).toBe(true);
  await expect(explore.locator(".reference-result")).toHaveCount(similar.rows.length);
  await expect(
    explore.getByRole("button", { name: /Next page|See more/i }),
  ).toHaveCount(0);
  const search = explore.getByRole("link", {
    name: "Search similar records",
    exact: true,
  });
  await expect(search).not.toHaveClass(/ant-btn-primary/);
  await search.click();
  expect(new URL(page.url()).searchParams.get("similar")).toBe(seed);
  await expect(
    page.getByRole("searchbox", { name: "Similar search", exact: true }),
  ).toHaveValue("Wovenwood Shield (Minor)");
  await expect(page.getByText("Top matches", { exact: true })).toBeVisible();
  await page.reload();
  await expect(
    page.getByRole("searchbox", { name: "Similar search", exact: true }),
  ).toHaveValue("Wovenwood Shield (Minor)");
  await page.goBack();
  await expect(
    explore.getByRole("tab", { name: "Similar", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await page.goBack();
  await expect(
    page.getByRole("button", { name: "Explore related records", exact: true }),
  ).toBeVisible();
  await expect(explore).not.toBeVisible();
  await page.goForward();
  await expect(
    explore.getByRole("tab", { name: "Similar", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await search.click();
  await page
    .getByRole("button", { name: "Clear similar criterion", exact: true })
    .click();
  await expect(page).not.toHaveURL(/similar=/);
  await page
    .getByRole("searchbox", { name: "Search records", exact: true })
    .fill("dragon");
  await page
    .getByRole("searchbox", { name: "Search records", exact: true })
    .press("Enter");
  await expect(
    page.getByRole("searchbox", { name: "Search records", exact: true }),
  ).toHaveValue("dragon");
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
