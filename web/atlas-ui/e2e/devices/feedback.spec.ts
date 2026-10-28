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
