import { expect, test } from "./fixtures";
import { activate, noPageOverflow, sentinel } from "./reading";
import type { Locator, Page } from "@playwright/test";

async function insideViewport(page: Page, control: Locator) {
  await expect(control).toBeVisible();
  const rect = (await control.boundingBox())!;
  const size = page.viewportSize()!;
  expect(rect.y).toBeGreaterThanOrEqual(15);
  expect(rect.y + rect.height).toBeLessThanOrEqual(size.height - 15);
  expect(rect.x).toBeGreaterThanOrEqual(0);
  expect(rect.x + rect.width).toBeLessThanOrEqual(size.width + 1);
}

test("encounter forms retain their anchor through expanding fields and reclamp on resize", async ({
  page,
}, info) => {
  const { encounter } = await (
    await page.request.post("/api/encounters", {
      data: { name: `Modal anchor ${Date.now()}` },
    })
  ).json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.post(`${path}/participants/manual`, {
          data: {
            encounter_ref: encounter.encounter_key,
            display_name: "Anchor hero",
            max_hp: 30,
            current_hp: 30,
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Anchor hero", exact: true }),
      info,
    );
    const trigger = page.getByRole("button", { name: "Add Condition", exact: true });
    await activate(trigger, info);
    const dialog = page.getByRole("dialog", { name: "Add condition", exact: true });
    await dialog
      .getByRole("combobox", { name: "Add condition", exact: true })
      .fill("Frightened");
    await page
      .locator(".ant-select-dropdown:visible")
      .getByText("Frightened", { exact: true })
      .click();
    const top = (await dialog.boundingBox())!.y;
    await activate(
      dialog.getByRole("button", { name: "Add duration", exact: true }),
      info,
    );
    await activate(
      dialog.getByRole("button", { name: "Add details", exact: true }),
      info,
    );
    await expect.poll(async () => (await dialog.boundingBox())!.y).toBeCloseTo(top, 1);
    const apply = dialog.getByRole("button", { name: "Apply", exact: true });
    await insideViewport(page, apply);
    await page.setViewportSize({ ...page.viewportSize()!, height: 400 });
    await expect
      .poll(async () => {
        const box = (await dialog.boundingBox())!;
        return box.y + box.height;
      })
      .toBeLessThanOrEqual(385);
    await insideViewport(page, apply);
    const shortTop = (await dialog.boundingBox())!.y;
    await apply.focus();
    expect((await dialog.boundingBox())!.y).toBeCloseTo(shortTop, 1);
    await noPageOverflow(page);
    await activate(dialog.getByRole("button", { name: "Close", exact: true }), info);
    await expect(trigger).toBeFocused();
  } finally {
    await page.request.delete(path);
  }
});

test("short settings, party forms and browse workspaces show actions before focus", async ({
  page,
}, info) => {
  await page.setViewportSize({ ...page.viewportSize()!, height: 400 });
  await page.goto("/search");
  const settings = page.getByRole("button", { name: "Search settings", exact: true });
  await activate(settings, info);
  let dialog = page.getByRole("dialog", { name: "Search settings", exact: true });
  await insideViewport(
    page,
    dialog.getByRole("button", { name: "Apply", exact: true }),
  );
  await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);
  await expect(settings).toBeFocused();
  await page.goto("/encounters");
  await activate(page.getByRole("button", { name: "Parties", exact: true }), info);
  await activate(page.getByRole("button", { name: "New Party", exact: true }), info);
  dialog = page.getByRole("dialog", { name: "New Party", exact: true });
  await activate(dialog.getByRole("button", { name: "Add PC", exact: true }), info);
  await insideViewport(page, dialog.getByRole("button", { name: "Save", exact: true }));
  await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);

  const { encounter } = await (
    await page.request.post("/api/encounters", {
      data: { name: `Browse shell ${Date.now()}` },
    })
  ).json();
  const path = `/api/encounters/${encounter.encounter_key}`;
  try {
    expect(
      (
        await page.request.patch(`${path}/difficulty`, {
          data: {
            expected_revision: (await (await page.request.get(path)).json()).revision,
            reference: { party_level: 5, party_size: 4 },
          },
        })
      ).ok(),
    ).toBeTruthy();
    await page.goto(`/encounters/${encounter.slug}`);
    await activate(
      page.getByRole("button", { name: "Add participants", exact: true }),
      info,
    );
    await activate(
      page.getByRole("menuitem", { name: "Add creature or hazard…", exact: true }),
      info,
    );
    dialog = page.getByRole("dialog", { name: "Add creature or hazard", exact: true });
    const done = dialog.getByRole("button", { name: "Done", exact: true });
    await insideViewport(page, done);
    const box = (await dialog.boundingBox())!;
    const result = dialog.locator(`[data-record-key="${sentinel}"]`).first();
    await expect(result).toBeVisible();
    await activate(result.getByText("Atlas Sentinel", { exact: true }), info);
    await insideViewport(page, done);
    expect(await dialog.boundingBox()).toEqual(box);
    const reader = dialog.locator(".reference-reader__scroll");
    await expect(
      dialog
        .locator(".reference-reader")
        .getByRole("heading", { name: "Atlas Sentinel", exact: true }),
    ).toBeVisible();
    const scroll = await reader.evaluate((node) => {
      node.scrollTop = 100;
      return {
        top: node.scrollTop,
        height: node.clientHeight,
        content: node.scrollHeight,
      };
    });
    expect(scroll.height).toBeGreaterThan(30);
    expect(scroll.top).toBeGreaterThan(0);
    expect(scroll.content).toBeGreaterThan(scroll.height);
    await done.focus();
    expect(await dialog.boundingBox()).toEqual(box);
    await noPageOverflow(page);
    await activate(done, info);
  } finally {
    await page.request.delete(path);
  }
});

test("confirmations focus Cancel and hold their anchor through errors and viewport changes", async ({
  page,
}, info) => {
  const slug = `modal-confirm-${info.project.name}-${Date.now()}`;
  const { list } = await (
    await page.request.post("/api/lists", {
      data: { slug, name: "Modal confirmation", tags: [] },
    })
  ).json();
  try {
    await page.goto(`/lists/${slug}/edit`);
    const trigger = page.getByRole("button", { name: "Delete List", exact: true });
    await activate(trigger, info);
    const dialog = page.getByRole("dialog", {
      name: "Delete Modal confirmation?",
      exact: true,
    });
    const cancel = dialog.getByRole("button", { name: "Cancel", exact: true });
    await expect(cancel).toBeFocused();
    const top = (await dialog.boundingBox())!.y;
    await page.route(`**/api/lists/${list.list_key}`, async (route) => {
      if (route.request().method() !== "DELETE") return route.continue();
      await route.fulfill({
        status: 400,
        contentType: "application/json",
        body: JSON.stringify({
          code: "invalid_request",
          message:
            "The list is being edited. Close the other edit and try again. ".repeat(12),
        }),
      });
    });
    await activate(
      dialog.getByRole("button", { name: "Delete list", exact: true }),
      info,
    );
    await expect(dialog.getByRole("alert")).toContainText(
      "Delete list could not be confirmed",
    );
    expect((await dialog.boundingBox())!.y).toBeCloseTo(top, 1);
    await insideViewport(page, cancel);
    await page.setViewportSize({ ...page.viewportSize()!, height: 400 });
    await expect
      .poll(
        async () =>
          (await cancel.boundingBox())!.y + (await cancel.boundingBox())!.height,
      )
      .toBeLessThanOrEqual(385);
    await insideViewport(page, cancel);
    await activate(cancel, info);
    await expect(trigger).toBeFocused();
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});
