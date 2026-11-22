import type { APIRequestContext } from "@playwright/test";
import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";
import type {
  EncounterCreateView,
  PartyIndexView,
  PartyView,
  PcReferenceView,
} from "../../src/generated/atlas";

function exportedCharacter(name: string, level = 7) {
  return JSON.stringify({
    success: true,
    build: {
      name,
      level,
      keyability: "int",
      abilities: { str: 10, dex: 16, con: 14, int: 18, wis: 12, cha: 8 },
      attributes: { ancestryhp: 8, classhp: 6, bonushp: 0, bonushpPerLevel: 0 },
      proficiencies: { stealth: 2, athletics: 0 },
      equipment: [
        ["Rope", 2, "Not Invested"],
        ["Rope", 1, "bag", "Not Invested"],
      ],
      equipmentContainers: { bag: { containerName: "Backpack", backpack: true } },
      feats: [],
      specials: ["Darkvision", "Night sight"],
      spellCasters: [],
    },
  });
}

async function createParty(
  request: APIRequestContext,
  name: string,
): Promise<PartyView> {
  const response = await request.post("/api/parties", { data: { name, members: [] } });
  expect(response.ok()).toBe(true);
  return response.json();
}

async function createEncounter(request: APIRequestContext, name: string) {
  const response = await request.post("/api/encounters", { data: { name } });
  expect(response.ok()).toBe(true);
  const { encounter } = (await response.json()) as EncounterCreateView;
  return encounter;
}

async function deleteCharacters(request: APIRequestContext, keys: string[]) {
  for (const key of keys) {
    const path = `/api/characters/${key}`;
    const response = await request.get(path);
    if (response.ok()) {
      const { character_revision } = (await response.json()) as PcReferenceView;
      await request.delete(path, { data: { expected_revision: character_revision } });
    }
  }
}

async function deleteParties(request: APIRequestContext, ...keys: string[]) {
  const { parties } = (await (
    await request.get("/api/parties")
  ).json()) as PartyIndexView;
  for (const { party_key, revision } of parties) {
    if (keys.includes(party_key))
      await request.delete(`/api/parties/${party_key}`, {
        data: { expected_revision: revision },
      });
  }
}

