import { expect, test } from "@playwright/test";

// Exact RecordKeys from the managed PF2e source pin.
const BATTLE_MEDICINE = "feats-srd:wYerMk6F1RZb0Fwt";
const MEDIC = "feats-srd:MJg24e9fJd7OASvF";
const SPEAKERS_DEFENSE = "feats-srd:J7j6sSIdkwMwoZTx";
const BUDDING_SPEAKER = "heritages:1FmcZqnmDm8H0DRB";
const SPEAKER_TRAINING = "feats-srd:gVCP5lZEWviHMGdV";
const AIUVARIN = "heritages:N36ZR4lh9eCazDaN";
const ELVEN_LORE = "feats-srd:zki5qdM5IQcAiscM";

function recordHref(key: string) {
  return new RegExp(key.replace(":", "(?::|%3A)"));
}

test("follows a prerequisite in both directions and returns to the prior focus record", async ({
  page,
}) => {
  await page.goto("/records/" + BATTLE_MEDICINE);
  await page.getByRole("button", { name: "Progression", exact: true }).press("Enter");
  const explore = page.getByRole("region", { name: "Explore record", exact: true });
  await expect(
    explore.getByRole("heading", { name: "Required by", exact: true }),
  ).toBeVisible();

  const expand = explore.getByRole("button", {
    name: "Expand Medic Dedication",
    exact: true,
  });
  await expand.press("Enter");
  await expect(
    explore.getByRole("button", { name: "Collapse Medic Dedication", exact: true }),
  ).toBeFocused();
  await expect(explore.getByText(/trained in medicine/i).first()).toBeVisible();

  await explore.getByRole("link", { name: "Medic Dedication", exact: true }).click();
  await expect(page).toHaveURL(
    (url) =>
      decodeURIComponent(url.pathname).endsWith(MEDIC) &&
      url.searchParams.get("record_tab") === "progression",
  );
  await expect(
    explore.getByRole("link", { name: "Battle Medicine", exact: true }).first(),
  ).toBeVisible();
  await page.goBack();
  await expect(page).toHaveURL(
    (url) =>
      decodeURIComponent(url.pathname).endsWith(BATTLE_MEDICINE) &&
      url.searchParams.get("record_tab") === "progression",
  );
  await expect(
    explore.getByRole("link", { name: "Medic Dedication", exact: true }),
  ).toBeVisible();
});

test("shows both alternatives and identifies the matching reverse branch", async ({
  page,
}) => {
  await page.goto("/records/" + SPEAKERS_DEFENSE);
  await page.getByRole("button", { name: "Progression", exact: true }).click();
  const explore = page.getByRole("region", { name: "Explore record", exact: true });
  const heritage = explore.getByRole("link", { name: /budding speaker centaur/i });
  const training = explore.getByRole("link", {
    name: "Speaker in Training",
    exact: true,
  });
  await expect(heritage).toHaveAttribute("href", recordHref(BUDDING_SPEAKER));
  await expect(training).toHaveAttribute("href", recordHref(SPEAKER_TRAINING));

  await heritage.click();
  await expect(
    explore.getByRole("heading", { name: "Required by", exact: true }),
  ).toBeVisible();
  const defense = explore.getByRole("link", { name: "Speaker's Defense", exact: true });
  await expect(defense).toBeVisible();
  await expect(
    explore.getByText("Alternative prerequisite", { exact: true }).first(),
  ).toBeVisible();
  await page.goBack();
  await expect(training).toBeVisible();
  await training.click();
  await expect(defense).toBeVisible();
  await expect(
    explore.getByText("Alternative prerequisite", { exact: true }).first(),
  ).toBeVisible();
});

test("explores Aiuvarin’s elf feat access on a narrow touch viewport", async ({
  browser,
  baseURL,
}) => {
  const context = await browser.newContext({
    baseURL,
    viewport: { width: 390, height: 500 },
    hasTouch: true,
  });
  try {
    const page = await context.newPage();
    await page.goto("/records/" + AIUVARIN);
    await page.getByRole("button", { name: "Progression", exact: true }).tap();
    const explore = page.getByRole("region", { name: "Explore record", exact: true });
    await expect(
      explore.getByRole("heading", { name: "Ancestry feat access", exact: true }),
    ).toBeVisible();
    await explore.getByText("Elf", { exact: true }).tap();
    const feat = explore.getByRole("link", { name: "Elven Lore", exact: true });
    await expect(feat).toHaveAttribute("href", recordHref(ELVEN_LORE));
    await feat.scrollIntoViewIfNeeded();
    const before = await page.evaluate(() => Math.round(window.scrollY));
    expect(before).toBeGreaterThan(0);
    await feat.tap();
    await expect(
      explore.getByRole("heading", { name: "Available through", exact: true }),
    ).toBeVisible();
    const source = explore.getByRole("link", { name: "Aiuvarin", exact: true });
    await expect(source).toHaveAttribute("href", recordHref(AIUVARIN));

    await page.goBack();
    await expect(feat).toBeVisible();
    await expect
      .poll(async () =>
        Math.abs((await page.evaluate(() => Math.round(window.scrollY))) - before),
      )
      .toBeLessThanOrEqual(2);
  } finally {
    await context.close();
  }
});
