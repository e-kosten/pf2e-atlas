import { expect, test } from "@playwright/test";
import { sentinel } from "./reading";

test("copy actions target the loaded list and record independently", async ({
  page,
}) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (value: string) =>
          sessionStorage.setItem("atlas-test-copy", value),
      },
    });
  });
  const slug = `copy-entity-link-${Date.now()}`;
  const create = await page.request.post("/api/lists", {
    data: { slug, name: "Link test list", tags: [] },
  });
  expect(create.ok()).toBe(true);
  const { list } = await create.json();
  try {
    const added = await page.request.post(`/api/lists/${slug}/items`, {
      data: { record_ref: sentinel },
    });
    expect(added.ok()).toBe(true);

    const copiedUrl = () =>
      page.evaluate(() => sessionStorage.getItem("atlas-test-copy"));
    await page.goto(`/lists/${slug}`);
    const listActions = page.getByRole("button", { name: "List actions", exact: true });
    await expect(listActions).toBeVisible();
    const listPosition = await listActions.boundingBox();
    await listActions.click();
    await page.getByRole("menuitem", { name: "Copy link", exact: true }).click();
    await expect(
      page.getByRole("tooltip").filter({ hasText: "Link copied" }),
    ).toBeVisible();
    expect(await listActions.boundingBox()).toEqual(listPosition);
    expect(await copiedUrl()).toBe(new URL(`/lists/${list.list_key}`, page.url()).href);

    await page.goto(`/lists/${slug}/records/${sentinel}`);
    const recordCopy = page.getByRole("button", { name: "Copy link", exact: true });
    await expect(recordCopy).toBeVisible();
    const recordPosition = await recordCopy.boundingBox();
    await recordCopy.click();
    await expect(
      page.getByRole("tooltip").filter({ hasText: "Link copied" }),
    ).toBeVisible();
    expect(await recordCopy.boundingBox()).toEqual(recordPosition);
    expect(await copiedUrl()).toBe(new URL(`/records/${sentinel}`, page.url()).href);

    await page.goto(`/lists/${slug}/edit`);
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect(page.getByRole("menuitem", { name: "Copy link" })).toHaveCount(0);
  } finally {
    await page.request.delete(`/api/lists/${slug}`);
  }
});
