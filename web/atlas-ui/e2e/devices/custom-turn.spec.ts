import type { Locator } from "@playwright/test";
import {
  markdownSource,
  pressMarkdownShortcut,
  replaceMarkdown,
} from "./markdownEditor";
import { expect, test } from "./fixtures";

async function switchEditorMode(dialog: Locator, mode: "Write" | "Preview") {
  const frame = await dialog.getByText(mode, { exact: true }).evaluate((control) => {
    const editor = control.closest(".markdown-editor")!;
    const bold = editor.querySelector('button[aria-label="Bold"]')!;
    const source = editor.querySelector(".markdown-editor__source")!;
    const choice = control.closest("label")!;
    // Resolve the old styles before switching so a visibility transition can start.
    const visiblyEditing = () => getComputedStyle(bold).visibility === "visible";
    visiblyEditing();
    control.click();
    return new Promise((resolve) => {
      requestAnimationFrame(() => {
        resolve({
          preview: !!editor.querySelector(".markdown-editor__preview"),
          source: getComputedStyle(source).visibility === "visible",
          bold: visiblyEditing(),
          selected: choice.querySelector("input")!.checked,
          highlighted: getComputedStyle(choice).backgroundColor !== "rgba(0, 0, 0, 0)",
        });
      });
    });
  });

  expect(frame).toEqual({
    preview: mode === "Preview",
    source: mode === "Write",
    bold: mode === "Write",
    selected: true,
    highlighted: true,
  });
}

