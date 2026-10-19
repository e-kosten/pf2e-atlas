const { chromium, expect } = require('@playwright/test');
const assert = require('node:assert/strict');
(async () => {
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage();
    await page.route('**/roll', async route => {
      const response = await route.fetch();
      await new Promise(resolve => setTimeout(resolve, 1300));
      await route.fulfill({response});
    });
    await page.goto('http://127.0.0.1:8820');
    await page.getByText('Animation comparison', {exact:true}).click();
    await page.getByRole('button', {name:'Roll all',exact:true}).click();
    const faces = page.locator('[data-engine="plain-spin"] .die-face');
    await expect.poll(() => faces.first().evaluate(el => el.getAnimations().some(a => a.playState === 'running'))).toBe(true);
    await page.waitForTimeout(750);
    const resting = await faces.evaluateAll(els => els.map(el => ({
      transform:getComputedStyle(el).transform,
      active:el.getAnimations().some(a => a.playState === 'running'),
      bounds:el.getBoundingClientRect().toJSON()
    })));
    assert(resting.every(x => x.transform === 'none' && !x.active));
    await expect(page.getByRole('button',{name:'Roll all',exact:true})).toBeEnabled();
    const final = await faces.evaluateAll(els => els.map(el => el.getBoundingClientRect().toJSON()));
    assert.deepEqual(final,resting.map(x => x.bounds));
    console.log('PASS spinning dice settle upright before delayed result; no position change on reveal');
  } finally { await browser.close(); }
})().catch(e => {console.error(e);process.exitCode=1});
