import { markdownSource, replaceMarkdown } from "./markdownEditor";
import type { Page } from "@playwright/test";
import { expect, test } from "./fixtures";

const first = "browser-rules:rule000000000001";
const second = "browser-conditions:offguard00000001";
const original = "Original note";

async function createList(page: Page, slug: string) {
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Note review" },
  });
  expect(created.ok()).toBeTruthy();
  return (await created.json()).list.list_key as string;
}

async function addItems(page: Page, slug: string) {
  for (const record of [first, second]) {
    const added = await page.request.post(`/api/lists/${slug}/items`, {
      data: {
        record_ref: record,
        ...(record === first ? { note: original } : {}),
      },
    });
    expect(added.ok()).toBeTruthy();
  }
}

async function openNote(page: Page, slug: string) {
  await page.goto(`/lists/${slug}/records/${first}`);
  const region = page.getByRole("region", { name: "List entry note" });
  await region.getByRole("button", { name: "Edit", exact: true }).click();
  const draft = region.getByRole("textbox", { name: "Note", exact: true });
  await expect.poll(() => markdownSource(draft)).toBe(original);
  return { region, draft };
}

test("two tabs saving the same note finish without review", async ({
  page,
  context,
}, info) => {
  const slug = `note-match-${info.project.name}-${Date.now()}`;
  const other = await context.newPage();
  const key = await createList(page, slug);
  try {
    await addItems(page, slug);
    const left = await openNote(page, slug);
    const right = await openNote(other, slug);
    const text = "Both tabs made the same edit";
    await page.bringToFront();
    await replaceMarkdown(left.draft, text);
    await expect.poll(() => markdownSource(left.draft)).toBe(text);
    await other.bringToFront();
    await replaceMarkdown(right.draft, text);
    await expect.poll(() => markdownSource(right.draft)).toBe(text);
    await right.region.getByRole("button", { name: "Save", exact: true }).click();
    await expect(right.region.getByText(text, { exact: true })).toBeVisible();
    const staleSave = page.waitForResponse(
      (response) =>
        response.request().method() === "PATCH" && response.url().endsWith("/note"),
    );
    await left.region.getByRole("button", { name: "Save", exact: true }).click();
    expect((await staleSave).status()).toBe(409);
    await expect(left.region.getByText(text, { exact: true })).toBeVisible();
    await expect(
      page.getByRole("dialog", { name: "Review note conflict" }),
    ).toBeHidden();
    await expect(
      left.region.getByRole("button", { name: "Edit", exact: true }),
    ).toBeFocused();
  } finally {
    await other.close();
    await page.request.delete(`/api/lists/${key}`);
  }
});

test("reorder in the same list view preserves an open note draft and both saves", async ({
  page,
}, info) => {
  test.skip(
    Boolean(info.project.use.hasTouch),
    "The desktop list and reader are side by side",
  );
  const slug = `note-reorder-${Date.now()}`;
  const key = await createList(page, slug);
  try {
    await addItems(page, slug);
    const { region, draft } = await openNote(page, slug);
    await replaceMarkdown(draft, "Edited after opening");
    const handle = page.locator(".list-member__drag-handle").first();
    await expect(handle).toBeEnabled();
    await handle.focus();
    await expect(handle).toBeFocused();
    await page.keyboard.press("Space", { delay: 20 });
    await expect(handle).toHaveAttribute("aria-pressed", "true");
    await page.keyboard.press("ArrowDown");
    await expect(
      page.getByRole("status").filter({ hasText: "was moved over droppable area" }),
    ).toContainText(`was moved over droppable area ${second}`);
    await page.keyboard.press("Space");
    await expect
      .poll(async () => {
        const response = await page.request.get(`/api/lists/${slug}`);
        expect(response.ok()).toBeTruthy();
        return (await response.json()).items.map(
          (item: { record_key: string }) => item.record_key,
        );
      })
      .toEqual([second, first]);
    await expect.poll(() => markdownSource(draft)).toBe("Edited after opening");
    await region.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      region.getByText("Edited after opening", { exact: true }),
    ).toBeVisible();
    const saved = await page.request.get(`/api/lists/${slug}`);
    expect(
      (await saved.json()).items.map((item: { record_key: string; note?: string }) => [
        item.record_key,
        item.note ?? null,
      ]),
    ).toEqual([
      [second, null],
      [first, "Edited after opening"],
    ]);
  } finally {
    await page.request.delete(`/api/lists/${key}`);
  }
});

