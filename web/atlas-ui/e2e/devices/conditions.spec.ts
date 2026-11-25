import { expect, test } from "./fixtures";
import type { Page } from "@playwright/test";
import { activate, noPageOverflow } from "./reading";

test("condition edits recover from failed previews and stay open during Apply", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Condition submission ${info.project.name} ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  let release: (() => void) | undefined;
  try {
    const actorResponse = await page.request.post(`${path}/participants/manual`, {
      data: { display_name: "Condition hero", max_hp: 30, current_hp: 30 },
    });
    expect(actorResponse.ok()).toBeTruthy();
    const read = async () => (await page.request.get(path)).json();
    const initial = await read();
    const actor = initial.participants[0];
    const participant = `${path}/participants/${actor.participant_key}`;
    const catalog = await (
      await page.request.get(`${participant}/condition-definitions`)
    ).json();
    const frightened = catalog.conditions.find(
      (c: { name: string }) => c.name === "Frightened",
    );
    const added = await page.request.post(`${participant}/conditions`, {
      data: {
        request_id: "add-frightened",
        expected_encounter_revision: initial.revision,
        condition_ref: frightened.condition_ref,
        value: 1,
      },
    });
    expect(added.ok(), await added.text()).toBeTruthy();
    const before = await read();
    const condition = before.participants[0].conditions[0];
    const conditionPath = `${participant}/conditions/${condition.condition_id}`;
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.locator(`[data-participant-key="${actor.participant_key}"]`),
      info,
    );
    await page.route(
      `**${conditionPath}/preview`,
      async (route) => {
        const response = await route.fetch();
        await route.fulfill({
          response,
          status: 500,
          json: { code: "query_failed", message: "Preview unavailable" },
        });
      },
      { times: 1 },
    );
    const increase = page.getByRole("button", { name: "Increase Frightened value" });
    await activate(increase, info);
    await expect(
      page.getByText("Could not preview condition", { exact: true }),
    ).toBeVisible();
    await expect(
      page.getByRole("spinbutton", { name: "Frightened value" }),
    ).toHaveValue("1");
    expect((await read()).revision).toBe(before.revision);
    await page
      .locator(".encounter-conditions")
      .filter({ has: increase })
      .scrollIntoViewIfNeeded();
    await page.screenshot({
      path: info.outputPath("condition-preview-failed.png"),
      animations: "disabled",
    });
    await activate(increase, info);
    await expect
      .poll(async () => (await read()).participants[0].conditions[0].value)
      .toBe(2);
    const held = new Promise<void>((resolve) => {
      release = resolve;
    });
    await page.route(`**${conditionPath}/preview`, async (route) => {
      await held;
      await route.continue();
    });
    await activate(page.getByRole("button", { name: "Edit Frightened" }), info);
    const editor = page.getByRole("dialog", { name: "Edit Frightened" });
    await activate(editor.getByRole("button", { name: "Add details" }), info);
    await editor
      .getByRole("textbox", { name: "Condition note" })
      .fill("Review at round two");
    await activate(editor.getByRole("button", { name: "Apply" }), info);
    await expect(
      editor.getByRole("button", { name: "Remove", exact: true }),
    ).toBeDisabled();
    await expect(
      editor.getByRole("button", { name: "Increase Condition value" }),
    ).toBeDisabled();
    await page.keyboard.press("Escape");
    await page
      .locator(".ant-modal-wrap:visible")
      .click({ position: { x: 1, y: 1 }, force: true });
    await expect(editor).toBeVisible();
    await expect(editor.getByRole("button", { name: "Close" })).toHaveCount(0);
    await page.screenshot({
      path: info.outputPath("condition-apply-preview.png"),
      animations: "disabled",
    });
    release!();
    await expect(editor).toHaveCount(0);
    const saved = await read();
    expect(saved.participants[0].conditions[0]).toMatchObject({
      value: 2,
      note: "Review at round two",
    });
    expect(saved.participants[0].current_hp).toBe(30);
  } finally {
    release?.();
    await page.request.delete(path);
  }
});

