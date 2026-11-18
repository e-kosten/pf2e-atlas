import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";

test("GM adjustments preview and change multiple stats through the participant inspector", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `GM adjustment review ${info.project.name} ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/record`, {
          data: {
            record_ref: "browser-actors:sentinel00000001",
            quantity: 1,
            initiative: 20,
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Atlas Sentinel", exact: true }),
      info,
    );
    const adjustments = page.getByRole("region", { name: "Adjustments" });
    const add = adjustments.getByRole("button", { name: "Add adjustment" });
    await activate(add, info);
    const editor = page.getByRole("dialog", { name: "Add adjustment" });
    const addStatistic = editor.getByRole("combobox", { name: "Statistic" });
    await expect(addStatistic).toBeFocused();
    await addStatistic.fill("AC");
    await activate(
      page.locator(".ant-select-dropdown:visible").getByText("AC", { exact: true }),
      info,
    );
    await editor.getByRole("spinbutton", { name: "Change" }).fill("-2");
    await editor.getByRole("textbox", { name: "Name" }).fill("Sacrifice Armor");
    await expect(editor.getByText("AC 22 → 20")).toBeVisible();
    await expect(editor.getByText("Untyped", { exact: true })).toBeVisible();
    await expect(
      editor.getByRole("spinbutton", { name: "Change" }),
    ).not.toHaveAttribute("placeholder");
    let releasePreview: (() => void) | undefined;
    const heldPreview = new Promise<void>((resolve) => {
      releasePreview = resolve;
    });
    const previewUrl = (url: URL) => url.pathname.endsWith("/adjustments/preview");
    await page.route(previewUrl, async (route) => {
      await heldPreview;
      await route.continue();
    });
    try {
      const apply = editor.getByRole("button", { name: "Apply", exact: true });
      const originalLayout = await apply.boundingBox();
      await editor.getByRole("spinbutton", { name: "Change" }).fill("-3");
      await expect(apply).toBeDisabled();
      await expect(editor.getByText("AC 22 → 20")).toBeVisible();
      await expect(editor.getByText("Calculating preview…")).toBeHidden();
      await expect(
        editor.getByRole("status", { name: "Calculating preview…" }),
      ).toBeVisible();
      expect(await apply.boundingBox()).toEqual(originalLayout);
    } finally {
      releasePreview?.();
    }
    await expect(editor.getByText("AC 22 → 19")).toBeVisible();
    await page.unroute(previewUrl);
    await editor.getByRole("spinbutton", { name: "Change" }).fill("-2");
    await expect(editor.getByText("AC 22 → 20")).toBeVisible();
    await noPageOverflow(page);
    await expect
      .poll(() =>
        editor.evaluate((element) => {
          const { top, bottom } = element.getBoundingClientRect();
          return top >= 0 && bottom <= innerHeight;
        }),
      )
      .toBe(true);
    await page.screenshot({
      path: info.outputPath("gm-adjustment-editor.png"),
      animations: "disabled",
    });
    await activate(editor.getByRole("button", { name: "Apply", exact: true }), info);
    await expect(editor).toHaveCount(0);
    await expect(
      adjustments.getByText("Sacrifice Armor", { exact: true }),
    ).toBeVisible();
    const ac = page.getByRole("button", {
      name: "Show value explanation: 20",
      exact: true,
    });
    await activate(ac, info);
    await expect(page.getByRole("dialog").getByText("Sacrifice Armor")).toBeVisible();
    const explanation = page.getByRole("dialog", { name: "Show value explanation" });
    const table = explanation.getByRole("region", { name: "Value calculation" });
    const header = explanation.locator(".value-explanation__header");
    const tableBounds = await table.boundingBox();
    const headerBounds = await header.boundingBox();
    expect(tableBounds).not.toBeNull();
    expect(headerBounds).not.toBeNull();
    expect(tableBounds!.width).toBeCloseTo(headerBounds!.width, 0);
    await activate(page.getByRole("button", { name: "Close explanation" }), info);
    await activate(ac, info);
    await page
      .getByRole("heading", { name: "Statistics", exact: true })
      .evaluate((element) => element.scrollIntoView({ block: "start" }));
    await expect(explanation).toHaveCount(0);

    await activate(
      adjustments.getByRole("button", { name: "Edit Sacrifice Armor" }),
      info,
    );
    const edit = page.getByRole("dialog", { name: "Edit adjustment" });
    await expect(edit.getByRole("spinbutton", { name: "Change" })).toBeFocused();
    const editStatistic = edit.getByRole("combobox", { name: "Statistic" });
    await activate(editStatistic, info);
    await editStatistic.fill("Fortitude");
    await activate(
      page
        .locator(".ant-select-dropdown:visible")
        .getByText("Fortitude", { exact: true }),
      info,
    );
    await expect(edit.getByText("AC 20 → 22")).toBeVisible();
    await expect(edit.getByText("Fortitude 15 → 13")).toBeVisible();
    await activate(edit.getByRole("button", { name: "Save changes" }), info);
    await expect(edit).toHaveCount(0);
    await expect(adjustments.getByText("Fortitude −2", { exact: false })).toBeVisible();

    await activate(
      adjustments.getByRole("button", { name: "End Sacrifice Armor now" }),
      info,
    );
    const end = page.getByRole("dialog", { name: "End adjustment" });
    await expect(end.getByText("Fortitude 13 → 15")).toBeVisible();
    await activate(end.getByRole("button", { name: "End now" }), info);
    await expect(end).toHaveCount(0);
    await expect(adjustments.getByText("Sacrifice Armor", { exact: true })).toHaveCount(
      0,
    );
  } finally {
    await page.request.delete(path);
  }
});
