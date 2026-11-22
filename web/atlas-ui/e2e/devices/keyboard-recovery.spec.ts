import { expect, test } from "@playwright/test";

test("Escape closes the Sort mode menu before closing Sort and restoring focus", async ({
  page,
}) => {
  await page.goto("/search");
  const trigger = page.getByRole("button", { name: /^Sort:/ });
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "Sort results" });
  const mode = dialog.getByRole("combobox", { name: "Sort mode" });
  await mode.click();
  await expect(mode).toHaveAttribute("aria-expanded", "true");
  await mode.press("Escape");
  await expect(mode).toHaveAttribute("aria-expanded", "false");
  await expect(dialog).toBeVisible();
  await mode.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
});

test("entity action menus focus the first command and return focus on Escape", async ({
  page,
}) => {
  const name = `Menu focus ${Date.now()}`;
  const slug = name.toLowerCase().replaceAll(" ", "-");
  const list = await page.request.post("/api/lists", {
    data: { slug, name, tags: [] },
  });
  expect(list.ok()).toBe(true);
  const { list: savedList } = await list.json();
  const party = await page.request.post("/api/parties", {
    data: { name, members: [] },
  });
  expect(party.ok()).toBe(true);
  const savedParty = await party.json();
  const character = await page.request.post("/api/characters", {
    data: { name, request_id: crypto.randomUUID() },
  });
  expect(character.ok()).toBe(true);
  const savedCharacter = await character.json();
  const created = await page.request.post("/api/encounters", { data: { name } });
  expect(created.ok()).toBe(true);
  const { encounter } = await created.json();
  try {
    for (const [route, label, first] of [
      [`/lists/${slug}`, "List actions", "Edit list"],
      [`/parties/${savedParty.party_key}`, "Party actions", "Rename party"],
      [
        `/characters/${savedCharacter.character_key}`,
        "Character actions",
        "Edit name and level",
      ],
      [`/encounters/${encounter.slug}`, "Encounter actions", "Edit encounter"],
    ]) {
      await page.goto(route);
      const trigger = page.getByRole("button", { name: label, exact: true });
      await trigger.press("Enter");
      const menu = page.getByRole("menu", { name: label, exact: true });
      await expect(
        menu.getByRole("menuitem", { name: first, exact: true }),
      ).toBeFocused();
      await page.keyboard.press("ArrowDown");
      await expect(
        menu.getByRole("menuitem", { name: first, exact: true }),
      ).not.toBeFocused();
      await page.keyboard.press("Escape");
      await expect(menu).toBeHidden();
      await expect(trigger).toBeFocused();
      if (label === "Character actions") {
        await trigger.press("Space");
        await expect(
          menu.getByRole("menuitem", { name: first, exact: true }),
        ).toBeFocused();
        await page.keyboard.press("Enter");
        const dialog = page.getByRole("dialog", { name: `Edit ${name}`, exact: true });
        await expect(dialog).toBeVisible();
        await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
        await expect(dialog).toBeHidden();
        await expect(trigger).toBeFocused();
      }
    }
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
    await page.request.delete(`/api/lists/${savedList.list_key}`);
    const characterPath = `/api/characters/${savedCharacter.character_key}`;
    const character = await (await page.request.get(characterPath)).json();
    await page.request.delete(characterPath, {
      data: { expected_revision: character.character_revision },
    });
    await page.request.delete(`/api/parties/${savedParty.party_key}`, {
      data: { expected_revision: savedParty.revision },
    });
  }
});

test("failed encounter removal keeps keyboard focus on the retry button", async ({
  page,
}) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Removal retry ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const detailPath = `/api/encounters/${encounter.encounter_key}`;
  try {
    const added = await page.request.post(`${detailPath}/participants/manual`, {
      data: { display_name: "Retry Hero", max_hp: 30, current_hp: 30 },
    });
    expect(added.ok()).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    let removals = 0;
    await page.route(
      (url) => /\/api\/encounters\/[^/]+\/participants\/[^/]+$/.test(url.pathname),
      (route) => {
        if (route.request().method() !== "DELETE") return route.continue();
        removals += 1;
        return removals === 1 ? route.abort("failed") : route.continue();
      },
    );
    const remove = page.getByRole("button", { name: "Remove Retry Hero" });
    await remove.press("Enter");
    await expect(page.getByText("Try removing again.")).toBeVisible();
    await expect(remove).toBeEnabled();
    await expect(remove).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(remove).toHaveCount(0);
    expect(removals).toBe(2);
  } finally {
    await page.request.delete(detailPath);
  }
});