test("a custom turn previews its linked notes and becomes the active turn", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `Void Font rehearsal ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  try {
    await page.goto(`/encounters/${encounter.slug}`);
    await expect(page.getByText("Add a participant or custom turn.")).toBeVisible();
    await page.getByRole("button", { name: "Add participants" }).click();
    await page.getByRole("menuitem", { name: "Add custom turn…" }).click();
    const editor = page.getByRole("dialog", { name: "Add custom turn" });
    await editor.getByRole("textbox", { name: "Name" }).fill("Lair action · Void Font");
    await editor.getByRole("spinbutton", { name: "Initiative" }).fill("20");
    const compactNote = editor.getByRole("textbox", { name: "Notes" });
    await editor.locator("label", { hasText: "Notes" }).click();
    await expect(compactNote).toBeFocused();
    await switchEditorMode(editor, "Preview");
    await switchEditorMode(editor, "Write");

    const expand = editor.getByRole("button", { name: "Expand editor" });
    await expand.focus();
    await expand.press("Enter");
    const expanded = page.getByRole("dialog", { name: "Notes", exact: true });
    const collapse = expanded.getByRole("button", { name: "Collapse editor" });
    await expect(collapse).toBeFocused();
    const note = expanded.getByRole("textbox", { name: "Notes", exact: true });
    const prefix = "## Trigger\n\nThe font pulses. Resolve ";
    await replaceMarkdown(note, prefix);
    await expanded.getByRole("button", { name: "Heading", exact: true }).click();
    const headings = page.getByRole("menuitem", { name: "Heading 1", exact: true });
    await expect(headings).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(headings).toBeHidden();
    await expect(expanded).toBeVisible();

    await pressMarkdownShortcut(note, "End");
    await note.pressSequentially("[[");

    const picker = page.getByRole("dialog", { name: "Insert record link" });
    await expect(
      picker.getByRole("searchbox", { name: "Search records" }),
    ).toBeFocused();
    await picker.locator('[data-record-key="browser-spells:spell00000000001"]').click();
    await picker.getByRole("button", { name: "Insert link" }).click();
    await expect(picker).toBeHidden();
    await expect(note).toBeFocused();
    const undo = expanded.getByRole("button", { name: "Undo" });
    const redo = expanded.getByRole("button", { name: "Redo" });
    const withLink = prefix + "[Lantern Ward](record:browser-spells:spell00000000001)";
    await undo.click();
    await expect.poll(() => markdownSource(note)).toBe(prefix + "[[");
    await redo.click();
    await expect.poll(() => markdownSource(note)).toBe(withLink);
    await note.click();
    await pressMarkdownShortcut(note, "Home");
    await note.press("ArrowDown");
    await note.press("ArrowDown");
    await note.press("Home");
    for (let offset = 0; offset < 4; offset += 1) await note.press("ArrowRight");
    for (let offset = 0; offset < 4; offset += 1) await note.press("Shift+ArrowRight");
    const selectedText = () => page.evaluate(() => document.getSelection()?.toString());
    await expect.poll(selectedText).toBe("font");
    await page.keyboard.press("Escape");
    await expect(expand).toBeFocused();
    await expand.press("Enter");
    await expect(collapse).toBeFocused();
    await expanded.getByRole("button", { name: "Record link" }).click();
    await expect(
      picker.getByRole("searchbox", { name: "Search records" }),
    ).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(picker).toBeHidden();
    await expect(expanded).toBeVisible();
    await expect(note).toBeFocused();
    await expect.poll(selectedText).toBe("font");
    await pressMarkdownShortcut(note, "b");
    const formatted = withLink.replace("font", "**font**");
    await expect.poll(() => markdownSource(note)).toBe(formatted);
    await expect(note).toBeFocused();
    const height = await expanded.locator(".markdown-editor").boundingBox();
    await expanded.screenshot({ path: info.outputPath("custom-turn-write.png") });
    await switchEditorMode(expanded, "Preview");
    expect((await expanded.locator(".markdown-editor").boundingBox())?.height).toBe(
      height?.height,
    );
    await expanded.screenshot({ path: info.outputPath("custom-turn-preview.png") });
    await collapse.click();
    await expect(expand).toBeFocused();
    await expect(editor.getByRole("heading", { name: "Trigger" })).toBeVisible();
    await expect(editor.locator("strong")).toHaveText("font");
    await expect(editor.getByRole("link", { name: "Lantern Ward" })).toBeVisible();
    await switchEditorMode(editor, "Write");
    await editor.getByRole("button", { name: "Undo" }).click();
    await expect.poll(() => markdownSource(compactNote)).toBe(withLink);
    await editor.getByRole("button", { name: "Redo" }).click();
    await expect.poll(() => markdownSource(compactNote)).toBe(formatted);
    await editor.getByRole("button", { name: "Add", exact: true }).click();

    await page.getByRole("button", { name: "Start turn" }).click();
    const inspector = page.getByRole("region", { name: "Custom turn" });
    await expect(inspector).toBeVisible();
    const back = page.getByRole("button", { name: "Back to roster" });
    const current = page.getByRole("button", {
      name: "Lair action · Void Font, Current turn",
    });
    const narrow = await back.isVisible();
    if (narrow) await back.click();
    await expect(current).toBeVisible();
    if (narrow) await current.click();
    await inspector.getByRole("button", { name: "Edit custom turn" }).click();
    const edit = page.getByRole("dialog", { name: "Edit custom turn" });
    await expect(edit).toBeVisible();
    await edit.getByRole("button", { name: "Expand editor" }).click();
    await expect.poll(() => markdownSource(note)).toBe(formatted);
    await collapse.click();
    await expect(edit.getByRole("textbox", { name: "Name" })).toHaveValue(
      "Lair action · Void Font",
    );
    await edit.getByRole("button", { name: "Cancel" }).click();
    await expect(edit).toBeHidden();
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

test("Enter continues both kinds of lists and exits empty items", async ({
  page,
}, info) => {
  const created = await page.request.post("/api/encounters", {
    data: { name: `List continuation ${Date.now()}` },
  });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  try {
    await page.goto(`/encounters/${encounter.slug}`);
    await page.getByRole("button", { name: "Add participants" }).click();
    await page.getByRole("menuitem", { name: "Add custom turn…" }).click();
    const editor = page.getByRole("dialog", { name: "Add custom turn" });
    const note = editor.getByRole("textbox", { name: "Notes" });
    for (const [source, continuation, emptyExit] of [
      ["- First", "- First\n- ", "- First\n"],
      ["- First\n  - Nested", "- First\n  - Nested\n  - ", "- First\n  - Nested\n- "],
      ["7. First", "7. First\n8. ", "7. First\n"],
    ]) {
      await replaceMarkdown(note, source);
      await note.press("Enter");
      await expect.poll(() => markdownSource(note)).toBe(continuation);
      await note.press("Enter");
      await expect.poll(() => markdownSource(note)).toBe(emptyExit);
    }

    await replaceMarkdown(note, "1. First\n2. Third");
    await pressMarkdownShortcut(note, "Home");
    await note.press("End");
    await note.press("Enter");
    await expect.poll(() => markdownSource(note)).toBe("1. First\n2. \n3. Third");

    for (const text of ["Plain paragraph", "  Indented prose", "```\n- Inside code"]) {
      await replaceMarkdown(note, text);
      await note.press("Enter");
      await expect.poll(() => markdownSource(note)).toBe(text + "\n");
    }
    await replaceMarkdown(note, "One stage\nTwo stages\nThree stages");
    await pressMarkdownShortcut(note, "a");
    await editor.getByRole("button", { name: "Numbered list" }).click();
    await expect
      .poll(() => markdownSource(note))
      .toBe("1. One stage\n2. Two stages\n3. Three stages");
    await pressMarkdownShortcut(note, "z");
    await expect
      .poll(() => markdownSource(note))
      .toBe("One stage\nTwo stages\nThree stages");
    // Playwright needs the uppercase key value for a shifted letter on WebKit.
    await pressMarkdownShortcut(note, "Shift+Z", "y");
    await expect
      .poll(() => markdownSource(note))
      .toBe("1. One stage\n2. Two stages\n3. Three stages");
    await editor.screenshot({ path: info.outputPath("custom-turn-ordered-list.png") });
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});
