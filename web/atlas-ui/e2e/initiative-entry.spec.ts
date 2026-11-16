import { expect, test } from "@playwright/test";

test("an initiative entry previews its linked notes and becomes the active turn", async ({
  page,
}) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Void Font rehearsal ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  try {
    await page.goto(`/encounters/${encounter.slug}`);
    await page.getByRole("button", { name: "Add participants" }).click();
    await page.getByRole("menuitem", { name: "Add initiative entry…" }).click();
    const editor = page.getByRole("dialog", { name: "Add initiative entry" });
    await editor.getByRole("textbox", { name: "Name" }).fill("Lair action · Void Font");
    await editor.getByRole("spinbutton", { name: "Initiative" }).fill("20");
    const note = editor.getByRole("textbox", { name: "Notes" });
    await note.fill("## Trigger\n\nThe font pulses. Resolve ");
    await note.press("End");
    await note.pressSequentially("[[");

    const picker = page.getByRole("dialog", { name: "Insert record link" });
    await expect(
      picker.getByRole("searchbox", { name: "Search records" }),
    ).toBeFocused();
    await picker.locator('[data-record-key="browser-spells:spell00000000001"]').click();
    await picker.getByRole("button", { name: "Insert link" }).click();
    await expect(picker).toBeHidden();
    await expect(note).toBeFocused();
    await note.evaluate((textarea: HTMLTextAreaElement) => {
      const start = textarea.value.indexOf("font");
      textarea.setSelectionRange(start, start + "font ".length);
    });
    await editor.getByRole("button", { name: "Bold" }).click();
    await expect(note).toHaveValue(/The \*\*font\*\* pulses/);
    await expect(note).toBeFocused();
    await editor.getByText("Preview", { exact: true }).click();
    await expect(editor.getByRole("heading", { name: "Trigger" })).toBeVisible();
    await expect(editor.locator("strong")).toHaveText("font");
    await expect(editor.getByRole("link", { name: "Lantern Ward" })).toBeVisible();
    await editor.getByText("Write", { exact: true }).click();
    await expect(note).toHaveValue(/record:browser-spells:spell00000000001/);
    await editor.getByRole("button", { name: "Add", exact: true }).click();

    await page.getByRole("button", { name: "Start turn" }).click();
    await expect(
      page.getByRole("button", { name: "Lair action · Void Font, Current turn" }),
    ).toBeVisible();
    await expect(page.getByRole("heading", { name: "Trigger" })).toBeVisible();
    const link = page.getByRole("link", { name: "Lantern Ward" });
    await link.focus();
    await link.press("Enter");
    await expect(
      page.getByRole("heading", { name: "Lantern Ward", exact: true }),
    ).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(link).toBeFocused();
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});
