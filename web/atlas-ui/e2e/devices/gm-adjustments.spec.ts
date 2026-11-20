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
    await expect
      .poll(async () => {
        const fieldBottom = await editor
          .getByRole("combobox", { name: "Lifetime" })
          .evaluate(
            (element) =>
              element.closest(".ant-form-item")!.getBoundingClientRect().bottom,
          );
        const cancel = await editor
          .getByRole("button", { name: "Cancel", exact: true })
          .boundingBox();
        const gap = cancel!.y - fieldBottom;
        return gap >= 0 && gap <= 48;
      })
      .toBe(true);
    await addStatistic.fill("AC");
    await activate(
      page.locator(".ant-select-dropdown:visible").getByText("AC", { exact: true }),
      info,
    );
    await editor.getByRole("spinbutton", { name: "Change" }).fill("-2");
    await editor.getByRole("textbox", { name: "Name" }).fill("Sacrifice Armor");
    await activate(editor.getByRole("combobox", { name: "Lifetime" }), info);
    await activate(
      page.locator(".ant-select-dropdown:visible").getByText("Timed", { exact: true }),
      info,
    );
    await activate(editor.getByRole("combobox", { name: "Duration unit" }), info);
    await activate(
      page.locator(".ant-select-dropdown:visible").getByText("hours", { exact: true }),
      info,
    );
    await editor.getByRole("spinbutton", { name: "Duration amount" }).fill("24");
    await expect
      .poll(async () => {
        const statisticTop = await addStatistic.evaluate(
          (element) => element.closest(".ant-form-item")!.getBoundingClientRect().top,
        );
        const changeTop = await editor
          .getByRole("spinbutton", { name: "Change" })
          .evaluate(
            (element) => element.closest(".ant-form-item")!.getBoundingClientRect().top,
          );
        return Math.abs(statisticTop - changeTop) < 1;
      })
      .toBe(true);
    await expect(editor.getByText("AC 22 → 20")).toBeVisible();
    await expect(editor.getByText("Untyped", { exact: true })).toBeVisible();
    await activate(editor.getByRole("combobox", { name: "Modifier type" }), info);
    await activate(
      page.locator(".ant-select-dropdown:visible").getByText("Item", { exact: true }),
      info,
    );
    await expect(editor.getByText("AC 22 → 20")).toBeVisible();
    await expect(
      editor.getByRole("spinbutton", { name: "Change" }),
    ).not.toHaveAttribute("placeholder");
    await expect(
      editor.getByRole("button", { name: "Apply", exact: true }),
    ).toBeEnabled();
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
    await expect(page.locator(".ant-popover:visible")).not.toHaveClass(
      /ant-zoom-big-(appear|enter)/,
    );
    const table = explanation.getByRole("region", { name: "Value calculation" });
    const header = explanation.locator(".value-explanation__header");
    await expect
      .poll(async () => {
        const tableBounds = await table.boundingBox();
        const headerBounds = await header.boundingBox();
        return (
          !!tableBounds &&
          !!headerBounds &&
          headerBounds.width > 0 &&
          Math.abs(tableBounds.width - headerBounds.width) < 1
        );
      })
      .toBe(true);
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
    const remaining = edit.getByRole("spinbutton", { name: "Remaining seconds" });
    await expect(remaining).toHaveAttribute("placeholder", "86400");
    await noPageOverflow(page);
    if (
      info.project.name === "desktop-chromium" ||
      info.project.name === "tablet-webkit"
    ) {
      await expect
        .poll(async () => {
          const duration = await edit
            .getByRole("spinbutton", { name: "Duration amount" })
            .boundingBox();
          const timeLeft = await remaining.boundingBox();
          return !!duration && !!timeLeft && Math.abs(duration.y - timeLeft.y) < 2;
        })
        .toBe(true);
    }
    await page.screenshot({
      path: info.outputPath("gm-adjustment-duration.png"),
      animations: "disabled",
    });
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
    await expect(
      end.getByRole("button", { name: "Cancel", exact: true }),
    ).toBeFocused();
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
