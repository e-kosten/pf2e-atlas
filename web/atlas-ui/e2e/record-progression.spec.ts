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

test("keeps compact branches open through a drawer visit and deliberate refocus", async ({
  page,
}) => {
  await page.goto("/records/" + BATTLE_MEDICINE);
  await page.getByRole("button", { name: "Progression", exact: true }).press("Enter");
  const explore = page.getByRole("region", { name: "Explore record", exact: true });
  await expect(
    explore.getByRole("heading", { name: "Required by", exact: true }),
  ).toBeVisible();
  await explore
    .getByRole("button", { name: "Expand Godless Healing", exact: true })
    .click();
  await expect(
    explore.getByText("can't have a patron deity", { exact: true }),
  ).toBeVisible();
  await expect(
    explore.getByRole("link", { name: "Mortal Healing", exact: true }),
  ).toBeVisible();
  await expect(explore.getByText("Matching prerequisite", { exact: true })).toHaveCount(
    0,
  );

  const expand = explore.getByRole("button", {
    name: "Expand Medic Dedication",
    exact: true,
  });
  await expand.press("Enter");
  await expect(
    explore.getByRole("button", { name: "Collapse Medic Dedication", exact: true }),
  ).toBeFocused();
  await expect(explore.getByText("Trained in Medicine", { exact: true })).toHaveCount(
    1,
  );
  const doctorsVisitation = explore.locator(".record-progression__relation").filter({
    has: page.getByRole("link", { name: "Doctor's Visitation", exact: true }),
  });
  await expect(
    doctorsVisitation.getByRole("link", {
      name: "Explore progression for Doctor's Visitation",
    }),
  ).toBeVisible();
  await expect(
    doctorsVisitation.getByRole("button", { name: "Expand Doctor's Visitation" }),
  ).toHaveCount(0);
  await doctorsVisitation
    .getByRole("link", { name: "Doctor's Visitation", exact: true })
    .click();
  const doctorDrawer = page.getByRole("dialog", { name: "Doctor's Visitation" });
  await expect(doctorDrawer).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(doctorDrawer).toBeHidden();
  await expect(explore.getByText("Matching prerequisite", { exact: true })).toHaveCount(
    0,
  );
  await explore
    .getByRole("button", { name: "Expand Treat Condition", exact: true })
    .click();
  const treatCondition = explore
    .getByRole("listitem")
    .filter({
      has: page.getByRole("button", { name: "Collapse Treat Condition", exact: true }),
    })
    .last();
  await expect(
    treatCondition.getByRole("link", { name: "Preventative Treatment", exact: true }),
  ).toBeVisible();
  await expect(
    treatCondition.getByRole("button", {
      name: "Expand Preventative Treatment",
      exact: true,
    }),
  ).toHaveCount(0);
  await expect(
    explore.getByRole("link", { name: "Medic Dedication", exact: true }),
  ).toHaveCount(1);
  await explore
    .getByRole("button", { name: "Expand Holistic Care", exact: true })
    .click();
  await expect(
    explore.getByText("Trained in Diplomacy", { exact: true }),
  ).toBeVisible();
  await explore
    .getByRole("button", { name: "Expand Preventative Treatment", exact: true })
    .click();
  const directPreventative = explore
    .getByRole("listitem")
    .filter({
      has: page.getByRole("button", {
        name: "Collapse Preventative Treatment",
        exact: true,
      }),
    })
    .last();
  await expect(
    directPreventative.getByRole("link", { name: "Treat Condition", exact: true }),
  ).toBeVisible();

  const currentUrl = page.url();
  const medic = explore.getByRole("link", { name: "Medic Dedication", exact: true });
  await medic.press("Enter");
  const drawer = page.getByRole("dialog", { name: "Medic Dedication" });
  await expect(drawer).toBeVisible();
  await expect(page).toHaveURL(currentUrl);
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
  await expect(medic).toBeFocused();
  await expect(
    explore.getByText("can't have a patron deity", { exact: true }),
  ).toBeVisible();

  await explore
    .getByRole("link", {
      name: "Explore progression for Medic Dedication",
      exact: true,
    })
    .click();
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
  await expect(
    explore.getByText("can't have a patron deity", { exact: true }),
  ).toBeVisible();
  await treatCondition
    .getByRole("link", {
      name: "Explore progression for Preventative Treatment",
      exact: true,
    })
    .click();
  await expect(
    explore.getByRole("heading", { name: "Prerequisites", exact: true }),
  ).toBeVisible();
  await expect(
    explore.getByRole("link", { name: "Medic Dedication", exact: true }),
  ).toBeVisible();
  await expect(
    explore.getByRole("link", { name: "Treat Condition", exact: true }),
  ).toBeVisible();
});

