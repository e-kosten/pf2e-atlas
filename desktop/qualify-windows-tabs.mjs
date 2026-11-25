import { join } from "node:path";

// The host dispatches directly on Window. WebView2 can invoke the product's
// existing listener before a later capture listener on that same target.
export async function holdNextTabCommand(page, match) {
  await page.evaluate((match) => {
    const kinds = match === "check-exit" ? ["check-exit"] : ["open", "select"];
    const field = match === "check-exit" ? "id" : "delivery";
    const dispatch = window.dispatchEvent;
    const previous = Object.getOwnPropertyDescriptor(window, "dispatchEvent");
    delete document.documentElement.dataset.heldTabCommand;
    window.dispatchEvent = function (event) {
      if (event.type === "atlas-windows-tab-command" && kinds.includes(event.detail?.kind)
        && Number.isSafeInteger(event.detail[field])) {
        if (previous) Object.defineProperty(window, "dispatchEvent", previous);
        else delete window.dispatchEvent;
        document.documentElement.dataset.heldTabCommand = String(event.detail[field]);
        return true;
      }
      return dispatch.call(this, event);
    };
  }, match);
}

// Exercise the installed host and real WebView2 documents after the signed
// update. The caller owns the disposable service, installer and workspace.
export async function qualifyWindowsTabs({ page, expect, openLink, listKey, stopWorker, nativePage, evidence, exit, windows, answer, log }) {
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

  const originalWindows = windows().filter((w) => w.menu).map((w) => w.id);
  await page.keyboard.press("Control+n");
  const secondary = await nativePage(async (candidate) => candidate !== page &&
    await candidate.getByRole("searchbox", { name: "Search records" }).isVisible());
  await secondary.getByRole("link", { name: "Characters", exact: true }).click();
  await secondary.getByRole("link", { name: "Parties", exact: true }).click();
  await secondary.getByRole("button", { name: "New party" }).click();
  const staleDraft = secondary.getByRole("textbox", { name: "Party name" });
  await staleDraft.fill("Keep this disconnected party");
  const secondaryWindow = windows().find((w) => w.menu && !originalWindows.includes(w.id))?.id;
  if (!secondaryWindow) throw new Error("Missing secondary window handle");
  exit();
  await answer("Exit Atlas", 7);
  await expect(staleDraft).toHaveValue("Keep this disconnected party");
  // Reactivate the original window so its held delivery is the one interrupted.
  openLink(recordLink);
  await expect(tab(recordName)).toHaveAttribute("aria-selected", "true");

  // Hold one real delivery before React acknowledges it, then interrupt its
  // service. A late acknowledgement from this document cannot own the queue.
  await holdNextTabCommand(page, "delivery");
  openLink(recordLink);
  await expect(page.locator("html")).toHaveAttribute("data-held-tab-command", /^\d+$/);
  stopWorker();
  const recovery = await nativePage((p) => p.getByRole("button", { name: "Reconnect", exact: true }).isVisible());
  windows(secondaryWindow);
  await answer("Close Atlas window", 7);
  exit();
  await answer("Exit Atlas", 7);
  await expect(staleDraft).toHaveValue("Keep this disconnected party");
  openLink(listLink);
  // Wait for the cancelled Exit handoff to re-enable the stale editor. Its
  // pending record and the subsequent list must both reach the new service.
  await expect.poll(() => windows().find((w) => w.id === secondaryWindow)?.enabled).toBe(true);
  const beforeReconnect = windows();
  log("reconnect-requested", { windows: beforeReconnect, secondaryWindow, oldPage: page.url(),
    recoveryPage: recovery.url(), heldDelivery: await page.locator("html").getAttribute("data-held-tab-command") });
  expect(beforeReconnect.map((w) => w.title)).not.toContain("Could not open Atlas link");
  await recovery.getByRole("button", { name: "Reconnect", exact: true }).click();
  const fresh = await nativePage((p) =>
    p !== page && p.getByRole("heading", { name: listName, exact: true }).isVisible());
  await expect(fresh.getByRole("tab")).toHaveCount(2);
  await expect(fresh.getByRole("tab", { name: recordName, exact: true })).toHaveCount(1);
  await expect(fresh.getByRole("tab", { name: listName, exact: true })).toHaveAttribute("aria-selected", "true");
  log("reconnected-links-delivered", { page: fresh.url(), windows: windows() });
  await fresh.screenshot({ path: join(evidence, "reconnected-tabs.png") });

  // Exit checks the live window for pending saves even with a stale draft.
  await fresh.getByRole("link", { name: "Characters", exact: true }).click();
  await fresh.getByRole("link", { name: "Parties", exact: true }).click();
  await fresh.getByRole("button", { name: "New party" }).click();
  await fresh.getByRole("textbox", { name: "Party name" }).fill("Connected party");
  let release;
  const hold = new Promise((done) => { release = done; });
  await fresh.route("**/api/parties", async (route) => {
    if (route.request().method() === "POST") await hold;
    await route.continue();
  });
  try {
    const post = fresh.waitForRequest((req) => req.method() === "POST" && req.url().endsWith("/api/parties"));
    await fresh.getByRole("button", { name: "Create party" }).click();
    await post;
    exit();
    await answer("Could not exit Atlas", 1);
  } finally {
    release();
  }
  await expect(fresh.getByRole("heading", { name: "Connected party" })).toBeVisible();
  exit();
  await answer("Exit Atlas", 7);
  await expect(staleDraft).toHaveValue("Keep this disconnected party");
  windows(secondaryWindow);
  await answer("Close Atlas window", 6);
  await expect.poll(() => secondary.isClosed()).toBe(true);
  await expect(fresh.getByRole("heading", { name: "Connected party" })).toBeVisible();
  // The original disconnected workspace also survives reconnection.
  exit();
  await answer("Exit Atlas", 6);
  await expect.poll(() => fresh.isClosed()).toBe(true);
}
