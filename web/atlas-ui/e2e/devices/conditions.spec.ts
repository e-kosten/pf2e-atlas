import { expect, test } from "./fixtures";
import type { Page } from "@playwright/test";
import { activate, noPageOverflow } from "./reading";

async function selectOption(page: Page, name: string, value: string) {
  const control = page.getByRole("combobox", { name, exact: true });
  await control.fill(value, { timeout: 10000 });
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText(value, { exact: true })
    .click({ timeout: 10000 });
  await expect(page.locator(".ant-select-dropdown:visible")).toHaveCount(0);
}

test("persistent sources stay grouped after reload and Remove acts directly without a confirmation", async ({
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
            encounter_ref: encounter.encounter_key,
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
    const group = page.locator(".encounter-conditions").getByRole("button", {
      name: new RegExp(`^Persistent Damage.*${damageType.label}`),
    });
    await expect(group).toHaveCount(1);
    await activate(group, info);
    const remove = page.getByRole("button", {
      name: `Remove ${definition.name}`,
      exact: true,
    });
    await expect(remove).toHaveCount(2);
    await expect(remove.first()).toBeVisible();
    await expect(remove.first()).toBeEnabled();
    await remove.last().click({ trial: true });
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
    await page.request.delete(path);
  }
});
