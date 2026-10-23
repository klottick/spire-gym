const {test,expect}=require('@playwright/test');
const f=require('./fixtures.cjs');
test.beforeEach(async({page})=>{
  page.reportErrors=[];page.on('pageerror',error=>page.reportErrors.push(String(error)));
  page.on('console',message=>{if(message.type()==='error')page.reportErrors.push(message.text());});
  page.on('request',request=>{if(/^https?:/.test(request.url()))page.reportErrors.push(request.url());});
});
test.afterEach(async({page})=>expect(page.reportErrors).toEqual([]));
test.afterAll(f.cleanup);
function shopFrames(){
  const frames=[f.frame(0),f.frame(1)];
  frames[0].state.screen={kind:'shop',name:'Merchant',selection:null,options:[
    {key:'buy:4',label:'Bash',status:'offered',data:{price:60}},
    {key:'purge:4',label:'Remove a card',status:'offered',data:{price:75}},
    {key:'buy:5',label:'Disabled offer',status:'disabled',data:{price:999}}]};
  frames[1].beforeScreen=frames[0].state.screen;
  frames[1].action={type:'buy',description:'Buy Bash',optionKey:'buy:4',preActionOffer:{price:60}};
  frames[1].kind='action';frames[1].state.screen={kind:'map',name:'Map',options:[],selection:null};
  frames[1].state.player.gold=39;
  return frames;
}
test('screen_exact_option_and_historical_offers',async({page})=>{
  await page.goto(f.exported(f.document({frames:shopFrames()})));
  await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
  const historical=page.getByRole('region',{name:'Before chosen action'});
  await expect(historical.locator('[data-chosen="true"]')).toHaveCount(1);
  await expect(historical.locator('[data-chosen="true"]')).toContainText('Bash');
  await expect(historical).toContainText('price: 60');await expect(historical).toContainText('disabled');
  await expect(page.locator('[data-frame-id]')).toContainText('Gold: 99 → 39');
  await expect(historical.getByRole('button')).toHaveCount(0);
  await page.evaluate(()=>{
    const report=JSON.parse(document.getElementById('spire-report-data').textContent);
    delete report.frames[1].action.optionKey;SpireReportViewer.mount(document,report);
  });
  await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
  await expect(page.locator('[data-chosen="true"]')).toHaveCount(0);
});

test('screen_map_history_does_not_use_future_choices',async({page})=>{
  const frames=[f.frame(0),f.frame(1)];
  for(const frame of frames)frame.state.map={current:'n0',visited:['n0'],nodes:[{id:'n0',kind:'map'},{id:'n1',kind:'question'},{id:'n2',kind:'combat'}],edges:[{from:'n0',to:'n1'},{from:'n0',to:'n2'}]};
  frames[1].state.map={...frames[1].state.map,current:'n2',visited:['n0','n2']};
  frames[1].action={type:'move',description:'Choose node n2',optionKey:'move:n2'};
  frames[1].kind='action';
  await page.goto(f.exported(f.document({frames})));
  await expect(page.getByRole('region',{name:'Recorded route history'})).not.toContainText('n2');
  await expect(page.getByRole('region',{name:'Public map'})).toContainText('n0 → n1');
  await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
  await expect(page.getByRole('region',{name:'Recorded route history'})).toContainText('n2');
});

test('screen_selection_inventory_and_unknown_screens',async({page})=>{
  for(const kind of ['event','shop','rewards','rest','selection','future_screen']){
    const frame=f.frame(0);frame.state.screen={kind,name:`Recorded ${kind}`,options:[{key:'a',label:'Historical option',status:'skipped',data:{heal:20}}],selection:{kind:'upgrade',cards:[{ref:'c1',name:'Bash',upgrades:1}]}};
    frame.state.inventory={deck:[{ref:'c1',name:'Bash',upgrades:1}],relics:[],potions:[null],keys:null};
    await page.goto(f.exported(f.document({frames:[frame]})));
    await expect(page.locator('[data-frame-id]')).toContainText('Historical option');
    await expect(page.locator('[data-frame-id]')).toContainText('Bash');
    await expect(page.getByText('Empty potion slot 1',{exact:true})).toBeVisible();
    await expect(page.locator('[data-frame-id]').getByRole('button')).toHaveCount(0);
  }
});

test('screen_outcomes_distinguish_capture_and_game',async({page})=>{
  const source={...f.document().source,kind:'run_prefix'};
  await page.goto(f.exported(f.document({source,game:{outcome:null,boundary:'act2_prefix_complete'}})));
  await expect(page.getByRole('status')).toContainText(/prefix/i);
  await expect(page.locator('[data-final-summary]')).toHaveCount(0);
  await page.getByRole('combobox',{name:'Jump to recorded context'}).selectOption('2');
  await expect(page.locator('[data-final-summary]')).toContainText(/not a complete run/i);
  for(const [outcome,label] of [['combat_victory','Combat victory'],['death','Run death'],['heart_victory','Heart victory'],['empty_deck','empty deck'],['act3_no_keys','without required keys']]){
    await page.goto(f.exported(f.document({game:{outcome,boundary:null}})));
    await expect(page.locator('[data-final-summary]')).toHaveCount(0);
    await page.getByRole('combobox',{name:'Jump to recorded context'}).selectOption('2');
    await expect(page.locator('[data-final-summary]')).toContainText(label);
  }
  for(const status of ['unfinished','abandoned','faulted','interrupted']){
    await page.goto(f.exported(f.document({capture:{status,reason:'fixture stop'}})));
    await page.getByRole('combobox',{name:'Jump to recorded context'}).selectOption('2');
    await expect(page.locator('[data-final-summary]')).toContainText(status);
    await expect(page.locator('[data-final-summary]')).not.toContainText('Run death');
  }
});
