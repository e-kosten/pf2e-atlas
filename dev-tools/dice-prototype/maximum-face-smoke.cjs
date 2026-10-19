const {chromium,expect}=require('@playwright/test');
(async()=>{
  const browser=await chromium.launch();
  try {
    const page=await browser.newPage({viewport:{width:1440,height:1000}});
    // Deliberately exercise two-digit faces rather than waiting for random rolls.
    await page.route('**/roll',async route=>{
      const request=route.request().postDataJSON();
      const groups=request.groups.map(g=>{
        const sides=Number(g.formula.slice(2));
        return {...g,total:sides,dice:[{sides,value:sides,kept:true}],breakdown:String(sides)};
      });
      await route.fulfill({json:{id:request.id,groups,total:groups.reduce((n,g)=>n+g.total,0)}});
    });
    await page.goto('http://127.0.0.1:8820');
    const roll=page.getByRole('button',{name:'Roll all',exact:true});
    await roll.click();await expect(roll).toBeEnabled();
    await expect(page.locator('[data-engine="plain-spin"] .die-face > span')).toHaveText(['4','6','8','10','12','20']);
    await page.screenshot({path:'evidence/maximum-faces.png',fullPage:true});
    console.log('PASS maximum-face preview includes 10, 12 and 20');
  } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
