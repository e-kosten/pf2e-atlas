import { expect, test } from "./fixtures";

test("creating a duplicate links to the archived list and preserves the new-list draft", async ({
  page,
}, info) => {
  const slug = `archived-name-${info.project.name}-${Date.now()}`;
  const name = `Archived plans ${slug}`;
  const created = await page.request.post("/api/lists", { data: { slug, name } });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  const apiPath = `/api/lists/${list.list_key}`;
  try {
    const archived = await page.request.patch(apiPath, {
      data: { archived: { expected: false, value: true } },
    });
    expect(archived.ok()).toBeTruthy();
    await page.goto("/lists");
    await page.getByRole("button", { name: "New list" }).click();
    const dialog = page.getByRole("dialog", { name: "New list", exact: true });
    const field = dialog.getByRole("textbox", { name: /Name$/ });
    const draft = ` ${name.toUpperCase()} `;
    await field.fill(draft);
    await dialog.getByRole("button", { name: "Create list" }).click();
    await expect(field).toHaveAttribute("aria-invalid", "true");
    await expect(field).toHaveAccessibleDescription(/An archived list named/);
    const link = dialog.getByRole("link", { name, exact: true });
    await expect(link).toHaveAttribute("href", `/lists/${slug}`);
    const [existing] = await Promise.all([page.waitForEvent("popup"), link.click()]);
    await expect(existing).toHaveURL(new RegExp(`/lists/${slug}$`));
    await expect(existing.getByRole("heading", { name })).toBeVisible();
    await existing.close();
    await expect(field).toHaveValue(draft);
    await field.fill(`${name} revised`);
    await expect(field).not.toHaveAttribute("aria-invalid", "true");
    await expect(link).toBeHidden();
  } finally {
    await page.request.delete(apiPath);
  }
});