test("keeps a deep branch aligned with the full progression width", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/records/" + BATTLE_MEDICINE);
  await page.getByRole("button", { name: "Progression", exact: true }).click();
  const explore = page.getByRole("region", { name: "Explore record", exact: true });
  for (const name of [
    "Mage's Field Dressing",
    "War Mage Dedication",
    "Arcana of Iron",
  ]) {
    await explore.getByRole("button", { name: "Expand " + name, exact: true }).click();
  }
  await expect(
    explore.getByRole("link", { name: "Secrets of Steel", exact: true }),
  ).toBeVisible();
  await expect(
    explore.getByRole("button", { name: "Expand Secrets of Steel", exact: true }),
  ).toHaveCount(0);
  const row = (name: string) =>
    explore.locator(".record-progression__relation").filter({
      has: page.getByRole("link", { name, exact: true }),
    });
  const topRight = await row("Mage's Field Dressing")
    .first()
    .evaluate((item) => item.getBoundingClientRect().right);
  const deepRight = await row("Secrets of Steel").evaluate(
    (item) => item.getBoundingClientRect().right,
  );
  expect(Math.abs(topRight - deepRight)).toBeLessThanOrEqual(2);
});

test("shows both alternatives and identifies the matching reverse branch", async ({
  page,
}) => {
  await page.goto("/records/" + SPEAKERS_DEFENSE);
  await page.getByRole("button", { name: "Progression", exact: true }).click();
  const explore = page.getByRole("region", { name: "Explore record", exact: true });
  const heritage = explore.getByRole("link", { name: /^budding speaker centaur/i });
  const training = explore.getByRole("link", {
    name: "Speaker in Training",
    exact: true,
  });
  await expect(heritage).toHaveAttribute("href", recordHref(BUDDING_SPEAKER));
  await expect(training).toHaveAttribute("href", recordHref(SPEAKER_TRAINING));

  await explore
    .getByRole("button", { name: /expand budding speaker centaur/i })
    .click();
  await explore
    .getByRole("link", { name: /explore progression for budding speaker centaur/i })
    .click();
  await expect(
    explore.getByRole("heading", { name: "Required by", exact: true }),
  ).toBeVisible();
  const defense = explore.getByRole("link", { name: "Speaker's Defense", exact: true });
  await expect(defense).toBeVisible();
  await expect(
    explore.getByText("Alternative prerequisite", { exact: true }).first(),
  ).toBeVisible();
  await explore
    .getByRole("button", { name: "Expand Speaker's Defense", exact: true })
    .click();
  await expect(explore.getByText("Any of", { exact: true })).toBeVisible();
  await expect(training).toBeVisible();
  await page.goBack();
  await expect(training).toBeVisible();
  await explore
    .getByRole("button", { name: "Expand Speaker in Training", exact: true })
    .click();
  await explore
    .getByRole("link", {
      name: "Explore progression for Speaker in Training",
      exact: true,
    })
    .click();
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
    const originUrl = page.url();
    const drawerScroll = await page.evaluate(() => Math.round(window.scrollY));
    expect(drawerScroll).toBeGreaterThan(0);
    await feat.tap();
    const drawer = page.getByRole("dialog", { name: "Elven Lore" });
    await expect(drawer).toBeVisible();
    await expect(page).toHaveURL(originUrl);
    await drawer.getByRole("button", { name: "Close reference preview" }).tap();
    await expect(drawer).toBeHidden();
    await expect(feat).toBeFocused();
    await expect
      .poll(() =>
        page.evaluate((scroll) => Math.abs(window.scrollY - scroll), drawerScroll),
      )
      .toBeLessThanOrEqual(2);

    await explore.getByRole("button", { name: "Expand Elven Lore", exact: true }).tap();
    const refocus = explore.getByRole("link", {
      name: "Explore progression for Elven Lore",
      exact: true,
    });
    await refocus.scrollIntoViewIfNeeded();
    const before = await page.evaluate(() => Math.round(window.scrollY));
    await refocus.tap();
    await expect(
      explore.getByRole("heading", { name: "Available through", exact: true }),
    ).toBeVisible();
    const source = explore.getByRole("link", { name: "Aiuvarin", exact: true });
    await expect(source).toHaveAttribute("href", recordHref(AIUVARIN));

    await page.goBack();
    await expect(feat).toBeVisible();
    await expect(refocus).toBeVisible();
    await expect
      .poll(async () =>
        Math.abs((await page.evaluate(() => Math.round(window.scrollY))) - before),
      )
      .toBeLessThanOrEqual(2);
  } finally {
    await context.close();
  }
});
