import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";

test("a version change keeps one reachable recovery inside an open editor and preserves its draft", async ({
  page,
}, info) => {
  await page.goto("/lists");
  await activate(page.getByRole("button", { name: "New list", exact: true }), info);
  const dialog = page.getByRole("dialog", { name: "New List", exact: true });
  const name = dialog.getByRole("textbox", { name: /Name$/ });
  await name.fill("Retained version draft");
  let attempts = 0;
  await page.route("**/api/lists", async (route) => {
    if (route.request().method() !== "POST") return route.continue();
    attempts++;
    await route.fulfill({
      status: 409,
      headers: {
        "content-type": "application/json",
        "x-atlas-api-contract": "incompatible-test-contract",
      },
      body: JSON.stringify({ message: "SQL private/path decoder consequences" }),
    });
  });
  await activate(dialog.getByRole("button", { name: "Create", exact: true }), info);
  const reload = dialog.getByRole("button", { name: "Reload page", exact: true });
  await expect(reload).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Reload page", exact: true }),
  ).toHaveCount(1);
  await expect(name).toHaveValue("Retained version draft");
  await name.focus();
  await expect(name).toBeFocused();
  await expect(
    page.getByText(/SQL private|different versions|Could not load HP/),
  ).toHaveCount(0);
  // Trial activation verifies the actual pointer target without discarding the draft.
  await reload.click({ trial: true });
  await noPageOverflow(page);
  await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);
  await expect(dialog).toBeHidden();
  await expect(
    page.getByRole("button", { name: "Reload page", exact: true }),
  ).toBeVisible();
  expect(attempts).toBe(1);
});

test("a lost write response preserves the draft and reconnecting never replays the action", async ({
  page,
}, info) => {
  await page.goto("/lists");
  await activate(page.getByRole("button", { name: "New list", exact: true }), info);
  const dialog = page.getByRole("dialog", { name: "New List", exact: true });
  const name = dialog.getByRole("textbox", { name: /Name$/ });
  await name.fill("Unconfirmed draft");
  let attempts = 0;
  await page.route("**/api/lists", async (route) => {
    if (route.request().method() !== "POST") return route.continue();
    attempts++;
    await route.abort("connectionreset");
  });
  await activate(dialog.getByRole("button", { name: "Create", exact: true }), info);
  await expect(
    dialog.getByText("Change could not be confirmed", { exact: true }),
  ).toBeVisible();
  await expect(
    dialog.getByText("Unable to connect to Atlas", { exact: true }),
  ).toBeVisible();
  await expect(name).toHaveValue("Unconfirmed draft");
  await activate(dialog.getByRole("button", { name: "Retry", exact: true }), info);
  await expect(
    dialog.getByText("Unable to connect to Atlas", { exact: true }),
  ).toHaveCount(0);
  await expect(
    dialog.getByText("Change could not be confirmed", { exact: true }),
  ).toBeVisible();
  expect(attempts).toBe(1);
  await noPageOverflow(page);
});

