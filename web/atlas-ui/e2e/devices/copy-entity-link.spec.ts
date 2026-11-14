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
    await page.getByRole("button", { name: "Copy list link", exact: true }).click();
    expect(await copiedUrl()).toBe(new URL(`/lists/${list.list_key}`, page.url()).href);

    await page.goto(`/lists/${slug}/records/${sentinel}`);
    await page.getByRole("button", { name: "Copy record link", exact: true }).click();
    expect(await copiedUrl()).toBe(new URL(`/records/${sentinel}`, page.url()).href);

    await page.goto(`/lists/${slug}/edit`);
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect(page.getByRole("button", { name: "Copy list link" })).toHaveCount(0);
  } finally {
    await page.request.delete(`/api/lists/${slug}`);
  }
});