async function selectOption(page: Page, name: string, value: string) {
  const control = page.getByRole("combobox", { name, exact: true });
  await control.fill(value, { timeout: 10000 });
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText(value, { exact: true })
    .click({ timeout: 10000 });
  await expect(page.locator(".ant-select-dropdown:visible")).toHaveCount(0);
}

test("persistent sources stay visible after reload and Remove acts directly without a confirmation", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Condition review ${info.project.name} ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            display_name: "Condition Hero",
            max_hp: 30,
            current_hp: 30,
          },
        })
      ).ok(),
    ).toBeTruthy();
    const detail = await (await page.request.get(path)).json();
    const actor = detail.participants[0];
    const catalog = await (
      await page.request.get(
        `${path}/participants/${actor.participant_key}/condition-definitions`,
      )
    ).json();
    const definition = catalog.conditions.find(
      (entry: { editor_kind: string }) => entry.editor_kind === "persistent_damage",
    );
    expect(definition).toBeTruthy();
    const damageType = catalog.damage_types.find(
      (entry: { value: string }) => entry.value === "fire",
    );
    expect(damageType).toBeTruthy();
    const openActor = async () => {
      const row = page.locator(`[data-participant-key="${actor.participant_key}"]`);
      if (info.project.use.hasTouch) await row.tap();
      else {
        await row.focus();
        await page.keyboard.press("Enter");
      }
      await expect(
        page.getByRole("button", { name: "Add Condition", exact: true }),
      ).toBeEnabled();
    };
    await page.goto(`/encounters/${encounter.slug}`);
    await openActor();
    for (const formula of ["1d6", "1d4"]) {
      const add = page.getByRole("button", { name: "Add Condition", exact: true });
      await expect(add).toBeEnabled();
      if (info.project.use.hasTouch) await activate(add, info);
      else {
        await add.focus();
        await page.keyboard.press("Enter");
      }
      const editor = page.getByRole("dialog", { name: "Add condition", exact: true });
      await expect(editor).toBeVisible();
      await selectOption(page, "Add condition", definition.name);
      await editor
        .getByRole("textbox", { name: "Damage formula", exact: true })
        .fill(formula);
      await selectOption(page, "Damage type", damageType.label);
      await noPageOverflow(page);
      if (formula === "1d6")
        await page.screenshot({
          path: info.outputPath("condition-editor.png"),
          fullPage: true,
          animations: "disabled",
        });
      await editor.getByRole("button", { name: "Apply", exact: true }).click();
      await expect(editor).toHaveCount(0);
    }
    const sourceCount = async () => {
      const latest = await (await page.request.get(path)).json();
      const groups = latest.participants[0].persistent_damage_groups;
      expect(groups).toHaveLength(1);
      return groups[0].sources.length;
    };
    await expect.poll(sourceCount).toBe(2);
    await page.reload();
    await openActor();
    const conditions = page.locator(".encounter-conditions");
    const links = conditions.getByRole("link", {
      name: new RegExp(`^Persistent Damage.*${damageType.label}`),
    });
    await expect(links).toHaveCount(2);
    await expect(links.first()).toBeVisible();
    await expect(links.last()).toBeVisible();
    await expect(conditions.getByText("Superseded", { exact: true })).toBeVisible();
    const remove = page.getByRole("button", {
      name: `Remove ${definition.name}`,
      exact: true,
    });
    await expect(remove).toHaveCount(2);
    await expect(remove.first()).toBeVisible();
    await expect(remove.first()).toBeEnabled();
    if (!info.project.use.hasTouch) {
      await remove.last().hover();
      await expect(
        page.getByRole("tooltip", { name: "Remove", exact: true }),
      ).toBeVisible();
    }
    await page.screenshot({
      path: info.outputPath("condition-sources.png"),
      fullPage: true,
      animations: "disabled",
    });
    await activate(remove.first(), info);
    await expect.poll(sourceCount).toBe(1);
    await expect(remove).toHaveCount(1);
    await expect(remove.first()).toBeEnabled();
    await expect(page.getByRole("dialog", { name: /Remove|Confirm/ })).toHaveCount(0);
    await noPageOverflow(page);
    await page.screenshot({
      path: info.outputPath("condition-after-remove.png"),
      fullPage: true,
      animations: "disabled",
    });
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});