test("compact roster removal uses the row menu and returns focus after retry", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 640 });
  const created = await page.request.post("/api/encounters", {
    data: { name: `Compact removal ${Date.now()}` },
  });
  expect(created.ok()).toBe(true);
  const { encounter } = await created.json();
  const detailPath = `/api/encounters/${encounter.encounter_key}`;
  try {
    for (const display_name of [
      "Long named champion of the northern pass",
      "Next participant",
    ]) {
      const added = await page.request.post(`${detailPath}/participants/manual`, {
        data: { display_name, max_hp: 30, current_hp: 30 },
      });
      expect(added.ok()).toBe(true);
    }
    await page.goto(`/encounters/${encounter.slug}`);
    const actions = page.getByRole("button", {
      name: "Actions for Long named champion of the northern pass",
    });
    await expect(page.getByRole("button", { name: /^Remove Long named/ })).toHaveCount(
      0,
    );
    let removals = 0;
    await page.route(
      (url) => /\/api\/encounters\/[^/]+\/participants\/[^/]+$/.test(url.pathname),
      (route) => {
        if (route.request().method() !== "DELETE") return route.continue();
        removals += 1;
        return removals === 1 ? route.abort("failed") : route.continue();
      },
    );
    for (let attempt = 0; attempt < 2; attempt++) {
      await actions.press("Enter");
      await page
        .getByRole("menuitem", { name: "Remove participant", exact: true })
        .press("Enter");
      if (!attempt) {
        await expect(page.getByText("Try removing again.")).toBeVisible();
        await expect(actions).toBeFocused();
      }
    }
    await expect(actions).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Next participant", exact: true }),
    ).toBeFocused();
    expect(removals).toBe(2);
  } finally {
    await page.request.delete(detailPath);
  }
});

test("Escape cancels an edited seed and returns focus to Sort", async ({ page }) => {
  await page.goto("/search?sort=random&seed=42");
  const trigger = page.getByRole("button", { name: "Sort: Random" });
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "Sort results" });
  const seed = dialog.getByRole("textbox", { name: "Random seed" });
  await seed.fill("43");
  await seed.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
  expect(new URL(page.url()).searchParams.get("seed")).toBe("42");
});

test("the Actions picker shows the selected row when reopened after a search", async ({
  page,
}) => {
  const action = "bestiary-ability-glossary-srd:grab";
  await page.route("**/api/filters/values", async (route) => {
    const request = route.request().postDataJSON();
    if (request.kind !== "contextual" || request.field_id !== "actions" || request.text)
      return route.continue();
    // The compact browser fixture has no resolved Actions options.
    const response = await route.fetch();
    const result = await response.json();
    result.outcome.value.options = [
      {
        value: action,
        label: "Grab",
        distinct_roots: 1,
        selected: false,
        help: {
          definition_key: null,
          destination: {
            kind: "record",
            navigation: {
              record_key: action,
              embedded_path: [],
              field: null,
              passage: null,
              source_fingerprint: null,
            },
          },
        },
      },
    ];
    result.outcome.value.total_values = 1;
    await route.fulfill({ response, json: result });
  });
  await page.goto("/search");
  await page.getByRole("button", { name: "Advanced filters", exact: true }).click();
  const builder = page.getByRole("dialog", { name: "Advanced filters" });
  const condition = builder.getByRole("combobox", { name: "Add condition" });
  await condition.click();
  await condition.fill("Actions");
  await condition.press("Enter");
  await builder.getByRole("combobox", { name: "Condition for Actions" }).click();
  await page
    .getByText("includes any", { exact: true })
    .filter({ visible: true })
    .click();
  const trigger = builder.getByRole("button", { name: "Edit Actions values" });
  await trigger.click();
  const picker = page.getByRole("dialog", { name: "Actions choices" });
  await picker
    .getByRole("button", { name: "Actions: Grab, activate to select" })
    .click();
  const search = picker.getByRole("textbox", { name: "Search actions choices" });
  await search.fill("nonexistentactiontest");
  await expect(picker.getByText("No matching values", { exact: true })).toBeVisible();
  await expect(picker.getByRole("button", { name: /^Actions: Grab/ })).toHaveCount(0);
  await expect(
    builder.getByRole("button", { name: "Remove selected Grab from Actions" }),
  ).toBeVisible();
  await search.press("Escape");
  await expect(picker).toBeHidden();
  await trigger.click();
  await expect(search).toBeFocused();
  await expect(
    picker.getByRole("button", { name: "Actions: Grab, selected; activate to remove" }),
  ).toBeVisible();
  await search.press("Escape");
  await expect(picker).toBeHidden();
  const remove = builder.getByRole("button", {
    name: "Remove selected Grab from Actions",
  });
  await remove.press("Enter");
  await expect(remove).toHaveCount(0);
  await expect(trigger).toBeFocused();
});
