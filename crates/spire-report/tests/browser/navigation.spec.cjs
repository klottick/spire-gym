const {test,expect}=require('@playwright/test');
const f=require('./fixtures.cjs');
test.beforeEach(async({page})=>{
  page.reportErrors=[];
  page.on('pageerror',error=>page.reportErrors.push(String(error)));
  page.on('console',message=>{if(message.type()==='error')page.reportErrors.push(message.text());});
  page.on('request',request=>{if(/^https?:/.test(request.url()))page.reportErrors.push(request.url());});
});
test.afterEach(async({page})=>expect(page.reportErrors).toEqual([]));
test.afterAll(f.cleanup);

test('navigation_boundaries',async({page})=>{
  await page.goto(f.exported());
  const previous=page.getByRole('button',{name:'Previous recorded frame',exact:true});
  const next=page.getByRole('button',{name:'Next recorded frame',exact:true});
  await expect(previous).toBeDisabled();
  await expect(page.locator('[data-frame-id]')).toHaveAttribute('data-frame-id','0');
  await next.click();await expect(page.locator('[data-frame-id]')).toHaveAttribute('data-frame-id','1');
  await previous.focus();await page.keyboard.press('ArrowRight');
  await expect(page.locator('[data-frame-id]')).toHaveAttribute('data-frame-id','2');
  await expect(next).toBeDisabled();await expect(previous).toBeFocused();
  await page.getByRole('slider',{name:'Recorded slide',exact:true}).evaluate(node=>{node.value='0';node.dispatchEvent(new Event('input',{bubbles:true}));});
  await expect(previous).toBeDisabled();
  await page.getByRole('combobox',{name:'Jump to recorded context',exact:true}).selectOption('2');
  await expect(page.locator('[data-frame-id]')).toHaveAttribute('data-frame-id','2');
});

test('navigation_context',async({page})=>{
  const frames=Array.from({length:160},(_,index)=>f.frame(index,{floor:index+1,combatId:`combat-${index}`,turn:1}));
  await page.goto(f.exported(f.document({frames})));
  const jump=page.getByRole('combobox',{name:'Jump to recorded context',exact:true});
  await expect(jump.locator('option').nth(0)).toContainText(/Act 1.*Floor 1.*combat-0.*Turn 1/);
  await expect(jump.locator('option').nth(1)).toContainText(/Floor 2.*combat-1.*Turn 1/);
  const outline=page.getByRole('navigation',{name:'Recorded timeline'});
  expect((await outline.boundingBox()).height).toBeLessThan(650);
  expect((await page.locator('[data-frame-id]').boundingBox()).y).toBeLessThan(350);
});

test('navigation_focus',async({page})=>{
  await page.goto(f.exported());
  await page.evaluate(()=>{
    const input=document.createElement('input');input.setAttribute('aria-label','Fixture input');document.body.append(input);
    const edit=document.createElement('div');edit.contentEditable='true';edit.textContent='Editable';edit.setAttribute('role','textbox');edit.setAttribute('aria-label','Fixture editable');document.body.append(edit);
  });
  for(const locator of [page.getByLabel('Fixture input'),page.getByLabel('Fixture editable')]){
    await locator.focus();await page.keyboard.press('ArrowRight');
    await expect(page.locator('[data-frame-id]')).toHaveAttribute('data-frame-id','0');await expect(locator).toBeFocused();
  }
  const jump=page.getByRole('combobox',{name:'Jump to recorded context'});
  await jump.focus();await page.keyboard.press('ArrowRight');
  await expect(page.locator('[data-frame-id]')).toHaveAttribute('data-frame-id','1');
  await expect(jump).toBeFocused(); // One native menu change, not a second global step.
});

test('safe_unknown_empty',async({page})=>{
  const report=f.document();report.frames[0].state.screen.name='</script><img src=x onerror="globalThis.pwned=1"> @@VIEWER_JS@@';
  await page.goto(f.exported(report));
  await expect(page.locator('[data-frame-id]')).toContainText('</script><img');
  expect(await page.evaluate(()=>globalThis.pwned)).toBeUndefined();
  await page.evaluate(()=>{
    const root=document.querySelector('main');
    root.append(SpireReportDom.asset({assets:{byKey:{broken:'b'},blobs:{b:'data:image/png;base64,AAAA'}}},'broken','Missing original image'));
  });
  await expect(page.getByText('Missing original image',{exact:true})).toBeVisible();
  await expect(page.locator('img')).toHaveCount(0);
  await page.evaluate(()=>{
    const report=JSON.parse(document.getElementById('spire-report-data').textContent);
    report.frames[0].kind='future_kind';report.frames[0].effects=[{kind:'future_effect',sequence:0,description:'Recorded future detail'}];
    SpireReportViewer.mount(document,report);
  });
  await expect(page.locator('[data-frame-id]')).toContainText('future_kind');
  await expect(page.getByRole('complementary',{name:'Recorded resolution'})).toContainText('Recorded future detail');
  await page.evaluate(()=>{
    const report=JSON.parse(document.getElementById('spire-report-data').textContent);report.schemaVersion=99;SpireReportViewer.mount(document,report);
  });
  await expect(page.getByRole('alert')).toContainText(/unsupported.*schema/i);
  await page.goto(f.exported(f.document({frames:[],capture:{status:'interrupted',reason:'fixture'}})));
  await expect(page.getByText('No recorded frames',{exact:true})).toBeVisible();
  await expect(page.getByRole('button',{name:'Next recorded frame'})).toBeDisabled();
  await expect(page.getByRole('status')).toContainText('interrupted');
});

test('responsive_offline_layout',async({page})=>{
  const url=f.exported();
  for(const width of [1440,390]){
    await page.setViewportSize({width,height:900});await page.goto(url);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await expect(page.getByRole('complementary',{name:'Recorded resolution'})).toBeVisible();
    await expect(page.getByRole('navigation',{name:'Recorded timeline'})).toBeVisible();
  }
});

test('public_piles_and_missing_art',async({page})=>{
  const report=f.document();
  report.frames[1].state.combat={piles:{draw:{count:17,knownCards:null,orderKnown:false},
    discard:[{name:'Bash',assetKey:'card/bash'}],exhaust:[]}};
  await page.goto(f.exported(report));
  await expect(page.locator('details.warnings')).not.toHaveAttribute('open','');
  await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
  await page.getByText('Public piles',{exact:true}).click();
  await expect(page.locator('[data-frame-id]')).toContainText('order unknown');
  await expect(page.locator('[data-frame-id]')).toContainText('Bash');
});
