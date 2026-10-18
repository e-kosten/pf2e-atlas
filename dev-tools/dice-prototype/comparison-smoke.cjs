const { chromium, expect } = require('@playwright/test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
(async () => {
 const browser = await chromium.launch();
 try {
  const page = await browser.newPage({viewport:{width:1440,height:1000}});
  const errors=[]; page.on('pageerror',e=>errors.push(e.message));
  await page.goto('http://127.0.0.1:8820');
  await page.getByText('Animation comparison',{exact:true}).click();
  await page.getByText('Earlier tray experiments',{exact:true}).click();
  const roll = page.getByRole('button',{name:'Roll all',exact:true});
  await expect(roll).toBeEnabled({timeout:30000});
  for (const label of ['Damage · 2d6 + 4','Mixed dice · d8 + d6 + 4','Keep higher · 2d20 + 8','Keep lower · 2d20 + 8','Large pool · 8d6','Initiative · d20 + 5']) {
   await page.getByRole('combobox',{name:'Comparison dice'}).click();
   await page.getByText(label,{exact:true}).last().click();
   await roll.click();
   await expect(roll).toBeEnabled({timeout:35000});
   assert.equal(await page.locator('.comparison-grid [role=alert]').count(),0);
   console.log('PASS',label,await page.locator('.comparison-grid small').allTextContents());
  }
  fs.mkdirSync('evidence',{recursive:true});
  await page.screenshot({path:'evidence/comparison-desktop.png',fullPage:true});
  await page.setViewportSize({width:390,height:844});
  await roll.click();
  await expect(roll).toBeEnabled({timeout:35000});
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
  await page.screenshot({path:'evidence/comparison-phone.png',fullPage:true});
  await page.emulateMedia({reducedMotion:'reduce'});
  await expect(page.locator('.comparison-stage iframe')).toHaveCount(0);
  await roll.click(); await expect(roll).toBeEnabled({timeout:5000});
  assert.deepEqual(errors,[]);
  console.log('PASS phone, reduced motion, no page errors');
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
