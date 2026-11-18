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
  try {
    await page.goto("/encounters");
    await activate(page.getByRole("link", { name: "Parties", exact: true }), info);
    await activate(page.locator(`a[href="/parties/${party.party_key}"]`), info);
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
    await activate(importer.getByRole("button", { name: "Add to party" }), info);
    await expect(page).toHaveURL(/\/characters\/[^/]+\/?$/);
    characterKey = new URL(page.url()).pathname.split("/").filter(Boolean).at(-1)!;
    const reference = page.locator(".character-reference");
    const darkvision = reference.getByRole("button", {
      name: "Darkvision",
      exact: true,
    });
    await expect(reference.getByRole("heading", { name: /^Features/ })).toBeVisible();
    await activate(darkvision, info);
    await expect(
      reference.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await activate(reference.getByRole("button", { name: "Back", exact: true }), info);
    if (!info.project.use.hasTouch) await expect(darkvision).toBeFocused();
    await activate(
      reference.getByRole("button", { name: "Search for Night sight" }),
      info,
    );
    const search = reference.getByRole("textbox", { name: "Search Atlas rules" });
    await search.fill("Darkvision");
    await activate(
      reference.getByRole("button", { name: "Search", exact: true }),
      info,
    );
    await activate(darkvision, info);
    await expect(
      reference.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await activate(reference.getByRole("button", { name: "Back", exact: true }), info);
    await expect(search).toHaveValue("Darkvision");
    await activate(darkvision, info);
    await activate(reference.getByRole("button", { name: "Link this rule" }), info);
    await expect(
      reference.getByRole("button", { name: "Night sight", exact: true }),
    ).toBeVisible();
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
