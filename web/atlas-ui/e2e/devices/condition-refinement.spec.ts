import { expect, test } from "./fixtures";
import type { Locator, Page, TestInfo } from "@playwright/test";
import { activate, noPageOverflow } from "./reading";

async function checkpoint(page: Page, info: TestInfo, name: string) {
  await noPageOverflow(page);
  await expect(page).toHaveScreenshot(name, {
    animations: "disabled",
    maxDiffPixels: 0,
  });
  await page.screenshot({
    path: info.outputPath(name),
    animations: "disabled",
  });
}
async function choose(page: Page, name: string, value: string) {
  await page.getByRole("combobox", { name, exact: true }).fill(value);
  await page
    .locator(".ant-select-dropdown:visible")
    .getByText(value, { exact: true })
    .click();
}

async function recoveryGeometry(popup: Locator) {
  return popup.evaluate((panel) => {
    const input = panel.querySelector<HTMLInputElement>(
      'input[aria-label="Recovery DC"]',
    );
    if (!input) throw new Error("Recovery DC input is missing");
    const bounds = panel.getBoundingClientRect();
    const control = input.getBoundingClientRect();
    return {
      x: bounds.x,
      width: bounds.width,
      input: {
        x: control.x,
        y: control.y - bounds.y,
        width: control.width,
        height: control.height,
      },
    };
  });
}

