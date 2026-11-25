import { join } from "node:path";

// Exercise the installed host and real WebView2 documents after the signed
// update. The caller owns the disposable service, installer and workspace.
export async function qualifyWindowsTabs({ page, expect, openLink, listKey, stopWorker, nativePage, evidence }) {
  const characters = page.getByRole("link", { name: "Characters", exact: true });
  const parties = page.getByRole("link", { name: "Parties", exact: true });
  const tab = (name) => page.getByRole("tab", { name, exact: true });
  const heading = (name) => page.getByRole("heading", { name, exact: true });
  const recordName = "Windows updater fixture";
  const listName = "Windows handoff saved work";
  const recordLink = "pf2e-atlas://v1/record/actions%3Aaaaaaaaaaaaaaaaa";
  const listLink = "pf2e-atlas://v1/list/" + encodeURIComponent(listKey);

  await characters.click();
  await parties.click({ modifiers: ["Control"] });
  await expect(tab("Parties")).toHaveAttribute("aria-selected", "false");
  await parties.click();
  await expect(heading("Parties")).toBeVisible();
  await characters.click();
  await expect(heading("Characters")).toBeVisible();
  await page.keyboard.press("Alt+ArrowLeft");
  await expect(heading("Parties")).toBeVisible();
  await page.getByRole("button", { name: "New party" }).click();
  const draft = page.getByRole("textbox", { name: "Party name" });
  await draft.fill("Keep this party draft");
  await page.keyboard.press("Alt+ArrowRight");
  const discard = page.getByRole("dialog", { name: "Discard new party?" });
  await expect(discard).toBeVisible();
  await discard.getByRole("button", { name: "Keep editing" }).click();
  await expect(draft).toHaveValue("Keep this party draft");
  await page.keyboard.press("Alt+ArrowLeft");
  await discard.getByRole("button", { name: "Discard", exact: true }).click();
  await expect(heading("Characters")).toBeVisible();

  await page.keyboard.press("Alt+ArrowRight");
  await expect(heading("Parties")).toBeVisible();
  await page.getByRole("button", { name: "New party" }).click();
  await draft.fill("Pending navigation fixture");
  let resume;
  const write = new Promise((resolve) => { resume = resolve; });
  const partyApi = "**/api/parties";
  const holdWrite = async (route) => {
    if (route.request().method() === "POST") await write;
    await route.continue();
  };
  await page.route(partyApi, holdWrite);
  try {
    const started = page.waitForRequest((request) =>
      new URL(request.url()).pathname === "/api/parties" && request.method() === "POST");
    await page.getByRole("button", { name: "Create party", exact: true }).click();
    await started;
    await page.keyboard.press("Alt+ArrowLeft");
    await expect(draft).toBeVisible();
    await expect(discard).toHaveCount(0);
    resume();
    await expect(heading("Characters")).toBeVisible();
  } finally {
    resume();
    await page.unroute(partyApi, holdWrite);
  }

  openLink(recordLink);
  await expect(heading(recordName)).toBeVisible();
  openLink("pf2e-atlas://v1/record/actions%3Abbbbbbbbbbbbbbbb");
  await expect(heading("Second Windows fixture")).toBeVisible();
  await expect(tab("Second Windows fixture")).toHaveCount(1);
  openLink(listLink);
  await expect(heading(listName)).toBeVisible();
  await expect(tab(recordName)).toHaveCount(1);
  await expect(tab(listName)).toHaveCount(1);
  openLink(recordLink);
  await expect(tab(recordName)).toHaveAttribute("aria-selected", "true");
  await expect(tab(recordName)).toHaveCount(1);
  await page.getByText("Disposable signed Windows updater qualification.", { exact: true })
    .click({ clickCount: 3, position: { x: 4, y: 8 } });
  const actions = page.getByRole("group", { name: "Search selected text" });
  await expect(actions).toHaveCount(1);
  await page.screenshot({ path: join(evidence, "tab-selection-search.png") });
  await actions.getByRole("link", { name: "Search Atlas" }).click();
  await expect(page.getByRole("searchbox", { name: "Search records" })).toHaveValue(/\S/);

  await tab("Characters").click();
  await tab("Characters").focus();
  await page.keyboard.press("Control+Tab");
  await expect(tab("Parties")).toBeFocused();
  await page.keyboard.press("Control+Shift+Tab");
  await expect(tab("Characters")).toBeFocused();
  await page.keyboard.press("Control+Tab");
  await expect(tab("Parties")).toBeFocused();
  await page.keyboard.press("Delete");
  await expect(tab("Parties")).toHaveCount(0);
  await expect(tab("Characters")).toHaveCount(1);

  // Hold one real delivery before React acknowledges it, then interrupt its
  // service. A late acknowledgement from this document cannot own the queue.
  await page.evaluate(() => {
    const hold = (event) => {
      if (event.detail.delivery === undefined) return;
      window.removeEventListener("atlas-windows-tab-command", hold, true);
      event.stopImmediatePropagation();
      document.documentElement.dataset.heldDelivery = String(event.detail.delivery);
    };
    window.addEventListener("atlas-windows-tab-command", hold, true);
  });
  openLink(recordLink);
  await expect(page.locator("html")).toHaveAttribute("data-held-delivery", /\d+/);
  stopWorker();
  const recovery = await nativePage((p) => p.getByRole("button", { name: "Reconnect", exact: true }).isVisible());
  openLink(listLink);
  await recovery.getByRole("button", { name: "Reconnect", exact: true }).click();
  const fresh = await nativePage((p) =>
    p !== page && p.getByRole("heading", { name: listName, exact: true }).isVisible());
  await expect(fresh.getByRole("tab")).toHaveCount(2);
  await expect(fresh.getByRole("tab", { name: recordName, exact: true })).toHaveCount(1);
  await expect(fresh.getByRole("tab", { name: listName, exact: true })).toHaveAttribute("aria-selected", "true");
  await fresh.screenshot({ path: join(evidence, "reconnected-tabs.png") });
}