test("same-note saves from two tabs require explicit conflict resolution", async ({
  page,
  context,
}, info) => {
  const slug = `note-conflict-${info.project.name}-${Date.now()}`;
  const keyboard = !info.project.use.hasTouch;
  if (keyboard) await page.setViewportSize({ width: 1200, height: 440 });
  const other = await context.newPage();
  const key = await createList(page, slug);
  try {
    await addItems(page, slug);
    const left = await openNote(page, slug);
    const right = await openNote(other, slug);
    const draftText = "My draft\n" + "More draft details\n".repeat(25) + "End of draft";
    const savedText =
      "Other tab saved\n" + "More saved details\n".repeat(25) + "End of saved note";
    await replaceMarkdown(left.draft, draftText);
    await replaceMarkdown(right.draft, savedText);
    await right.region.getByRole("button", { name: "Save", exact: true }).click();
    await expect(
      right.region.getByText("Other tab saved").filter({ visible: true }),
    ).toBeVisible();
    let failedRefresh = false;
    await page.route("**/items/*/note", async (route) => {
      if (route.request().method() === "GET" && !failedRefresh) {
        failedRefresh = true;
        await route.abort();
      } else {
        await route.continue();
      }
    });
    const reader = page.locator(".list-workspace > .reference-browser");
    const beforeReview = await reader.boundingBox();
    await left.region.getByRole("button", { name: "Save", exact: true }).click();
    const review = page.getByRole("dialog", { name: "Review note conflict" });
    const retry = review.getByRole("button", {
      name: "Retry loading saved note",
    });
    await expect(retry).toBeInViewport();
    await expect.poll(() => markdownSource(left.draft)).toBe(draftText);
    expect(await reader.boundingBox()).toEqual(beforeReview);
    await review.getByRole("button", { name: "Back to note" }).press("Enter");
    const reopen = left.region.getByRole("button", { name: "Review conflict" });
    await expect(review).toBeHidden();
    await expect(reopen).toBeFocused();
    await expect.poll(() => markdownSource(left.draft)).toBe(draftText);
    await reopen.press("Enter");
    if (keyboard) {
      await expect(
        review.getByText("Review note conflict", { exact: true }),
      ).toBeFocused();
      await retry.press("Enter");
    } else {
      await retry.tap();
    }
    await expect(review.getByText("Latest saved note", { exact: true })).toBeVisible();
    await expect(review.getByText("Your version", { exact: true })).toBeVisible();
    await expect(review.getByText("Other tab saved")).toBeVisible();
    await expect.poll(() => markdownSource(left.draft)).toBe(draftText);
    if (keyboard) {
      await expect(review.getByText("Review note conflict")).toBeFocused();
      await page.keyboard.press("Tab");
      await expect(review.getByRole("region", { name: "Note versions" })).toBeFocused();
      const body = review.locator(".ant-modal-body");
      expect(await body.evaluate((el) => el.scrollHeight)).toBeGreaterThan(
        await body.evaluate((el) => el.clientHeight),
      );
      await page.keyboard.press("PageDown");
      await expect.poll(() => body.evaluate((el) => el.scrollTop)).toBeGreaterThan(0);
      await page.keyboard.press("Tab");
      await expect(review.getByRole("button", { name: "Back to note" })).toBeFocused();
      await page.keyboard.press("Tab");
      await expect(review.getByRole("button", { name: "Use saved" })).toBeFocused();
      await page.keyboard.press("Tab");
      await expect(review.getByRole("button", { name: "Overwrite" })).toBeFocused();
    }
    await expect(review.getByRole("button", { name: /Use saved/ })).toBeInViewport();
    const overwrite = review.getByRole("button", { name: "Overwrite" });
    await expect(overwrite).toBeInViewport();
    const notePath = `/api/lists/${slug}/items/${first}/note`;
    const newerNote = "Saved while the conflict was open";
    const changed = await other.request.patch(notePath, {
      data: {
        note: { expected: savedText, value: newerNote },
      },
    });
    expect(changed.ok()).toBeTruthy();
    await overwrite.press("Enter");
    await expect(review.getByText(newerNote, { exact: true })).toBeVisible();
    expect((await (await page.request.get(notePath)).json()).note).toBe(newerNote);
    await overwrite.press("Enter");
    await expect(review).toBeHidden();
    await expect(
      left.region.getByRole("button", { name: "Edit", exact: true }),
    ).toBeFocused();
    expect(await reader.boundingBox()).toEqual(beforeReview);
    await expect(
      left.region.getByText("My draft").filter({ visible: true }),
    ).toBeVisible();
    const savedAfter = await page.request.get(notePath);
    expect((await savedAfter.json()).note).toBe(draftText);
  } finally {
    await other.close();
    await page.request.delete(`/api/lists/${key}`);
  }
});
