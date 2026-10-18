const {chromium,expect}=require('@playwright/test');
const assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch();try{
const p=await browser.newPage({viewport:{width:1440,height:1000}});const errors=[];p.on('pageerror',e=>errors.push(e.message));
await p.goto('http://127.0.0.1:8820');
await expect(p.getByText('All dice types · d4 to d20',{exact:true})).toBeVisible();
await expect(p.locator('.inline-grid .dice-example')).toHaveCount(6);
await expect(p.locator('.inline-grid h3')).toHaveText(['Spin','Tumble','Rock','Spin','Tumble','Rock']);
const roll=p.getByRole('button',{name:'Roll all',exact:true});
for(const label of ['Fortune · keep higher','Misfortune · keep lower','Mixed dice · d8 + d6 + 4','Large pool · 8d6','All dice types · d4 to d20',... [4,6,8,10,12,20].map(s=>`Single d${s}`)]){
await p.getByRole('combobox',{name:'Comparison dice'}).click();await p.getByText(label,{exact:true}).last().click();
await roll.click();await expect(roll).toBeEnabled({timeout:10000});
await expect(p.locator('.inline-grid [role=alert]')).toHaveCount(0);
if(label.startsWith('Fortune')||label.startsWith('Misfortune')) await expect(p.locator('.inline-grid .discarded')).toHaveCount(6);
if(label.startsWith('All dice')) {
  await expect(p.locator('.inline-grid .die')).toHaveCount(36);
  for (const card of await p.locator('.inline-grid .dice-example').all()) {
    await expect(card.locator('.die-caption')).toHaveText(['d4','d6','d8','d10','d12','d20']);
  }
  await p.screenshot({path:'evidence/all-dice-types.png',fullPage:true});
}
if(label.startsWith('Mixed'))await expect(p.locator('.inline-grid .die')).toHaveCount(12);
console.log('PASS',label);
if(label.startsWith('Fortune'))await p.screenshot({path:'evidence/inline-fortune.png',fullPage:true});
}
await p.setViewportSize({width:390,height:844});await roll.click();await expect(roll).toBeEnabled();
assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
await p.screenshot({path:'evidence/inline-phone.png',fullPage:true});
await p.emulateMedia({reducedMotion:'reduce'});await expect(p.locator('.inline-grid canvas')).toHaveCount(0);await roll.click();await expect(roll).toBeEnabled();
await expect(p.locator('.inline-grid .rolling')).toHaveCount(0);
assert.deepEqual(errors,[]); console.log('PASS phone, reduced motion, no page errors');
}finally{await browser.close()}})().catch(e=>{console.error(e);process.exitCode=1});
