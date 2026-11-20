import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";
import type { PartyView, PcReferenceView } from "../../src/generated/atlas";

function exportedCharacter(name: string, level = 7) {
  return JSON.stringify({
    success: true,
    build: {
      name,
      level,
      feats: [],
      specials: ["Darkvision", "Night sight"],
      spellCasters: [],
    },
  });
}

test("Pathbuilder character rules stay shared between parties and encounters", async ({
  page,
}, info) => {
  const name = `Pathbuilder ${info.project.name} ${Date.now()}`;
  const party = (await (
    await page.request.post("/api/parties", { data: { name, members: [] } })
  ).json()) as PartyView;
  const second = (await (
    await page.request.post("/api/parties", {
      data: { name: `${name} current`, members: [] },
    })
  ).json()) as PartyView;
  const { encounter } = await (
    await page.request.post("/api/encounters", { data: { name } })
  ).json();
  let characterKey: string | undefined;
  let releaseSearch: (() => void) | undefined;
  try {
    await page.goto("/encounters");
    await activate(page.getByRole("link", { name: "Parties", exact: true }), info);
    const directory = page.getByRole("navigation", {
      name: "Parties",
      includeHidden: true,
    });
    const mountedDirectory = await directory.elementHandle();
    const chooseParty = () =>
      activate(page.locator(`a[href="/parties/${party.party_key}"]`), info);
    await chooseParty();
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
    await expect(page).toHaveURL(/\/characters\/[^/]+\/?$/);
    characterKey = new URL(page.url()).pathname.split("/").filter(Boolean).at(-1)!;
    const imported = (await (
      await page.request.get(`/api/characters/${characterKey}`)
    ).json()) as PcReferenceView;
    expect(
      imported.abilities.find((ability) => ability.label === "Darkvision")?.manual,
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
    await expect(
      reference.getByRole("button", { name: "Night sight", exact: true }),
    ).toBeVisible();
    await page.goBack();
    await expect(
      page.getByRole("heading", { name: second.name, exact: true }),
    ).toBeVisible();
    await expect(
      page.locator(`a[href="/parties/${second.party_key}"]`),
    ).toHaveAttribute("aria-current", "page");
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Add participants", exact: true }),
      info,
    );
    await activate(page.getByRole("menuitem", { name: "Add PC…" }), info);
    await activate(
      page.getByRole("button", { name: "Existing character", exact: true }),
      info,
    );
    const picker = page.getByRole("dialog", { name: "Choose a character" });
    await picker.getByRole("textbox", { name: "Find a character" }).fill("Aster");
    await activate(
      picker
        .getByRole("listitem")
        .filter({ hasText: "Aster" })
        .getByRole("button", { name: "Add", exact: true }),
      info,
    );
    await expect(page.locator("[data-participant-key]")).toHaveCount(1);
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
    if (characterKey) {
      const result = await page.request.get(`/api/characters/${characterKey}`);
      if (result.ok())
        await page.request.delete(`/api/characters/${characterKey}`, {
          data: { expected_revision: (await result.json()).character_revision },
        });
    }
    const index = await (await page.request.get("/api/parties")).json();
    for (const key of [party.party_key, second.party_key]) {
      const current = (index.parties as PartyView[]).find((p) => p.party_key === key);
      if (current)
        await page.request.delete(`/api/parties/${key}`, {
          data: { expected_revision: current.revision },
        });
    }
  }
});

test("encounter imports and manual PCs create reusable characters", async ({
  page,
}, info) => {
  const stamp = `${info.project.name} ${Date.now()}`;
  const manualName = `Manual ${stamp}`;
  const importedName = `Imported ${stamp}`;
  const { encounter } = await (
    await page.request.post("/api/encounters", { data: { name: stamp } })
  ).json();
  const created: string[] = [];
  try {
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(page.getByRole("button", { name: "Add participants" }), info);
    await activate(page.getByRole("menuitem", { name: "Add PC…" }), info);
    await activate(page.getByRole("button", { name: "Enter manually" }), info);
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
    await activate(page.getByRole("button", { name: "Add participants" }), info);
    await activate(page.getByRole("menuitem", { name: "Add PC…" }), info);
    await activate(page.getByRole("button", { name: "From Pathbuilder" }), info);
    const importer = page.getByRole("dialog", { name: "Import from Pathbuilder" });
    await importer.getByLabel("Pathbuilder JSON").fill(exportedCharacter(importedName));
    await activate(
      importer.getByRole("button", { name: "Continue", exact: true }),
      info,
    );
    await activate(importer.getByRole("button", { name: "Add to encounter" }), info);
    const drawer = page.getByRole("dialog", { name: importedName, exact: true });
    await expect(drawer).toBeVisible();
    await expect(drawer.getByRole("heading", { name: /^Features/ })).toBeVisible();
    await expect(page.locator("[data-participant-key]")).toHaveCount(2);
    const index = await (await page.request.get("/api/characters")).json();
    created.push(
      index.characters.find((item: { name: string }) => item.name === importedName)
        .character_key,
    );
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
    for (const key of created) {
      const result = await page.request.get(`/api/characters/${key}`);
      if (result.ok())
        await page.request.delete(`/api/characters/${key}`, {
          data: { expected_revision: (await result.json()).character_revision },
        });
    }
  }
});

test("party edit tasks preserve keyboard drafts and update shared characters explicitly", async ({
  page,
  browserName,
}, info) => {
  const name = `Party edits ${info.project.name} ${Date.now()}`;
  const first = (await (
    await page.request.post("/api/parties", { data: { name, members: [] } })
  ).json()) as PartyView;
  const second = (await (
    await page.request.post("/api/parties", {
      data: { name: `${name} other`, members: [] },
    })
  ).json()) as PartyView;
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
      page.getByRole("button", { name: "Rename party", exact: true }),
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
      page.getByRole("button", { name: "Set character levels…", exact: true }),
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
      page.getByRole("button", { name: "Set character levels…", exact: true }),
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
      page.getByRole("button", { name: "Set character levels…", exact: true }),
      info,
    );
    await target.fill("8");
    await activate(levels.getByRole("checkbox", { name: "Bram", exact: true }), info);
    await expect(
      levels.getByText(`Also updates ${second.name}`, { exact: true }),
    ).toBeVisible();
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
      page.getByRole("button", { name: "Rename party", exact: true }),
      info,
    );
    await partyName.fill(`${name} renamed`);
    await activate(rename.getByRole("button", { name: "Rename", exact: true }), info);
    await expect(rename).toBeHidden();
    await expect(
      page.getByRole("heading", { name: `${name} renamed`, exact: true }),
    ).toBeVisible();
  } finally {
    for (const key of keys) {
      const response = await page.request.get(`/api/characters/${key}`);
      if (response.ok())
        await page.request.delete(`/api/characters/${key}`, {
          data: { expected_revision: (await response.json()).character_revision },
        });
    }
    const index = await (await page.request.get("/api/parties")).json();
    for (const key of [first.party_key, second.party_key]) {
      const current = (index.parties as PartyView[]).find((p) => p.party_key === key);
      if (current)
        await page.request.delete(`/api/parties/${key}`, {
          data: { expected_revision: current.revision },
        });
    }
  }
});
