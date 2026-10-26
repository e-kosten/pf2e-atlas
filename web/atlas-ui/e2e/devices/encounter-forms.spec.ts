import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";

test("inline HP formulas stay bounded without displacing facts or actions", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `HP formulas ${info.project.name} ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name: "Formula Hero",
            current_hp: 300,
            max_hp: 500,
          },
        })
      ).ok(),
    ).toBeTruthy();
    const state = await (await page.request.get(path)).json();
    expect(
      (
        await page.request.patch(
          `${path}/participants/${state.participants[0].participant_key}/life-settings`,
          {
            data: {
              request_id: crypto.randomUUID(),
              expected_encounter_revision: state.revision,
              track_dying: false,
            },
          },
        )
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Formula Hero", exact: true }),
      info,
    );
    const current = page.getByRole("button", { name: "Edit Current HP", exact: true });
    await activate(current, info);
    const input = page.getByRole("textbox", { name: "Current HP", exact: true });
    const shortWidth = (await input.boundingBox())!.width;
    const panel = page.locator(".encounter-hp-panel");
    const actions = panel.locator(".encounter-hp-panel__actions");
    const actionPosition = (await actions.boundingBox())!;
    const expectBoundedRow = async (field: typeof input) => {
      const panelBounds = (await panel.boundingBox())!;
      const fieldBounds = (await field.boundingBox())!;
      const groupBounds = (await actions.boundingBox())!;
      expect(fieldBounds.x).toBeGreaterThanOrEqual(panelBounds.x);
      expect(fieldBounds.x + fieldBounds.width).toBeLessThanOrEqual(
        panelBounds.x + panelBounds.width + 1,
      );
      expect(Math.abs(groupBounds.x - actionPosition.x)).toBeLessThan(2);
      expect(Math.abs(groupBounds.y - actionPosition.y)).toBeLessThan(2);
      const buttons = actions.getByRole("button");
      await expect(buttons).toHaveCount(3);
      for (const button of await buttons.all()) {
        const bounds = (await button.boundingBox())!;
        expect(bounds.x).toBeGreaterThanOrEqual(panelBounds.x);
        expect(bounds.x + bounds.width).toBeLessThanOrEqual(
          panelBounds.x + panelBounds.width + 1,
        );
        expect(Math.abs(bounds.y - groupBounds.y)).toBeLessThan(2);
      }
      const maximum = (await panel.locator(".encounter-hp-maximum").boundingBox())!;
      const values = panel.locator(".encounter-hp-panel__values");
      const valuesBounds = (await values.boundingBox())!;
      for (const fact of await values.locator(":scope > *").all()) {
        const bounds = (await fact.boundingBox())!;
        expect(bounds.x).toBeGreaterThanOrEqual(valuesBounds.x - 1);
        expect(bounds.x + bounds.width).toBeLessThanOrEqual(
          valuesBounds.x + valuesBounds.width + 1,
        );
      }
      if (!(await values.getByRole("textbox").count())) {
        expect(fieldBounds.x).toBeGreaterThanOrEqual(
          valuesBounds.x + valuesBounds.width,
        );
      }
      expect(
        Math.abs(
          maximum.y + maximum.height / 2 - fieldBounds.y - fieldBounds.height / 2,
        ),
      ).toBeLessThan(3);
      await noPageOverflow(page);
    };
    const expectVisibleFormula = async (field: typeof input) => {
      // WebKit reserves a couple of caret pixels within the leading padding.
      const { overflow, hiddenStart } = await field.evaluate((node) => ({
        overflow: node.scrollWidth - node.clientWidth,
        hiddenStart:
          Math.abs(node.scrollLeft) - parseFloat(getComputedStyle(node).paddingLeft),
      }));
      expect(overflow).toBeLessThanOrEqual(2);
      expect(hiddenStart).toBeLessThanOrEqual(0);
    };
    for (const draft of ["300+300", "888+888"]) {
      await input.fill("");
      await input.pressSequentially(draft);
      await expectVisibleFormula(input);
      await expectBoundedRow(input);
    }
    const formula = "300 - 10 + 5 - 2";
    await input.fill(formula);
    const grownWidth = (await input.boundingBox())!.width;
    expect(grownWidth).toBeGreaterThan(shortWidth + 8);
    await expectBoundedRow(input);
    await input.fill(`300${"+1".repeat(180)}`);
    await expectBoundedRow(input);
    expect(await input.evaluate((node) => node.scrollWidth > node.clientWidth)).toBe(
      true,
    );
    await input.fill("300");
    expect((await input.boundingBox())!.width).toBeLessThan(grownWidth);
    await input.press("Escape");
    await expect(current).toHaveText("300");
    await activate(current, info);
    await input.fill(formula);
    await input.press("Enter");
    await expect(current).toHaveText("293");

    await activate(
      page.getByRole("button", { name: "Add temp HP", exact: true }),
      info,
    );
    const temporary = page.getByRole("textbox", { name: "Temporary HP", exact: true });
    await temporary.pressSequentially("300+300");
    await expectVisibleFormula(temporary);
    await expectBoundedRow(temporary);
    await temporary.fill(`300${"+1".repeat(180)}`);
    await expectBoundedRow(temporary);
    expect(
      await temporary.evaluate((node) => node.scrollWidth > node.clientWidth),
    ).toBe(true);
    await temporary.fill(formula);
    await temporary.press("Enter");
    const editTemporary = page.getByRole("button", {
      name: "Edit Temporary HP",
      exact: true,
    });
    await expect(editTemporary).toHaveText("293");
    await activate(editTemporary, info);
    await temporary.fill(`300${"+1".repeat(180)}`);
    await expectBoundedRow(temporary);
    await temporary.fill("");
    await temporary.press("Enter");
    await expect(
      page.getByRole("button", { name: "Add temp HP", exact: true }),
    ).toBeFocused();
    const saved = await (await page.request.get(path)).json();
    expect(saved.participants[0].current_hp).toBe(293);
    expect(saved.participants[0].temporary_hp).toBe(0);
    await noPageOverflow(page);
  } finally {
    expect((await page.request.delete(path)).ok()).toBeTruthy();
  }
});

test("persistent form uses visible workarea, flat DC, accessible damage row and stable focused editing", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Forms checkpoint ${Date.now()}` },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name: "Form Hero",
            initiative: 20,
            current_hp: 30,
            max_hp: 30,
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(page.getByRole("button", { name: "Form Hero", exact: true }), info);
    await activate(
      page.getByRole("button", { name: "Add Condition", exact: true }),
      info,
    );
    const dialog = page.getByRole("dialog", { name: "Add condition", exact: true });
    const choice = dialog.getByRole("combobox", { name: "Add condition" });
    await choice.fill("Persistent Damage");
    await page
      .locator(".ant-select-dropdown:visible")
      .getByText("Persistent Damage", { exact: true })
      .click();
    const formula = dialog.getByRole("textbox", { name: "Damage formula" });
    await expect(formula).toHaveAttribute("placeholder", "Formula, e.g. 1d6");
    await expect(dialog.getByRole("spinbutton", { name: "Flat check DC" })).toHaveValue(
      "15",
    );
    await expect(
      dialog.getByRole("combobox", { name: "Persistent recovery" }),
    ).toHaveCount(0);
    await formula.fill("1d6");
    const type = dialog.getByRole("combobox", { name: "Damage type" });
    await type.fill("Fire");
    await page
      .locator(".ant-select-dropdown:visible")
      .getByText("Fire", { exact: true })
      .click();
    const formulaBounds = (await formula.boundingBox())!;
    const typeBounds = (await type.boundingBox())!;
    expect(
      Math.abs(
        formulaBounds.y +
          formulaBounds.height / 2 -
          typeBounds.y -
          typeBounds.height / 2,
      ),
    ).toBeLessThan(3);
    await expect(
      dialog.getByRole("button", { name: "Add duration", exact: true }),
    ).toBeVisible();
    await noPageOverflow(page);
    const bounds = (await dialog.boundingBox())!;
    const viewport = page.viewportSize()!;
    expect(bounds.y).toBeGreaterThanOrEqual(0);
    expect(bounds.y + bounds.height).toBeLessThanOrEqual(viewport.height + 1);
    const applyBounds = (await dialog
      .getByRole("button", { name: "Apply", exact: true })
      .boundingBox())!;
    expect(applyBounds.y + applyBounds.height).toBeLessThanOrEqual(
      bounds.y + bounds.height,
    );
    expect(
      await dialog
        .getByRole("button", { name: "Apply", exact: true })
        .evaluate((button) => !!(button as HTMLButtonElement).form),
    ).toBe(true);
    const before = await dialog.boundingBox();
    await formula.focus();
    await formula.press("End");
    await formula.pressSequentially("+1");
    await expect(formula).toBeFocused();
    expect(await dialog.boundingBox()).toEqual(before);
    await page.screenshot({
      path: info.outputPath("persistent-flat-form.png"),
      fullPage: true,
      animations: "disabled",
    });
    await activate(
      dialog.getByRole("button", { name: "Add duration", exact: true }),
      info,
    );
    await expect(dialog.getByRole("combobox", { name: "Whose turn" })).toBeVisible();
    await page.screenshot({
      path: info.outputPath("persistent-duration-form.png"),
      fullPage: true,
      animations: "disabled",
    });
    await activate(dialog.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(dialog).toHaveCount(0);
    const latest = await (await page.request.get(path)).json();
    expect(latest.participants[0].conditions[0].parameters).toMatchObject({
      formula: "1d6+1",
      damage_type: "fire",
      recovery: { kind: "specified", dc: 15 },
    });
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

test("incomplete persistent damage is a reminder without eager errors or automation", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Reminder ${Date.now()}` },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    await page.request.post(`${path}/participants/manual`, {
      data: {
        encounter_ref: encounter.encounter_key,
        display_name: "Reminder Hero",
        initiative: 20,
        current_hp: 30,
        max_hp: 30,
      },
    });
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Reminder Hero", exact: true }),
      info,
    );
    for (const label of ["Strike history", "Relationships", "Action context"])
      await expect(page.getByRole("button", { name: label, exact: true })).toHaveCount(
        0,
      );
    await activate(
      page.getByRole("button", { name: "Add Condition", exact: true }),
      info,
    );
    const dialog = page.getByRole("dialog", { name: "Add condition", exact: true });
    await dialog
      .getByRole("combobox", { name: "Add condition" })
      .fill("Persistent Damage");
    await page
      .locator(".ant-select-dropdown:visible")
      .getByText("Persistent Damage", { exact: true })
      .click();
    await expect(
      dialog.getByRole("button", { name: "Apply", exact: true }),
    ).toBeEnabled();
    await expect(dialog.locator(".ant-form-item-has-error")).toHaveCount(0);
    await activate(
      dialog.getByRole("button", { name: "Add details", exact: true }),
      info,
    );
    await dialog
      .getByRole("textbox", { name: "Condition note" })
      .fill("Burning until the blessing ends");
    await page.screenshot({
      path: info.outputPath("persistent-reminder.png"),
      animations: "disabled",
    });
    await activate(dialog.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(dialog).toBeHidden();
    let detail = await (await page.request.get(path)).json();
    expect(detail.participants[0].conditions[0].parameters.kind).toBe(
      "persistent_damage_reminder",
    );
    expect(detail.participants[0].persistent_damage_groups).toEqual([]);
    await page.reload();
    await activate(
      page.getByRole("button", { name: "Reminder Hero", exact: true }),
      info,
    );
    await expect(
      page.getByText("Burning until the blessing ends", { exact: true }),
    ).toBeVisible();
    await activate(
      page.getByRole("button", { name: "Edit Persistent Damage", exact: true }),
      info,
    );
    const edit = page.getByRole("dialog", {
      name: "Edit Persistent Damage",
      exact: true,
    });
    await edit.getByRole("textbox", { name: "Damage formula" }).fill("1d6");
    await edit.getByRole("combobox", { name: "Damage type" }).fill("Fire");
    await page
      .locator(".ant-select-dropdown:visible")
      .getByText("Fire", { exact: true })
      .click();
    await expect(
      edit.getByRole("button", { name: "Apply", exact: true }),
    ).toBeEnabled();
    await activate(edit.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(edit).toBeHidden();
    detail = await (await page.request.get(path)).json();
    expect(detail.participants[0].persistent_damage_groups).toHaveLength(1);
    expect(detail.participants[0].conditions[0].note).toBe(
      "Burning until the blessing ends",
    );
    const previousEpisode = detail.participants[0].conditions[0].parameters.episode_id;
    await activate(
      page.getByRole("button", { name: "Edit Persistent Damage", exact: true }),
      info,
    );
    await edit.getByRole("combobox", { name: "Damage type" }).fill("Cold");
    await page
      .locator(".ant-select-dropdown:visible")
      .getByText("Cold", { exact: true })
      .click();
    await expect(
      edit.getByRole("button", { name: "Apply", exact: true }),
    ).toBeEnabled();
    await activate(edit.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(edit).toBeHidden();
    detail = await (await page.request.get(path)).json();
    expect(detail.participants[0].conditions[0].parameters).toMatchObject({
      formula: "1d6",
      damage_type: "cold",
    });
    expect(detail.participants[0].conditions[0].parameters.episode_id).not.toBe(
      previousEpisode,
    );
    expect(detail.participants[0].persistent_damage_groups).toHaveLength(1);
    expect(detail.participants[0].conditions[0].note).toBe(
      "Burning until the blessing ends",
    );
  } finally {
    await page.request.delete(path);
  }
});

test("life status stays compact and stable while saving, without replacing the settings draft", async ({
  page,
}, info) => {
  const { encounter } = await (
    await page.request.post("/api/encounters", {
      data: { name: `Life settings ${Date.now()}` },
    })
  ).json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/record`, {
          data: {
            encounter_ref: encounter.encounter_key,
            record_ref: "browser-actors:sentinel00000001",
            quantity: 1,
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    const detail = await (await page.request.get(path)).json();
    await activate(
      page.locator(
        `[data-participant-key="${detail.participants[0].participant_key}"]`,
      ),
      info,
    );
    await activate(
      page.getByRole("button", { name: "Participant settings", exact: true }),
      info,
    );
    const dialog = page.getByRole("dialog", {
      name: "Participant settings",
      exact: true,
    });
    const name = dialog.getByRole("textbox", { name: "Participant name", exact: true });
    await name.fill("Unfinished draft");
    await activate(
      dialog.getByRole("checkbox", { name: "Track dying", exact: true }),
      info,
    );
    const dead = dialog.getByRole("radio", { name: "Dead", exact: true });
    await expect(dead).toBeEnabled();
    const group = dialog.getByRole("radiogroup", { name: "Life status" });
    const geometry = await group.evaluate((element) => {
      const labels = [...element.querySelectorAll("label")].map((e) =>
        e.getBoundingClientRect(),
      );
      const box = element.getBoundingClientRect();
      return {
        trailing: box.right - labels.at(-1)!.right,
        width: box.width,
        optionWidth: labels.reduce((n, r) => n + r.width, 0),
      };
    });
    expect(geometry.trailing).toBeLessThan(8);
    expect(geometry.width - geometry.optionWidth).toBeLessThan(12);
    await expect(
      dialog.getByRole("link", { name: "View creature reference", exact: true }),
    ).toHaveCount(0);
    await dead.scrollIntoViewIfNeeded();
    const nameNode = await name.elementHandle();
    const dialogNode = await dialog.elementHandle();
    let release!: () => void;
    const held = new Promise<void>((resolve) => {
      release = resolve;
    });
    await page.route("**/life-settings", async (route) => {
      await held;
      await route.continue();
    });
    await activate(dead.locator(".."), info);
    await expect(dead).toBeChecked();
    await expect(name).toBeEnabled();
    await expect(
      dialog.getByRole("button", { name: "Save", exact: true }),
    ).toBeDisabled();
    await page.screenshot({
      path: info.outputPath("life-status-pending.png"),
      animations: "disabled",
    });
    release();
    await expect(dead).toBeEnabled();
    await expect(dead).toBeChecked();
    expect(await nameNode!.evaluate((element) => element.isConnected)).toBe(true);
    expect(await dialogNode!.evaluate((element) => element.isConnected)).toBe(true);
    await expect(name).toHaveValue("Unfinished draft");
    await activate(
      dialog.getByRole("radio", { name: "Alive", exact: true }).locator(".."),
      info,
    );
    await expect(
      dialog.getByRole("radio", { name: "Alive", exact: true }),
    ).toBeChecked();
    await expect(
      dialog.getByRole("radio", { name: "Alive", exact: true }),
    ).toBeEnabled();
    await page.screenshot({
      path: info.outputPath("life-status-confirmed.png"),
      animations: "disabled",
    });
    await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(dialog).toBeHidden();
    const original = page.getByRole("link", {
      name: "View creature reference",
      exact: true,
    });
    await expect(original).toBeVisible();
    expect(await original.getAttribute("href")).toContain(
      "/records/browser-actors:sentinel00000001",
    );
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});