test("Pathbuilder character rules stay shared between parties and encounters", async ({
  page,
}, info) => {
  const name = `Pathbuilder ${info.project.name} ${Date.now()}`;
  const party = await createParty(page.request, name);
  const second = await createParty(page.request, `${name} current`);
  const encounter = await createEncounter(page.request, name);
  let characterKey: string | undefined;
  let releaseSearch: (() => void) | undefined;
  try {
    await page.goto("/encounters");
    const currentView = page.getByRole("banner").locator('a[aria-current="page"]');
    const characterPages = page.getByRole("navigation", { name: "Character pages" });
    const currentPage = characterPages.locator('a[aria-current="page"]');
    await expect(currentView).toHaveText(["Encounters"]);
    await expect(characterPages).toHaveCount(0);
    await activate(page.getByRole("link", { name: "Characters", exact: true }), info);
    await expect(currentView).toHaveText(["Characters"]);
    await expect(currentPage).toHaveText(["All characters"]);
    await activate(page.getByRole("link", { name: "Parties", exact: true }), info);
    const directory = page.getByRole("navigation", {
      name: "Parties",
      includeHidden: true,
    });
    const mountedDirectory = await directory.elementHandle();
    const chooseParty = () =>
      activate(page.locator(`a[href="/parties/${party.party_key}"]`), info);
    await chooseParty();
    await expect(currentPage).toHaveText(["Parties"]);
    await expect(
      page.getByRole("heading", { name: party.name, exact: true }),
    ).toBeVisible();
    expect(
      await directory.evaluate((node, original) => node === original, mountedDirectory),
    ).toBe(true);
    const backToParties = page.getByRole("link", { name: "Back to parties" });
    if ((page.viewportSize()?.width ?? 0) < 768) {
      await expect(directory).toBeHidden();
      await activate(backToParties, info);
      await expect(
        page.getByRole("heading", { name: "Parties", exact: true }),
      ).toBeVisible();
      await expect(page.locator(`a[href="/parties/${party.party_key}"]`)).toBeFocused();
      await chooseParty();
    } else {
      await activate(page.locator(`a[href="/parties/${second.party_key}"]`), info);
      await expect(
        page.getByRole("heading", { name: second.name, exact: true }),
      ).toBeVisible();
      await chooseParty();
      expect(
        await directory.evaluate(
          (node, original) => node === original,
          mountedDirectory,
        ),
      ).toBe(true);
    }
    await noPageOverflow(page);
    await activate(page.getByRole("button", { name: "Add PC", exact: true }), info);
    await activate(page.getByRole("menuitem", { name: "From Pathbuilder" }), info);
    const importer = page.getByRole("dialog", { name: "Import from Pathbuilder" });
    await importer.getByLabel("Pathbuilder JSON").fill(exportedCharacter("Aster"));
    await activate(
      importer.getByRole("button", { name: "Continue", exact: true }),
      info,
    );
    await expect(
      importer.getByRole("textbox", { name: "Pathbuilder JSON" }),
    ).toBeHidden();
    await expect(
      importer.getByRole("heading", { name: /Aster.*Level 7/ }),
    ).toBeVisible();
    await expect(importer.getByRole("heading", { name: /^Features/ })).toBeVisible();
    await expect(importer.getByText("Darkvision", { exact: true })).toHaveCount(1);
    await expect(importer.getByText("1 linked · 1 unlinked")).toBeVisible();
    await activate(
      importer.getByRole("button", { name: "Change link for Darkvision" }),
      info,
    );
    const importResult = importer.getByRole("link", { name: /^Darkvision\b/ });
    if (info.project.use.hasTouch) await activate(importResult, info);
    else await importResult.press("Enter");
    await expect(
      importer.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await activate(importer.getByRole("button", { name: "Link this rule" }), info);
    await expect(importer.getByRole("region", { name: "PC abilities" })).toBeVisible();
    await activate(importer.getByRole("button", { name: "Add to party" }), info);
    await expect(page).toHaveURL(new RegExp(`/parties/${party.party_key}`));
    const importedRow = page.getByRole("row").filter({ hasText: "Aster" });
    const importedLink = importedRow.getByRole("link", { name: "Aster", exact: true });
    const characterPath = await importedLink.getAttribute("href");
    characterKey = characterPath!.split("/").filter(Boolean).at(-1)!;
    await activate(importedLink, info);
    await expect(page.getByRole("tab", { name: "Stats", exact: true })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await expect(page.getByText("Max HP", { exact: true })).toBeVisible();
    await expect(page.getByText("Intelligence", { exact: true }).first()).toBeVisible();
    await activate(page.getByRole("button", { name: "Show details for Max HP" }), info);
    const breakdown = page.getByRole("dialog", {
      name: "Show details for Max HP",
    });
    await expect(breakdown.getByText("64", { exact: true })).toBeVisible();
    await expect(
      breakdown.getByText("Constitution per level", { exact: true }),
    ).toBeVisible();
    await activate(breakdown.getByRole("button", { name: "Close explanation" }), info);
    const skills = page.getByRole("heading", { name: "Skills", exact: true });
    const jump = page.getByRole("link", { name: "Jump to Skills" });
    if (info.project.name.startsWith("phone-")) {
      const beforeJump = page.url();
      await activate(jump, info);
      await expect(skills).toBeFocused();
      await expect(skills).toBeInViewport();
      await expect(page).toHaveURL(beforeJump);
    } else if (info.project.name === "desktop-chromium") {
      await expect(jump).toBeHidden();
    }
    await activate(page.getByRole("tab", { name: "Inventory", exact: true }), info);
    await expect(
      page.getByRole("heading", { name: "Backpack", exact: true }),
    ).toBeVisible();
    await expect(
      page
        .getByRole("tabpanel", { name: "Inventory" })
        .getByText("Rope", { exact: true }),
    ).toHaveCount(2);
    await activate(page.getByRole("tab", { name: "Abilities", exact: true }), info);
    const imported = (await (
      await page.request.get(`/api/characters/${characterKey}`)
    ).json()) as PcReferenceView;
    expect(imported).not.toHaveProperty("abilities");
    expect(
      imported.sections
        .flatMap((section) => section.abilities)
        .find((ability) => ability.label === "Darkvision")?.manual,
    ).toBe(true);
    const reference = page.locator(".pc-ability-browser");
    const darkvision = reference.getByRole("button", {
      name: "Darkvision",
      exact: true,
    });
    await expect(reference.getByRole("heading", { name: /^Features/ })).toBeVisible();
    const filter = reference.getByRole("textbox", { name: "Find an ability" });
    await filter.fill("Darkvision");
    const characterUrl = page.url();
    const shell = page.locator("#atlas-main");
    expect(
      await shell.evaluate((node) => node.scrollHeight - node.clientHeight),
    ).toBeLessThanOrEqual(1);
    await activate(darkvision, info);
    await expect(page).toHaveURL(/\?ability=/);
    await expect(
      reference.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    expect(
      await shell.evaluate((node) => node.scrollHeight - node.clientHeight),
    ).toBeLessThanOrEqual(1);
    const abilityUrl = page.url();
    await page.goBack();
    await expect(page).toHaveURL(characterUrl);
    await expect(filter).toHaveValue("Darkvision");
    await expect(darkvision).toBeFocused();
    await page.goForward();
    await expect(page).toHaveURL(abilityUrl);
    await expect(currentView).toHaveText(["Characters"]);
    await expect(currentPage).toHaveText(["All characters"]);
    await expect(
      reference.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    const backToAbilities = reference.getByRole("button", {
      name: "Back to abilities",
      exact: true,
    });
    if (info.project.name === "desktop-chromium") {
      await expect(backToAbilities).toHaveCount(0);
      await expect(
        reference.getByRole("region", { name: "PC abilities" }),
      ).toBeVisible();
      await activate(reference.getByRole("button", { name: "Clear selection" }), info);
    } else {
      await activate(backToAbilities, info);
    }
    await expect(page).toHaveURL(characterUrl);
    await expect(darkvision).toBeFocused();
    await activate(
      reference.getByRole("button", { name: "Link options for Darkvision" }),
      info,
    );
    await expect(
      page.getByRole("menuitem", { name: "Use automatic match" }),
    ).toBeVisible();
    await page.keyboard.press("Escape");
    await filter.fill("sight");
    let started!: () => void;
    const requestStarted = new Promise<void>((resolve) => {
      started = resolve;
    });
    const holdSearch = new Promise<void>((resolve) => {
      releaseSearch = resolve;
    });
    let held = false;
    await page.route("**/api/search/page", async (route) => {
      if (!held && route.request().postDataJSON()?.criteria?.query === "Night sight") {
        held = true;
        started();
        await holdSearch;
      }
      await route.continue();
    });
    await activate(
      reference.getByRole("button", { name: "Search for Night sight" }),
      info,
    );
    await requestStarted;
    const task = reference.getByRole("region", { name: "Ability search" });
    await expect(
      task.getByRole("heading", { name: "Find rule for Night sight" }),
    ).toBeVisible();
    await expect(task.getByText("Searching Atlas…", { exact: true })).toBeVisible();
    await expect(task.getByText(/No rules found/)).toHaveCount(0);
    await expect(task.getByRole("link", { name: "Open full search" })).toHaveCount(0);
    const pendingBounds = (await task.boundingBox())!;
    releaseSearch!();
    await expect(task.locator(".reference-results__scroll")).toHaveAttribute(
      "aria-busy",
      "false",
    );
    const loadedBounds = (await task.boundingBox())!;
    expect(Math.abs(pendingBounds.height - loadedBounds.height)).toBeLessThan(1);
    const search = reference.getByLabel("Search Atlas rules");
    await search.fill("Darkvision");
    await search.press("Enter");
    await expect(page).toHaveURL(/q=Darkvision/);
    const lookupUrl = page.url();
    await page.goBack();
    await expect(page).toHaveURL(characterUrl);
    await expect(filter).toHaveValue("sight");
    await expect(
      reference.getByRole("button", { name: "Search for Night sight" }),
    ).toBeFocused();
    await page.goForward();
    await expect(page).toHaveURL(lookupUrl);
    await expect(search).toHaveValue("Darkvision");
    const result = task.getByRole("link", { name: /^Darkvision\b/ });
    await activate(result, info);
    await expect(
      reference.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    const ruleIdentity = task.locator(".record-reference__header").filter({
      has: page.getByRole("heading", { name: "Darkvision", exact: true }),
    });
    await expect(ruleIdentity.getByRole("button", { name: "Copy link" })).toBeVisible();
    await expect(
      ruleIdentity.getByRole("link", { name: "Open full page" }),
    ).toBeVisible();
    const results = reference.getByRole("button", {
      name: "Results",
      exact: true,
    });
    if (await results.isVisible()) {
      await activate(results, info);
      await expect(search).toHaveValue("Darkvision");
      await activate(result, info);
    } else {
      await expect(search).toBeVisible();
      await expect(search).toHaveValue("Darkvision");
    }
    await activate(reference.getByRole("button", { name: "Link this rule" }), info);
    await expect(
      reference.getByRole("button", { name: "Night sight", exact: true }),
    ).toBeVisible();
    await activate(
      reference.getByRole("button", { name: "Link options for Night sight" }),
      info,
    );
    await expect(page.getByRole("menuitem", { name: "Change link" })).toBeVisible();
    await expect(
      page.getByRole("menuitem", { name: "Use automatic match" }),
    ).toHaveCount(0);
    await page.keyboard.press("Escape");
    if (info.project.name === "desktop-chromium") {
      const viewport = page.viewportSize()!;
      await page.setViewportSize({ width: 500, height: 320 });
      await filter.fill("Darkvision");
      await activate(darkvision, info);
      const ruleText = reference.getByText("Fixture darkvision reference content.");
      await ruleText.scrollIntoViewIfNeeded();
      await expect(ruleText).toBeInViewport();
      expect(
        await reference
          .locator(".reference-reader__scroll")
          .evaluate((node) => node.scrollHeight - node.clientHeight),
      ).toBeLessThanOrEqual(1);
      await activate(
        reference.getByRole("button", { name: "Back to abilities", exact: true }),
        info,
      );
      await page.setViewportSize(viewport);
    }
    await page.goto(abilityUrl);
    await expect(
      reference.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await expect(currentView).toHaveText(["Characters"]);
    await expect(currentPage).toHaveText(["All characters"]);
    const saved = (await (
      await page.request.get(`/api/characters/${characterKey}`)
    ).json()) as PcReferenceView;
    const joined = await page.request.patch(`/api/parties/${second.party_key}`, {
      data: {
        name: second.name,
        expected_revision: second.revision,
        members: [
          {
            character_key: characterKey,
            expected_character_revision: saved.character_revision,
          },
        ],
      },
    });
    expect(joined.ok()).toBe(true);
    await page.goto(`/parties/${second.party_key}`);
    await activate(page.getByRole("link", { name: "Aster", exact: true }), info);
    await expect(page).toHaveURL(new RegExp(`/characters/${characterKey}`));
    await activate(page.getByRole("tab", { name: "Abilities", exact: true }), info);
    await expect(
      reference.getByRole("button", { name: "Night sight", exact: true }),
    ).toBeVisible();
    await page.goBack();
    await expect(currentView).toHaveText(["Characters"]);
    await expect(currentPage).toHaveText(["Parties"]);
    await expect(
      page.getByRole("heading", { name: second.name, exact: true }),
    ).toBeVisible();
    await expect(
      page.locator(`a[href="/parties/${second.party_key}"]`),
    ).toHaveAttribute("aria-current", "page");
    await page.goto(`/encounters/${encounter.slug}`);
    await expect(currentView).toHaveText(["Encounters"]);
    await expect(characterPages).toHaveCount(0);
    await activate(
      page.getByRole("button", { name: "Add participants", exact: true }),
      info,
    );
    await activate(
      page.getByRole("menuitem", { name: "Existing character", exact: true }),
      info,
    );
    const picker = page.getByRole("dialog", { name: "Choose a character" });
    await expect(
      picker.getByRole("combobox", { name: "Filter by party" }),
    ).toBeVisible();
    await picker.getByRole("textbox", { name: "Find a character" }).fill("Aster");
    await activate(
      picker
        .getByRole("listitem")
        .filter({ hasText: "Aster" })
        .getByRole("button", { name: /^Add Aster/ }),
      info,
    );
    await expect(page.locator("[data-participant-key]")).toHaveCount(1);
    await expect(picker.getByRole("textbox", { name: "Find a character" })).toHaveValue(
      "Aster",
    );
    await expect(picker.getByText("Aster added to the encounter.")).toBeVisible();
    await activate(picker.getByRole("button", { name: "Done" }), info);
    if (info.project.use.hasTouch)
      await activate(
        page.locator("[data-participant-key]").filter({ hasText: "Aster" }),
        info,
      );
    await activate(page.getByRole("button", { name: "View PC reference" }), info);
    const drawer = page.getByRole("dialog", { name: "Aster", exact: true });
    await expect
      .poll(() =>
        drawer.evaluate((element) =>
          Math.abs(element.getBoundingClientRect().right - innerWidth),
        ),
      )
      .toBeLessThan(0.5);
    await activate(
      drawer.getByRole("button", { name: "Night sight", exact: true }),
      info,
    );
    await expect(
      drawer.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await expect(page).toHaveURL(new RegExp(`/encounters/${encounter.slug}`));
    await noPageOverflow(page);
  } finally {
    releaseSearch?.();
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
    if (characterKey) await deleteCharacters(page.request, [characterKey]);
    await deleteParties(page.request, party.party_key, second.party_key);
  }
});

test("an empty saved sheet imports into the same character and returns focus", async ({
  page,
}, info) => {
  const name = `First import ${info.project.name} ${Date.now()}`;
  const response = await page.request.post("/api/characters", {
    data: { name, level: 7, request_id: crypto.randomUUID() },
  });
  expect(response.ok()).toBe(true);
  const { character_key } = await response.json();
  try {
    await page.goto(`/characters/${character_key}`);
    const directories = page.getByRole("navigation", { name: "Character pages" });
    await expect(directories.getByRole("link")).toHaveText([
      "All characters",
      "Parties",
    ]);
    await expect(
      directories.getByRole("link", { name: "All characters" }),
    ).toHaveAttribute("aria-current", "page");
    const opener = page.getByRole("button", {
      name: "Import from Pathbuilder",
      exact: true,
    });
    const dialog = page.getByRole("dialog", { name: "New import from Pathbuilder" });
    await activate(opener, info);
    await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(opener).toBeFocused();

    await activate(opener, info);
    const input = dialog.getByLabel("Pathbuilder JSON");
    await input.fill("{");
    await activate(dialog.getByRole("button", { name: "Continue", exact: true }), info);
    await expect(
      dialog.getByText("Could not review character", { exact: true }),
    ).toBeVisible();
    await expect(input).toHaveValue("{");
    await input.fill(exportedCharacter(name));
    await activate(dialog.getByRole("button", { name: "Continue", exact: true }), info);
    await activate(
      dialog.getByRole("button", { name: "Update character", exact: true }),
      info,
    );
    await expect(dialog).toBeHidden();
    await expect(page).toHaveURL(new RegExp(`/characters/${character_key}/?$`));
    await expect(page.getByText("Max HP", { exact: true })).toBeVisible();
    await expect(opener).toHaveCount(0);
    await expect(
      page.getByRole("tabpanel", { name: "Stats" }).getByText(/^Imported\b/),
    ).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Character actions" })).toBeFocused();
    await page.setViewportSize({ width: 390, height: 740 });
    await page.goto("/characters");
    const row = page.getByRole("row", { name: new RegExp(name) });
    await expect(row.getByRole("link", { name, exact: true })).toHaveCSS(
      "text-align",
      "start",
    );
  } finally {
    await deleteCharacters(page.request, [character_key]);
  }
});

test("Characters imports and manual PCs can be used in encounters", async ({
  page,
}, info) => {
  const stamp = `${info.project.name} ${Date.now()}`;
  const manualName = `Manual ${stamp}`;
  const importedName = `Imported ${stamp}`;
  const encounter = await createEncounter(page.request, stamp);
  const created: string[] = [];
  try {
    await page.goto("/characters");
    await activate(page.getByRole("button", { name: "Add PC", exact: true }), info);
    await activate(page.getByRole("menuitem", { name: "From Pathbuilder" }), info);
    const importer = page.getByRole("dialog", { name: "Import from Pathbuilder" });
    await importer.getByLabel("Pathbuilder JSON").fill(exportedCharacter(importedName));
    await activate(
      importer.getByRole("button", { name: "Continue", exact: true }),
      info,
    );
    await activate(importer.getByRole("button", { name: "Add character" }), info);
    await expect(page).toHaveURL(/\/characters\/[^/]+$/);
    const importedKey = new URL(page.url()).pathname.split("/").pop()!;
    created.push(importedKey);

    await page.goto(`/encounters/${encounter.slug}`);
    const add = page.getByRole("button", { name: "Add participants" });
    const menu = page.getByRole("menu", { name: "Add participants" });
    const enterManually = menu.getByRole("menuitem", { name: "Enter manually" });
    if (info.project.use.hasTouch) {
      await activate(add, info);
    } else {
      await add.focus();
      await page.keyboard.press("Enter");
    }
    await expect(menu).toBeVisible();
    await expect(
      menu.getByRole("menuitem", { name: /Pathbuilder|^Add PC/ }),
    ).toHaveCount(0);
    if (info.project.use.hasTouch) {
      await activate(enterManually, info);
    } else {
      await page.keyboard.press("Tab");
      await page.keyboard.press("Home");
      await expect(
        menu.getByRole("menuitem", { name: /Add creature or hazard/ }),
      ).toBeFocused();
      for (const name of [/Add custom turn/, "Existing character", "Enter manually"]) {
        await page.keyboard.press("ArrowDown");
        await expect(menu.getByRole("menuitem", { name })).toBeFocused();
      }
      await page.keyboard.press("Enter");
    }
    const manual = page.getByRole("dialog", { name: "Add PC", exact: true });
    await manual.getByLabel("Name", { exact: true }).fill(manualName);
    await manual.getByLabel("Max HP", { exact: true }).fill("30");
    await manual.getByLabel("Initiative", { exact: true }).fill("17");
    await activate(manual.getByRole("button", { name: "Add to encounter" }), info);
    await expect(page.locator("[data-participant-key]")).toHaveCount(1);
    const first = await (
      await page.request.get(`/api/encounters/${encounter.encounter_key}`)
    ).json();
    const participant = first.participants[0];
    const saved = await (await page.request.get("/api/characters")).json();
    const character = saved.characters.find(
      (item: { name: string }) => item.name === manualName,
    );
    expect(character).toBeDefined();
    created.push(character.character_key);
    expect(character.imported).toBe(false);
    await expect(add).toBeFocused();
    await activate(add, info);
    await activate(menu.getByRole("menuitem", { name: "Existing character" }), info);
    const picker = page.getByRole("dialog", { name: "Choose a character" });
    await picker.getByRole("textbox", { name: "Find a character" }).fill(importedName);
    await activate(
      picker
        .getByRole("listitem")
        .filter({ hasText: importedName })
        .getByRole("button", {
          name: /^Add /,
        }),
      info,
    );
    await expect(page.locator("[data-participant-key]")).toHaveCount(2);
    await activate(picker.getByRole("button", { name: "Done", exact: true }), info);
    await expect(add).toBeFocused();
    await activate(
      page.locator("[data-participant-key]").filter({ hasText: importedName }),
      info,
    );
    await activate(page.getByRole("button", { name: "View PC reference" }), info);
    const drawer = page.getByRole("dialog", { name: importedName, exact: true });
    await expect(drawer).toBeVisible();
    await expect(drawer.getByRole("heading", { name: /^Features/ })).toBeVisible();
    await activate(drawer.getByRole("tab", { name: "Stats", exact: true }), info);
    await expect(drawer.getByText("Max HP", { exact: true })).toBeVisible();
    await expect(page.locator("[data-participant-key]")).toHaveCount(2);
    await activate(drawer.getByRole("button", { name: "Close", exact: true }), info);
    const live = await (
      await page.request.get(`/api/characters/${character.character_key}`)
    ).json();
    const removed = await page.request.delete(
      `/api/characters/${character.character_key}`,
      { data: { expected_revision: live.character_revision } },
    );
    expect(removed.ok()).toBe(true);
    const after = await (
      await page.request.get(`/api/encounters/${encounter.encounter_key}`)
    ).json();
    const retained = after.participants.find(
      (item: { participant_key: string }) =>
        item.participant_key === participant.participant_key,
    );
    expect(retained.display_name).toBe(participant.display_name);
    expect(retained.current_hp).toBe(participant.current_hp);
    expect(retained.initiative).toBe(participant.initiative);
    const reference = await (
      await page.request.get(
        `/api/encounters/${encounter.encounter_key}/participants/${participant.participant_key}/pc-reference`,
      )
    ).json();
    expect(reference.reference).toBeUndefined();
    await noPageOverflow(page);
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
    await deleteCharacters(page.request, created);
  }
});

test("party drag tracks the pointer and shows the reordered row on release @engine", async ({
  page,
}) => {
  const name = "Tamsin the Lantern Keeper of the Northern Watch";
  const names = [name, "Bram"];
  const keys: string[] = [];
  let party: PartyView | undefined;
  try {
    for (const character of names) {
      const response = await page.request.post("/api/characters", {
        data: { request_id: crypto.randomUUID(), name: character, level: 7 },
      });
      expect(response.ok()).toBe(true);
      keys.push((await response.json()).character_key);
    }
    const created = await page.request.post("/api/parties", {
      data: {
        name: `Party drag ${Date.now()}`,
        members: keys.map((character_key) => ({
          character_key,
          expected_character_revision: 0,
        })),
      },
    });
    expect(created.ok()).toBe(true);
    party = (await created.json()) as PartyView;
    await page.goto(`/parties/${party.party_key}`);

    const handle = page.getByRole("button", { name: `Reorder ${name}`, exact: true });
    const row = page.getByRole("row").filter({ has: handle });
    const target = page.getByRole("button", { name: "Reorder Bram", exact: true });
    const roster = page.getByRole("table");
    const order = roster.getByRole("link");
    await expect(order).toHaveText(names);
    const source = (await row.boundingBox())!;
    const from = (await handle.boundingBox())!;
    const to = (await target.boundingBox())!;
    const start = {
      x: Math.round(from.x + from.width / 2),
      y: Math.round(from.y + from.height / 2),
    };
    const end = { x: start.x + 12, y: Math.round(to.y + to.height / 2) };
    const preview = page.locator(".party-roster__drag-preview");

    await page.mouse.move(start.x, start.y);
    await page.mouse.down();
    await page.mouse.move(end.x, end.y, { steps: 12 });
    await expect
      .poll(async () => (await preview.boundingBox())?.y)
      .toBeCloseTo(source.y + end.y - start.y, 0);
    const dragged = (await preview.boundingBox())!;
    expect(dragged.x).toBeCloseTo(source.x + end.x - start.x, 0);
    expect(dragged.height).toBeCloseTo(source.height, 0);
    expect(Math.abs(dragged.y + dragged.height / 2 - end.y)).toBeLessThanOrEqual(1);

    await page.mouse.up();
    // Inspect the handoff in the first painted frames, before a drop animation could end.
    await page.evaluate(
      () =>
        new Promise<void>((resolve) => {
          requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
        }),
    );
    expect(await preview.count()).toBe(0);
    expect(await row.evaluate((element) => getComputedStyle(element).opacity)).toBe(
      "1",
    );
    await expect(order).toHaveText(["Bram", name]);
    await expect(roster).toHaveAttribute("aria-busy", "false");
    await page.reload();
    await expect(order).toHaveText(["Bram", name]);
  } finally {
    if (party) await deleteParties(page.request, party.party_key);
    await deleteCharacters(page.request, keys);
  }
});

test("party edit tasks preserve keyboard drafts and update shared characters explicitly", async ({
  page,
  browserName,
}, info) => {
  const name = `Party edits ${info.project.name} ${Date.now()}`;
  const first = await createParty(page.request, name);
  const second = await createParty(page.request, `${name} other`);
  const keys: string[] = [];
  try {
    for (const pc of ["Aster", "Bram"]) {
      const response = await page.request.post("/api/characters", {
        data: { request_id: crypto.randomUUID(), name: pc, level: 7 },
      });
      expect(response.ok()).toBe(true);
      keys.push((await response.json()).character_key);
    }
    const members = keys.map((character_key) => ({
      character_key,
      expected_character_revision: 0,
    }));
    expect(
      (
        await page.request.patch(`/api/parties/${first.party_key}`, {
          data: { name, expected_revision: first.revision, members, level_updates: [] },
        })
      ).ok(),
    ).toBe(true);
    expect(
      (
        await page.request.patch(`/api/parties/${second.party_key}`, {
          data: {
            name: second.name,
            expected_revision: second.revision,
            members: [members[0]],
            level_updates: [],
          },
        })
      ).ok(),
    ).toBe(true);
    await page.goto(`/parties/${first.party_key}`);
    const actions = page.getByRole("button", { name: "Party actions", exact: true });
    await activate(actions, info);
    await activate(
      page.getByRole("menuitem", { name: "Rename party", exact: true }),
      info,
    );
    const rename = page.getByRole("dialog", { name: "Rename party", exact: true });
    const partyName = rename.getByLabel("Party name", { exact: true });
    await expect(partyName).toBeFocused();
    await partyName.fill("A temporary rename");
    await partyName.fill(name);
    await page.keyboard.press("Escape");
    await expect(rename).toBeHidden();
    await expect(actions).toBeFocused();
    await activate(actions, info);
    await activate(
      page.getByRole("menuitem", { name: "Set character levels…", exact: true }),
      info,
    );
    const levels = page.getByRole("dialog", {
      name: "Set character levels",
      exact: true,
    });
    const target = levels.getByRole("spinbutton", { name: "New level", exact: true });
    await expect(target).toBeFocused();
    await target.fill("-");
    await target.press("Enter");
    await expect(target).toHaveValue("-");
    await page.keyboard.press("Escape");
    await expect(
      levels.getByRole("button", { name: "Keep editing", exact: true }),
    ).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(target).toBeFocused();
    await expect(target).toHaveValue("-");
    await target.fill("");
    await target.pressSequentially("1.");
    await target.press("Enter");
    await expect(target).toHaveValue("1.");
    await page.keyboard.press("Escape");
    await expect(
      levels.getByRole("button", { name: "Keep editing", exact: true }),
    ).toBeFocused();
    const discard = levels.getByRole("button", { name: "Discard", exact: true });
    if (browserName === "webkit") {
      await discard.focus();
    } else {
      await page.keyboard.press("Tab");
      await expect(discard).toBeFocused();
    }
    await page.keyboard.press("Enter");
    await expect(levels).toBeHidden();
    await expect(actions).toBeFocused();
    await activate(actions, info);
    await activate(
      page.getByRole("menuitem", { name: "Set character levels…", exact: true }),
      info,
    );
    await target.fill("7");
    await expect(
      levels.getByRole("button", { name: "Set character levels", exact: true }),
    ).toBeDisabled();
    await page.keyboard.press("Escape");
    await expect(levels).toBeHidden();
    await activate(actions, info);
    await activate(
      page.getByRole("menuitem", { name: "Set character levels…", exact: true }),
      info,
    );
    await target.fill("8");
    await activate(levels.getByRole("checkbox", { name: "Bram", exact: true }), info);
    await expect(levels.getByText(second.name, { exact: false })).toHaveCount(0);
    await noPageOverflow(page);
    await activate(
      levels.getByRole("button", { name: "Set 1 character to level 8", exact: true }),
      info,
    );
    await expect(levels).toBeHidden();
    const index = await (await page.request.get("/api/parties")).json();
    const saved = (index.parties as PartyView[]).find(
      (p) => p.party_key === first.party_key,
    )!;
    expect(saved.members.map((m) => [m.name, m.level])).toEqual([
      ["Aster", 8],
      ["Bram", 7],
    ]);
    const shared = (index.parties as PartyView[]).find(
      (p) => p.party_key === second.party_key,
    )!;
    expect(shared.members[0].level).toBe(8);
    await activate(actions, info);
    await activate(
      page.getByRole("menuitem", { name: "Rename party", exact: true }),
      info,
    );
    await partyName.fill(`${name} renamed`);
    await activate(rename.getByRole("button", { name: "Rename", exact: true }), info);
    await expect(rename).toBeHidden();
    await expect(
      page.getByRole("heading", { name: `${name} renamed`, exact: true }),
    ).toBeVisible();
  } finally {
    await deleteCharacters(page.request, keys);
    await deleteParties(page.request, first.party_key, second.party_key);
  }
});

test("guards inline character drafts when leaving a party", async ({ page }, info) => {
  const party = await createParty(page.request, `Draft review ${Date.now()}`);
  try {
    await page.goto("/parties");
    await activate(page.locator(`a[href="/parties/${party.party_key}"]`), info);
    const add = page.getByRole("button", { name: "Add PC", exact: true });
    async function startDraft() {
      await activate(add, info);
      await activate(page.getByRole("menuitem", { name: "Enter manually" }), info);
    }
    await startDraft();
    const editor = page.locator(".character-inline-editor");
    const level = editor.getByLabel("Level", { exact: true });
    await level.fill("invalid");
    await page.evaluate(() => history.back());
    const discard = page.getByRole("dialog", { name: "Discard new character?" });
    await expect(discard).toBeVisible();
    await activate(discard.getByRole("button", { name: "Keep editing" }), info);
    await expect(discard).toBeHidden();
    await expect(level).toHaveValue("invalid");
    await level.fill(" ");
    await activate(editor.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(editor).toBeHidden();
    await expect(discard).toBeHidden();

    await startDraft();
    await editor.getByLabel("Name", { exact: true }).fill("A draft character");
    await page.evaluate(() => history.back());
    await expect(discard).toBeVisible();
    await activate(discard.getByRole("button", { name: "Discard", exact: true }), info);
    await expect(page).toHaveURL(/\/parties\/?$/);
  } finally {
    await deleteParties(page.request, party.party_key);
  }
});

test("character sheet keeps formulas together and level choices across import, page and drawer", async ({
  page,
}, info) => {
  const name = `Sheet layout ${info.project.name} ${Date.now()}`;
  const source = JSON.stringify({
    success: true,
    build: {
      name,
      level: 7,
      feats: [
        ["Later feat", null, "Class Feat", 10, "FutureFeat", "customChoice", "Feat"],
        [
          "Assurance",
          "Arcana",
          "Skill Feat",
          2,
          "Class FeatHuman Feat 1",
          "childChoice",
          "Human Feat 1",
        ],
        ["Starting feat", null, "Ancestry Feat", 0],
        [
          "Assurance",
          "Athletics",
          "Skill Feat",
          2,
          "Heritage Featversatile heritage",
          "childChoice",
          "Heritage Feat",
        ],
      ],
      specials: ["Darkvision"],
      spellCasters: [],
      weapons: [{ name: "Dagger", attack: 0, qty: 0, die: "d4", damageType: "P" }],
      equipment: [
        ["Rope", 2, "Not Invested"],
        ["Container", 0, "0", "Custom state"],
      ],
      equipmentContainers: {
        "0": { containerName: "Backpack", backpack: false },
        empty: { containerName: "Backpack", bagOfHolding: false },
      },
      formula: [
        { type: "wandering chef", known: ["Iron Wine"] },
        { type: "other", known: ["Iron Wine", "Mender's Soup"] },
      ],
    },
  });
  let characterKey: string | undefined;
  const encounter = await createEncounter(page.request, name);
  try {
    await page.goto("/characters");
    await activate(page.getByRole("button", { name: "Add PC", exact: true }), info);
    await activate(page.getByRole("menuitem", { name: "From Pathbuilder" }), info);
    const importer = page.getByRole("dialog", { name: "Import from Pathbuilder" });
    await importer.getByLabel("Pathbuilder JSON").fill(source);
    await activate(
      importer.getByRole("button", { name: "Continue", exact: true }),
      info,
    );
    await activate(importer.getByRole("tab", { name: "Inventory", exact: true }), info);
    await expect(
      importer.getByRole("heading", { name: "Backpack (2)", exact: true }),
    ).toBeVisible();
    await expect(importer.getByText("Container", { exact: true })).toHaveCount(1);
    await expect(
      importer.getByRole("heading", {
        name: "Known formulas · Wandering chef",
        exact: true,
      }),
    ).toBeVisible();
    await activate(
      importer.getByRole("tab", { name: "Progression", exact: true }),
      info,
    );
    await expect(importer.getByRole("heading", { name: /^Level / })).toHaveText([
      "Level 0",
      "Level 2",
      "Level 10",
    ]);
    const progression = importer.getByRole("tabpanel", { name: "Progression" });
    const arcana = progression
      .getByRole("listitem")
      .filter({ has: page.getByRole("heading", { name: "Assurance (Arcana)" }) })
      .last();
    await expect(arcana.getByText("Skill Feat", { exact: true })).toBeVisible();
    await expect(progression.getByText("Type", { exact: true })).toHaveCount(0);
    await expect(
      progression.getByRole("heading", { name: "Assurance (Athletics)" }),
    ).toBeVisible();
    const switcher = progression.getByRole("switch", { name: "Show feat sources" });
    await expect(switcher).not.toBeChecked();
    await expect(progression.getByText("Slot", { exact: true })).toBeHidden();
    if (info.project.use.hasTouch) await activate(switcher, info);
    else await switcher.press("Space");
    await expect(switcher).toBeChecked();
    await expect(
      arcana.getByText("Class Feat · Human Feat 1", { exact: true }),
    ).toBeVisible();
    await expect(
      progression.getByText("Heritage Featversatile heritage", { exact: true }),
    ).toBeVisible();
    await expect(progression.getByText("FutureFeat", { exact: true })).toBeVisible();
    await noPageOverflow(page);
    await activate(switcher, info);
    await expect(progression.getByText("Slot", { exact: true })).toBeHidden();
    await activate(
      importer.getByRole("button", { name: "Add character", exact: true }),
      info,
    );
    await expect(page).toHaveURL(/\/characters\/character_/);
    characterKey = new URL(page.url()).pathname.split("/").at(-1)!;
    await activate(page.getByRole("tab", { name: "Inventory", exact: true }), info);
    const inventory = page.getByRole("tabpanel", { name: "Inventory" });
    const gear = inventory
      .getByRole("heading", { name: "Backpack (2)", exact: true })
      .locator("..");
    const chef = inventory
      .getByRole("heading", { name: "Known formulas · Wandering chef", exact: true })
      .locator("..");
    const other = inventory
      .getByRole("heading", { name: "Known formulas · Other", exact: true })
      .locator("..");
    const gearBox = (await gear.boundingBox())!;
    const chefBox = (await chef.boundingBox())!;
    const otherBox = (await other.boundingBox())!;
    expect(chefBox.y).toBeGreaterThanOrEqual(gearBox.y + gearBox.height);
    expect(otherBox.y).toBeGreaterThanOrEqual(chefBox.y + chefBox.height);
    expect(Math.abs(chefBox.x - otherBox.x)).toBeLessThan(1);
    await expect(inventory.getByText("Iron Wine", { exact: true })).toHaveCount(2);
    await expect(inventory.getByText("Custom state", { exact: true })).toBeVisible();
    const weapon = inventory
      .getByRole("heading", { name: "Weapons", exact: true })
      .locator("..");
    const width = await weapon.evaluate((node) => ({
      group: node.clientWidth,
      available: node.parentElement!.clientWidth,
    }));
    expect(width.group).toBe(width.available);
    await noPageOverflow(page);
    await activate(page.getByRole("tab", { name: "Progression", exact: true }), info);
    await expect(
      page
        .getByRole("tabpanel", { name: "Progression" })
        .getByRole("heading", { name: /^Level / }),
    ).toHaveText(["Level 0", "Level 2", "Level 10"]);
    const character = (await (
      await page.request.get(`/api/characters/${characterKey}`)
    ).json()) as PcReferenceView;
    const currentEncounter = await (
      await page.request.get(`/api/encounters/${encounter.encounter_key}`)
    ).json();
    const added = await page.request.post(
      `/api/encounters/${encounter.encounter_key}/participants/character`,
      {
        data: {
          request_id: crypto.randomUUID(),
          expected_encounter_revision: currentEncounter.revision,
          source: {
            kind: "existing",
            character_key: characterKey,
            expected_character_revision: character.character_revision,
          },
        },
      },
    );
    expect(added.ok()).toBe(true);
    await page.goto(`/encounters/${encounter.slug}`);
    if (info.project.use.hasTouch)
      await activate(
        page.locator("[data-participant-key]").filter({ hasText: name }),
        info,
      );
    await activate(page.getByRole("button", { name: "View PC reference" }), info);
    const drawer = page.getByRole("dialog", { name, exact: true });
    await activate(drawer.getByRole("tab", { name: "Inventory", exact: true }), info);
    await expect(drawer.getByText("Custom state", { exact: true })).toBeVisible();
    await expect(drawer.getByText("Iron Wine", { exact: true })).toHaveCount(2);
    await activate(drawer.getByRole("tab", { name: "Progression", exact: true }), info);
    await expect(drawer.getByRole("heading", { name: /^Level / })).toHaveText([
      "Level 0",
      "Level 2",
      "Level 10",
    ]);
    await expect(
      drawer.getByRole("heading", { name: "Assurance (Arcana)" }),
    ).toBeVisible();
    await expect(
      drawer.getByRole("switch", { name: "Show feat sources" }),
    ).not.toBeChecked();
    await noPageOverflow(page);
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
    if (characterKey) await deleteCharacters(page.request, [characterKey]);
  }
});

test("Magic keeps the spell list readable and reveals preparations at their exported rank", async ({
  page,
}, info) => {
  const source = {
    success: true,
    build: {
      name: "Caster with a borrowed spellbook",
      level: 7,
      money: { gp: 0, sp: null },
      focusPoints: 1,
      feats: [],
      specials: [],
      spellCasters: [
        {
          name: "Wizard",
          perDay: [0, 2, 1, 3, 1],
          spells: [{ spellLevel: 1, list: ["Force Barrage"] }],
          prepared: [
            { spellLevel: 3, list: ["Force Barrage", "Force Barrage", "Haste"] },
          ],
        },
        {
          name: "Innate spells",
          spells: [],
          prepared: [{ spellLevel: 2, list: ["See the Unseen"] }],
        },
        {
          name: "Bard",
          spells: [{ spellLevel: 1, list: ["Fear"] }],
        },
      ],
    },
  };
  await page.goto("/characters");
  await activate(page.getByRole("button", { name: "Add PC", exact: true }), info);
  await activate(page.getByRole("menuitem", { name: "From Pathbuilder" }), info);
  const importer = page.getByRole("dialog", { name: "Import from Pathbuilder" });
  await importer.getByLabel("Pathbuilder JSON").fill(JSON.stringify(source));
  await activate(importer.getByRole("button", { name: "Continue", exact: true }), info);
  await activate(importer.getByRole("tab", { name: "Inventory", exact: true }), info);
  await expect(importer.getByRole("tabpanel", { name: "Inventory" })).toHaveText(
    "Coins 0 gp",
  );
  await activate(importer.getByRole("tab", { name: "Magic", exact: true }), info);
  const magic = importer.getByRole("tabpanel", { name: "Magic" });
  await expect(magic.getByText("Focus pool 1 point", { exact: true })).toBeVisible();
  const snapshot = magic.getByRole("button", { name: "Prepared at export" });
  await expect(snapshot).toHaveAttribute("aria-expanded", "false");
  await expect(magic.getByText("Force Barrage", { exact: true })).toBeVisible();
  await expect(magic.getByText("See the Unseen", { exact: true })).toBeVisible();
  await expect(magic.getByText("Fear", { exact: true })).toBeVisible();
  await expect(magic.getByText(/\bHaste$/)).toBeHidden();
  await expect(magic.getByText(/Rank 4.*1 slot/)).toBeVisible();
  if (info.project.use.hasTouch) await activate(snapshot, info);
  else await snapshot.press("Space");
  await expect(snapshot).toHaveAttribute("aria-expanded", "true");
  const preparedRow = magic.locator(".actor-casting__row").filter({ hasText: "Haste" });
  await expect(preparedRow).toContainText("Rank 3");
  await expect(preparedRow).toContainText(/Force Barrage\s*×2/);
  await expect(preparedRow).toContainText("Haste");
  await noPageOverflow(page);
  await activate(snapshot, info);
  await expect(magic.getByText(/\bHaste$/)).toBeHidden();
  await expect(magic.getByText("Force Barrage", { exact: true })).toBeVisible();
});
