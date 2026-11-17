import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";

test("Pathbuilder imports and links PC rules in place", async ({ page }, info) => {
  const name = `Pathbuilder ${info.project.name} ${Date.now()}`;
  const partyResponse = await page.request.post("/api/parties", {
    data: { name, members: [] },
  });
  expect(partyResponse.ok()).toBeTruthy();
  const party = await partyResponse.json();
  const encounterResponse = await page.request.post("/api/encounters", {
    data: { name },
  });
  expect(encounterResponse.ok()).toBeTruthy();
  const { encounter } = await encounterResponse.json();
  const partyUrl = `/api/parties/${party.party_key}`;
  const encounterUrl = `/api/encounters/${encounter.encounter_key}`;
  try {
    await page.goto("/encounters");
    await activate(page.getByRole("button", { name: "Parties", exact: true }), info);
    const manager = page.getByRole("dialog", { name: "Parties", exact: true });
    const partyRow = manager.getByRole("row").filter({
      has: page.getByRole("button", { name, exact: true }),
    });
    await activate(
      partyRow.getByRole("button", { name: "Import from Pathbuilder" }),
      info,
    );
    const importer = page.getByRole("dialog", { name: "Import from Pathbuilder" });
    await importer.getByLabel("Pathbuilder JSON").fill(
      JSON.stringify({
        success: true,
        build: {
          name: "Aster",
          level: 7,
          feats: [],
          specials: ["Darkvision", "Night sight"],
          spellCasters: [],
        },
      }),
    );
    await activate(importer.getByRole("button", { name: "Preview import" }), info);
    await expect(
      importer.getByRole("heading", { name: /Aster.*Level 7/ }),
    ).toBeVisible();
    await activate(importer.getByRole("button", { name: "Add to party" }), info);

    const character = partyRow.getByRole("button", { name: "Aster", exact: true });
    await activate(character, info);
    const drawer = page.getByRole("dialog", { name: "Aster", exact: true });
    const darkvision = drawer.getByRole("button", { name: "Darkvision", exact: true });
    await activate(darkvision, info);
    await expect(
      drawer.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await activate(drawer.getByRole("button", { name: "Back", exact: true }), info);
    if (!info.project.use.hasTouch) await expect(darkvision).toBeFocused();

    const searchOpener = drawer.getByRole("button", { name: "Search for Night sight" });
    await activate(searchOpener, info);
    const search = drawer.getByRole("textbox", { name: "Search Atlas rules" });
    await expect(search).toHaveValue("Night sight");
    await search.fill("Darkvision");
    const revisedQuery = page.waitForRequest(
      (request) =>
        new URL(request.url()).pathname === "/api/search/page" &&
        request.postDataJSON()?.criteria?.query === "Darkvision",
    );
    await activate(drawer.getByRole("button", { name: "Search", exact: true }), info);
    await revisedQuery;
    await activate(darkvision, info);
    await expect(
      drawer.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await activate(drawer.getByRole("button", { name: "Back", exact: true }), info);
    await expect(search).toHaveValue("Darkvision");
    if (!info.project.use.hasTouch) await expect(darkvision).toBeFocused();
    await activate(darkvision, info);
    await activate(drawer.getByRole("button", { name: "Link this rule" }), info);
    await expect(
      drawer.getByRole("button", { name: "Night sight", exact: true }),
    ).toBeVisible();
    await noPageOverflow(page);
    await activate(drawer.getByRole("button", { name: "Close", exact: true }), info);
    await activate(manager.getByRole("button", { name: "Done", exact: true }), info);

    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Add participants", exact: true }),
      info,
    );
    await activate(
      page.getByRole("menuitem", { name: new RegExp(`${name}.*Aster`) }),
      info,
    );
    await expect(page.locator("[data-participant-key]")).toHaveCount(1);
    const reference = page.getByRole("button", { name: "View PC reference" });
    if (info.project.use.hasTouch) {
      await activate(
        page.locator("[data-participant-key]").filter({ hasText: "Aster" }),
        info,
      );
    }
    await activate(reference, info);
    await activate(
      drawer.getByRole("button", { name: "Night sight", exact: true }),
      info,
    );
    await expect(
      drawer.getByText("Fixture darkvision reference content."),
    ).toBeVisible();
    await expect(page).toHaveURL(new RegExp(`/encounters/${encounter.slug}`));
  } finally {
    await page.request.delete(encounterUrl);
    const parties = await (await page.request.get("/api/parties")).json();
    const current = parties.parties.find(
      (item: { party_key: string }) => item.party_key === party.party_key,
    );
    if (current)
      await page.request.delete(partyUrl, {
        data: { expected_revision: current.revision },
      });
  }
});
