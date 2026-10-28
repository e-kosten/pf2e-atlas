import { expect, test } from "./fixtures";

test("startup mismatch reloads the same URL once and recovers from a matching backend", async ({
  page,
}) => {
  let handshakes = 0;
  await page.route("**/api/compatibility", async (route) => {
    handshakes++;
    if (handshakes === 1) {
      await route.fulfill({
        status: 409,
        headers: { "x-atlas-api-contract": "outdated" },
        json: {},
      });
    } else await route.continue();
  });
  await page.goto("/encounters");
  await expect(
    page.getByRole("heading", { name: "Encounters", exact: true }),
  ).toBeVisible();
  expect(handshakes).toBe(2);
  await expect(page).toHaveURL(/\/encounters$/);
});

test("persistent mismatch stops after one automatic reload without feature requests", async ({
  page,
}) => {
  const apiRequests: string[] = [];
  page.on("request", (request) => {
    if (new URL(request.url()).pathname.startsWith("/api/"))
      apiRequests.push(new URL(request.url()).pathname);
  });
  await page.route("**/api/compatibility", (route) =>
    route.fulfill({
      status: 409,
      headers: { "x-atlas-api-contract": "outdated" },
      json: {},
    }),
  );
  await page.goto("/encounters");
  await expect.poll(() => apiRequests.length).toBe(2);
  await expect(page.getByRole("alert")).toContainText("Atlas versions don’t match");
  expect(apiRequests).toEqual(["/api/compatibility", "/api/compatibility"]);
  await page.getByRole("button", { name: "Reload page" }).click();
  await expect.poll(() => apiRequests.length).toBe(3);
  await expect(page.getByRole("alert")).toContainText("Atlas versions don’t match");
});

test("a mismatch during use preserves an unsaved form and never replays its mutation", async ({
  page,
}) => {
  await page.goto("/encounters");
  await page.getByRole("button", { name: "New Encounter", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "New encounter", exact: true });
  await dialog.getByRole("textbox", { name: "Name" }).fill("Unsaved draft");
  let submissions = 0;
  await page.route("**/api/encounters", async (route) => {
    if (route.request().method() === "POST") {
      submissions++;
      await route.fulfill({
        status: 409,
        headers: { "x-atlas-api-contract": "updated" },
        json: {},
      });
    } else await route.continue();
  });
  await dialog.getByRole("button", { name: "OK", exact: true }).click();
  await expect(
    dialog.getByRole("status").filter({ hasText: "Reload Atlas to continue" }),
  ).toBeVisible();
  await expect(dialog.getByRole("textbox", { name: "Name" })).toHaveValue(
    "Unsaved draft",
  );
  expect(submissions).toBe(1);
  await expect(page).toHaveURL(/\/encounters$/);
});