test("precedence survives source removal and recovery retains its anchor after clearing", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: "Precedence regression" },
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
            display_name: "Precedence hero",
            initiative: 20,
            current_hp: 40,
            max_hp: 40,
          },
        })
      ).ok(),
    ).toBeTruthy();
    let detail = await (await page.request.get(path)).json();
    const actor = detail.participants[0];
    const catalog = await (
      await page.request.get(
        `${path}/participants/${actor.participant_key}/condition-definitions`,
      )
    ).json();
    const definition = catalog.conditions.find(
      (c: { editor_kind: string }) => c.editor_kind === "persistent_damage",
    );
    for (const [index, formula] of ["1d12", "2d6", "6d12"].entries()) {
      detail = await (await page.request.get(path)).json();
      const added = await page.request.post(
        `${path}/participants/${actor.participant_key}/conditions`,
        {
          data: {
            request_id: `source-${index}`,
            expected_encounter_revision: detail.revision,
            participant_key: actor.participant_key,
            condition_ref: definition.condition_ref,
            event_intent: "apply",
            decisions: { keep_upright: false },
            scope: { kind: "everyone" },
            extra_action: { kind: "unspecified" },
            immunity_override: false,
            parameters: {
              kind: "persistent_damage",
              formula,
              damage_type: "fire",
              multiplier: "normal",
              recovery: { kind: "specified", dc: 1 },
            },
          },
        },
      );
      expect(added.ok(), await added.text()).toBeTruthy();
    }
    detail = await (await page.request.get(path)).json();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Precedence hero", exact: true }),
      info,
    );
    await activate(
      page.getByRole("button", { name: "Edit Persistent Damage", exact: true }).last(),
      info,
    );
    const editor = page.getByRole("dialog", {
      name: "Edit Persistent Damage",
      exact: true,
    });
    const list = editor.getByRole("list", { name: "fire damage precedence" });
    await expect(list.getByRole("listitem")).toHaveCount(3);
    await expect(list.getByRole("listitem").first()).toContainText("6d12");
    const reorder = list.getByRole("button", {
      name: /^Reorder 1d12 fire/,
    });
    const targetKey = await list
      .getByRole("listitem")
      .nth(1)
      .getAttribute("data-sortable-key");
    await expect(reorder).toBeEnabled();
    await reorder.focus();
    await page.keyboard.press("Space", { delay: 20 });
    await expect(reorder).toHaveAttribute("aria-pressed", "true");
    await page.keyboard.press("ArrowUp");
    await expect(
      page
        .getByRole("status")
        .filter({ hasText: `was moved over droppable area ${targetKey}.` }),
    ).toBeVisible();
    await page.keyboard.press("Space");
    await expect(list.getByRole("listitem").nth(1)).toContainText("1d12");
    // A local reorder must not mutate the encounter before Apply.
    expect((await (await page.request.get(path)).json()).revision).toBe(
      detail.revision,
    );
    await checkpoint(page, info, "precedence-editor.png");
    await activate(editor.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(editor).toHaveCount(0);
    await checkpoint(page, info, "precedence-sources.png");
    const strongest = page
      .getByRole("link", { name: /Persistent Damage.*6d12/ })
      .locator("..");
    await activate(
      strongest.getByRole("button", { name: "Remove Persistent Damage", exact: true }),
      info,
    );
    await expect(
      page.getByRole("link", { name: /Persistent Damage.*6d12/ }),
    ).toHaveCount(0);
    detail = await (await page.request.get(path)).json();
    const group = detail.participants[0].persistent_damage_groups[0];
    expect(
      group.sources.find(
        (s: { condition_id: number }) => s.condition_id === group.selected_source_id,
      ).formula,
    ).toBe("1d12");
    await checkpoint(page, info, "precedence-fallback.png");
    const back = page.getByRole("button", { name: "Back to roster", exact: true });
    if (await back.isVisible()) await activate(back, info);
    await activate(
      page
        .getByRole("button", { name: "Start turn", exact: true })
        .filter({ visible: true }),
      info,
    );
    await activate(
      page
        .getByRole("button", { name: "Next turn", exact: true })
        .filter({ visible: true }),
      info,
    );
    const reminders = page.getByRole("dialog", { name: "Turn reminders", exact: true });
    await expect(
      reminders.getByRole("spinbutton", { name: "Fire damage amount", exact: true }),
    ).toBeVisible();
    await reminders
      .getByRole("spinbutton", { name: "Fire damage amount", exact: true })
      .fill("2");
    await checkpoint(page, info, "turn-damage.png");
    await activate(
      reminders.getByRole("button", { name: "Apply persistent damage", exact: true }),
      info,
    );
    await expect(
      reminders.getByText("Total: 2 damage applied", { exact: true }),
    ).toBeVisible();
    await checkpoint(page, info, "turn-damage-applied.png");
    await activate(
      reminders.getByRole("button", { name: "Close", exact: true }).last(),
      info,
    );
    const recovery = page.getByRole("button", {
      name: "Roll Fire recovery",
      exact: true,
    });
    await activate(recovery, info);
    const popup = page
      .locator(".ant-popover:visible")
      .filter({ hasText: "Fire recovery" });
    const dc = popup.getByRole("spinbutton", { name: "Recovery DC" });
    await expect(dc).toHaveValue("1");
    await expect(popup.getByRole("combobox")).toHaveCount(0);
    await expect(popup).toHaveCSS("opacity", "1");
    await checkpoint(page, info, "recovery-before.png");
    const before = await recoveryGeometry(popup);
    await activate(popup.getByRole("button", { name: "Roll", exact: true }), info);
    await expect(popup.getByText("Recovered · DC 1", { exact: true })).toBeVisible();
    await expect(recovery).toBeVisible();
    const after = await recoveryGeometry(popup);
    expect(Math.abs(after.x - before.x)).toBeLessThan(2);
    expect(Math.abs(after.width - before.width)).toBeLessThan(2);
    expect(after.input.x).toBe(before.input.x);
    expect(after.input.width).toBe(before.input.width);
    expect(after.input.height).toBe(before.input.height);
    // WebKit can clamp scroll when cleared rows shorten the reader. Compare the
    // controls inside their retained popover in one frame, independent of that scroll.
    expect(Math.abs(after.input.y - before.input.y)).toBeLessThan(1);
    await expect(dc).toBeDisabled();
    await checkpoint(page, info, "recovery-cleared-anchor.png");
    expect(
      (await (await page.request.get(path)).json()).participants[0].conditions,
    ).toHaveLength(0);
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

test("delayed condition previews retain geometry, focus and the HP opt-out", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: "Preview regression" },
  });
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    await page.request.post(`${path}/participants/manual`, {
      data: {
        encounter_ref: encounter.encounter_key,
        display_name: "Preview hero",
        max_hp: 40,
        current_hp: 40,
      },
    });
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Preview hero", exact: true }),
      info,
    );
    await activate(
      page.getByRole("button", { name: "Add Condition", exact: true }),
      info,
    );
    await choose(page, "Add condition", "Drained");
    const editor = page.getByRole("dialog", { name: "Add condition", exact: true });
    const effect = editor.getByRole("checkbox", {
      name: "Applies HP loss (Level needed)",
      exact: true,
    });
    await expect(effect).toBeEnabled();
    await effect.uncheck();
    const value = editor.getByRole("spinbutton", {
      name: "Condition value",
      exact: true,
    });
    await value.focus();
    await expect(editor).toHaveCSS("opacity", "1");
    const before = await editor.boundingBox();
    const monitor = await editor.evaluateHandle((node) => {
      const preview = node.querySelector('[aria-label="Changes on application"]')!;
      const original = node.getBoundingClientRect();
      const failures = new Set<string>();
      let frames = 0;
      let frame: number;
      const sample = () => {
        frames++;
        const rect = node.getBoundingClientRect();
        if (!preview.isConnected) failures.add("preview unmounted");
        if (getComputedStyle(node).opacity !== "1") failures.add("dialog flickered");
        if (
          (["x", "y", "width", "height"] as const).some(
            (key) => Math.abs(rect[key] - original[key]) > 1,
          )
        )
          failures.add("dialog moved");
        if (document.activeElement?.getAttribute("aria-label") !== "Condition value")
          failures.add("focus moved");
        frame = requestAnimationFrame(sample);
      };
      frame = requestAnimationFrame(sample);
      return {
        finish: () => {
          cancelAnimationFrame(frame);
          return { frames, failures: [...failures] };
        },
      };
    });
    let release!: () => void;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    let requested!: () => void;
    const started = new Promise<void>((resolve) => {
      requested = resolve;
    });
    await page.route("**/conditions/preview", async (route) => {
      requested();
      await gate;
      await route.continue();
    });
    try {
      await value.fill("2");
      await started;
      await expect(value).toBeFocused();
      await expect(effect).not.toBeChecked();
      await expect(effect).toBeDisabled();
      expect(await editor.boundingBox()).toEqual(before);
      await checkpoint(page, info, "preview-pending.png");
    } finally {
      release();
    }
    await expect(effect).toBeEnabled();
    // A changed magnitude is a new consequence and resets the earlier opt-out.
    await expect(effect).toBeChecked();
    await expect(value).toBeFocused();
    expect(await editor.boundingBox()).toEqual(before);
    await checkpoint(page, info, "preview-settled.png");
    const observed = await monitor.evaluate((state) => state.finish());
    expect(observed.frames).toBeGreaterThan(0);
    expect(observed.failures).toEqual([]);
    await effect.uncheck();
    await activate(editor.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(editor).toHaveCount(0);
    const actor = (await (await page.request.get(path)).json()).participants[0];
    expect(actor.current_hp).toBe(40);
    expect(actor.conditions[0].value).toBe(2);
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});