test("blocked reference readiness has one notice without false model or empty-result states", async ({
  page,
}) => {
  await page.route("**/api/readiness", async (route) => {
    const response = await route.fetch();
    const data = await response.json();
    await route.fulfill({
      response,
      json: {
        ...data,
        status: "blocked",
        message: "SQL secret schema/path",
        search_profiles: data.search_profiles.map((profile: object) => ({
          ...profile,
          usable: false,
        })),
      },
    });
  });
  await page.goto("/search?q=signal");
  await expect(
    page.getByRole("status").filter({ hasText: "Reference unavailable." }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toHaveCount(1);
  await expect(
    page.getByText(/No matching records|SQL secret|unavailable for text search/),
  ).toHaveCount(0);
  await noPageOverflow(page);
});

test("a Similarity inventory outage preserves settings and recovers as a read", async ({
  page,
}, info) => {
  let offline = true;
  let attempts = 0;
  await page.route("**/api/search/similarity/parts", async (route) => {
    attempts++;
    if (offline) await route.abort("connectionreset");
    else await route.continue();
  });
  await page.goto("/search?similar=browser-rules:rule000000000001");
  // Adopt the initial result generation before opening a generation-bound inventory.
  await expect(
    page
      .getByRole("region", { name: "Search results", exact: true })
      .locator(".reference-result")
      .first(),
  ).toBeVisible();
  const settings = page.getByRole("button", { name: "Search settings", exact: true });
  async function openSettings() {
    await activate(settings, info);
    const dialog = page.getByRole("dialog", { name: "Search settings", exact: true });
    await expect(dialog).toBeVisible();
    // Touch can land during Ant's entrance prepare/start frames before settling.
    await expect(dialog).not.toHaveClass(/ant-zoom-(?:appear|enter)(?:[-\s]|$)/);
    return dialog;
  }
  let dialog = await openSettings();
  // The section preset is a mutable draft even while its inventory is unavailable.
  const sourceSections = dialog.getByRole("combobox", {
    name: "Source sections",
    exact: true,
  });
  await activate(sourceSections, info);
  await expect(sourceSections).toHaveAttribute("aria-expanded", "true");
  const dropdown = page.locator(".ant-select-dropdown:visible");
  await expect(dropdown).toBeVisible();
  await expect(dropdown).not.toHaveClass(/ant-slide-up-(?:appear|enter)(?:[-\s]|$)/);
  await activate(dropdown.getByText("Core sections", { exact: true }), info);
  await expect(sourceSections).toHaveAttribute("aria-expanded", "false");
  await expect(sourceSections.locator("..")).toHaveText("Core sections");
  await expect(
    dialog.getByRole("button", { name: "Choose sections", exact: true }),
  ).toBeEnabled();
  await expect(
    dialog.getByText("Unable to connect to Atlas", { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Retry", exact: true })).toHaveCount(1);
  await expect(
    dialog.getByRole("button", { name: "Apply", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByText(/change could not be confirmed|Check the current values/i),
  ).toHaveCount(0);
  expect(attempts).toBe(2); // The existing single automatic read retry stays bounded.
  await noPageOverflow(page);
  await info.attach("similarity-read-outage", {
    body: await page.screenshot(),
    contentType: "image/png",
  });
  offline = false;
  await activate(dialog.getByRole("button", { name: "Retry", exact: true }), info);
  await expect(
    dialog.getByText("Unable to connect to Atlas", { exact: true }),
  ).toHaveCount(0);
  await expect(
    dialog.getByRole("button", { name: "Apply", exact: true }),
  ).toBeEnabled();
  expect(attempts).toBe(3);
  await expect(sourceSections).toHaveAttribute("aria-expanded", "false");
  await expect(sourceSections.locator("..")).toHaveText("Core sections");
  await expect(dialog.getByText("Core sections", { exact: true })).toBeVisible();
  await activate(
    dialog.getByRole("button", { name: "Choose sections", exact: true }),
    info,
  );
  dialog = page.getByRole("dialog", { name: "Choose sections", exact: true });
  const identity = dialog.getByRole("checkbox", {
    name: "Signal Protocol: Name and identity",
    exact: true,
  });
  await expect(identity).toBeChecked();
  await activate(identity, info);
  await expect(identity).not.toBeChecked();
  await activate(
    dialog.getByRole("button", { name: "Back to search settings", exact: true }),
    info,
  );
  dialog = page.getByRole("dialog", { name: "Search settings", exact: true });
  await expect(dialog.getByText("Custom", { exact: true })).toBeVisible();
  await activate(
    dialog.getByRole("button", { name: "Choose sections", exact: true }),
    info,
  );
  dialog = page.getByRole("dialog", { name: "Choose sections", exact: true });
  await expect(identity).not.toBeChecked();
  await activate(dialog.getByRole("button", { name: "Apply", exact: true }), info);
  await expect(dialog).toBeHidden();
  const appliedUrl = page.url();
  dialog = await openSettings();
  await activate(
    dialog.getByRole("button", { name: "Choose sections", exact: true }),
    info,
  );
  dialog = page.getByRole("dialog", { name: "Choose sections", exact: true });
  await expect(identity).not.toBeChecked();
  await activate(identity, info);
  await activate(dialog.getByRole("button", { name: "Cancel", exact: true }), info);
  await expect(dialog).toBeHidden();
  expect(page.url()).toBe(appliedUrl);
  dialog = await openSettings();
  await activate(
    dialog.getByRole("button", { name: "Choose sections", exact: true }),
    info,
  );
  dialog = page.getByRole("dialog", { name: "Choose sections", exact: true });
  await expect(identity).not.toBeChecked();
  await noPageOverflow(page);
  await info.attach("similarity-read-recovered", {
    body: await page.screenshot(),
    contentType: "image/png",
  });
});
