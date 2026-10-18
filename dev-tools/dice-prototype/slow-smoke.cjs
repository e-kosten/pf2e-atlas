const { chromium } = require("playwright");
const assert = require("node:assert/strict");
(async () => {
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage({
      viewport: { width: 1100, height: 900 },
    });
    await page.goto("http://127.0.0.1:8820");
    await page.getByText("Initiative", { exact: true }).click();
    const ogre = page
      .locator(".initiative-row")
      .filter({ hasText: "Ogre Warrior" });
    await ogre.getByRole("combobox").click();
    await page.getByTitle("Stealth", { exact: true }).click();
    assert.equal(await ogre.getByRole("spinbutton").inputValue(), "-1");
    await page
      .getByRole("button", { name: "Preview settings", exact: true })
      .click();
    await page.locator(".settings").getByRole("combobox").click();
    await page.getByTitle("Slow (2.5 seconds)", { exact: true }).click();
    await page
      .getByRole("button", { name: "Preview settings", exact: true })
      .click();
    const response = page.waitForResponse((r) => r.url().endsWith("/roll"));
    await ogre.getByRole("button", { name: "Roll Ogre Warrior" }).click();
    await ogre.getByText("Still rolling…").waitFor();
    assert.equal(await ogre.locator(".rolling").count(), 1);
    await page.screenshot({ path: "evidence/slow-roll.png" });
    const data = await (await response).json();
    assert.equal(data.groups[0].formula, "1d20-1");
    await ogre.getByRole("button", { name: "Breakdown" }).waitFor();
    assert.equal(await ogre.locator(".rolling").count(), 0);
    console.log(
      "PASS: unlisted Stealth -1, slow response retains animation and resolves actual result",
    );
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
