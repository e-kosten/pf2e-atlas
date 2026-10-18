const { chromium } = require("playwright");
const assert = require("node:assert/strict");
const fs = require("node:fs");
(async () => {
  fs.mkdirSync("evidence", { recursive: true });
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({
      viewport: { width: 1440, height: 1000 },
    });
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.goto("http://127.0.0.1:8820");
    await page.getByText("Initiative", { exact: true }).click();
    await page
      .getByRole("button", { name: "Roll unrolled (4)", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Roll unrolled (0)", exact: true })
      .waitFor();
    assert.equal(await page.locator(".initiative-row .roll-total").count(), 4);
    assert.equal(await page.locator(".initiative-order span").count(), 4);
    await page.screenshot({
      path: "evidence/initiative-desktop.png",
      fullPage: true,
    });
    await page.getByText("Attacks & dice", { exact: true }).click();
    for (const name of [
      "Damage",
      "Mixed dice",
      "Keep higher",
      "Keep lower",
      "Eight dice",
    ]) {
      const card = page
        .locator(".dice-example")
        .filter({ has: page.getByRole("heading", { name, exact: true }) });
      await card.getByRole("button", { name: "Roll", exact: true }).click();
      await card.getByRole("button", { name: "Breakdown" }).first().waitFor();
      if (name === "Damage")
        assert.equal(await card.locator(".die").count(), 3);
      if (name === "Mixed dice")
        assert.equal(await card.locator(".die").count(), 2);
      if (name.startsWith("Keep"))
        assert.equal(await card.locator(".discarded").count(), 1);
      if (name === "Eight dice")
        assert.equal(await card.locator(".die").count(), 8);
    }
    await page.screenshot({
      path: "evidence/dice-desktop.png",
      fullPage: true,
    });
    await page.getByText("Roll table", { exact: true }).first().click();
    await page.getByRole("button", { name: "Roll table", exact: true }).click();
    await page.locator(".table-rows .matched").waitFor();
    assert.equal(await page.locator(".table-rows .matched").count(), 1);
    await page.setViewportSize({ width: 390, height: 844 });
    await page.getByText("Attacks & dice", { exact: true }).click();
    await page.screenshot({ path: "evidence/dice-phone.png", fullPage: true });
    assert.equal(
      await page.evaluate(
        () => document.documentElement.scrollWidth > innerWidth,
      ),
      false,
    );
    await page
      .getByRole("button", { name: "Preview settings", exact: true })
      .click();
    const pop = page.locator(".settings");
    await pop
      .locator("label")
      .filter({ hasText: "Simulate failure" })
      .getByRole("switch")
      .click();
    await page
      .getByRole("button", { name: "Preview settings", exact: true })
      .click();
    const attack = page
      .locator(".dice-example")
      .filter({
        has: page.getByRole("heading", { name: "Crossbow", exact: true }),
      });
    await attack.getByRole("button", { name: "Roll", exact: true }).click();
    await attack.getByRole("alert").waitFor();
    await attack.getByRole("button", { name: "Retry", exact: true }).click();
    await attack
      .getByRole("button", { name: "Breakdown", exact: true })
      .waitFor();
    assert.equal(await attack.getByRole("alert").count(), 0);
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.getByText("Initiative", { exact: true }).click();
    await page
      .getByRole("button", { name: "Roll Kobold Warrior 1", exact: true })
      .click();
    assert.equal(await page.locator(".rolling").count(), 0);
    assert.deepEqual(errors, []);
    console.log(
      "PASS: batch attribution, mixed dice, separate damage, keep/drop,8dice, table, phone width, failure/retry, reduced-motion; no browser errors",
    );
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