test("Quickened keeps action restrictions through edits and clears hidden guidance", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: "Quickened regression" },
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
            display_name: "Quickened hero",
            max_hp: 40,
            current_hp: 40,
          },
        })
      ).ok(),
    ).toBeTruthy();
    let detail = await (await page.request.get(path)).json();
    const actor = detail.participants[0];
    const catalog = await (
      await page.request.get(
        `${path}/participants/${actor.participant_key}/condition-definitions`,
      )
    ).json();
    const definition = catalog.conditions.find(
      (entry: { supports_extra_action: boolean }) => entry.supports_extra_action,
    );
    expect(
      (
        await page.request.post(
          `${path}/participants/${actor.participant_key}/conditions`,
          {
            data: {
              request_id: "quickened-source",
              participant_key: actor.participant_key,
              expected_encounter_revision: detail.revision,
              condition_ref: definition.condition_ref,
              event_intent: "apply",
              decisions: { keep_upright: false },
              scope: { kind: "everyone" },
              extra_action: {
                kind: "restricted",
                actions: ["stride", "strike"],
                guidance: "Only with the granted weapon",
              },
              immunity_override: false,
              parameters: { kind: "none" },
            },
          },
        )
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Quickened hero", exact: true }),
      info,
    );
    await activate(
      page.getByRole("button", { name: "Edit Quickened", exact: true }),
      info,
    );
    let dialog = page.getByRole("dialog", { name: "Edit Quickened", exact: true });
    await expect(
      dialog.getByRole("textbox", { name: "Custom restriction" }),
    ).toHaveValue("Only with the granted weapon");
    await expect(dialog.getByText("Stride", { exact: true })).toBeVisible();
    await expect(dialog.getByText("Strike", { exact: true })).toBeVisible();
    await checkpoint(page, info, "quickened-restrictions.png");
    await activate(dialog.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(dialog).toHaveCount(0);
    detail = await (await page.request.get(path)).json();
    expect(detail.participants[0].conditions[0].extra_action).toEqual({
      kind: "restricted",
      actions: ["stride", "strike"],
      guidance: "Only with the granted weapon",
    });
    await activate(
      page.getByRole("button", { name: "Edit Quickened", exact: true }),
      info,
    );
    dialog = page.getByRole("dialog", { name: "Edit Quickened", exact: true });
    await activate(
      dialog.getByRole("combobox", { name: "Extra action", exact: true }),
      info,
    );
    await activate(
      page
        .locator(".ant-select-dropdown:visible")
        .getByText("Unrestricted", { exact: true }),
      info,
    );
    await expect(
      dialog.getByRole("textbox", { name: "Custom restriction" }),
    ).toHaveCount(0);
    await dialog
      .getByRole("combobox", { name: "Extra action", exact: true })
      .press("Escape");
    await expect(page.locator(".ant-select-dropdown:visible")).toHaveCount(0);
    await activate(dialog.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(dialog).toHaveCount(0);
    detail = await (await page.request.get(path)).json();
    expect(detail.participants[0].conditions[0].extra_action).toEqual({
      kind: "unrestricted",
    });
    await activate(
      page.getByRole("button", { name: "Edit Quickened", exact: true }),
      info,
    );
    dialog = page.getByRole("dialog", { name: "Edit Quickened", exact: true });
    await activate(
      dialog.getByRole("combobox", { name: "Extra action", exact: true }),
      info,
    );
    await activate(
      page.locator(".ant-select-dropdown:visible").getByText("Stride", { exact: true }),
      info,
    );
    await dialog
      .getByRole("combobox", { name: "Extra action", exact: true })
      .press("Escape");
    await expect(page.locator(".ant-select-dropdown:visible")).toHaveCount(0);
    await activate(dialog.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(dialog).toHaveCount(0);
    detail = await (await page.request.get(path)).json();
    expect(detail.participants[0].conditions[0].extra_action).toEqual({
      kind: "restricted",
      actions: ["stride"],
      guidance: null,
    });
  } finally {
    await page.request.delete(path, { timeout: 5000 }).catch(() => undefined);
  }
});
