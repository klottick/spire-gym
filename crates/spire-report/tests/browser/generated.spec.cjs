const {test,expect}=require('@playwright/test');
const fs=require('node:fs');const path=require('node:path');const {pathToFileURL}=require('node:url');
const dir=process.env.SPIRE_REPORT_GENERATED_DIR;
test.skip(!dir,'Set SPIRE_REPORT_GENERATED_DIR to real native artifacts');
async function open(page,name){
  const errors=[];page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});
  page.on('request',r=>{if(/^https?:/.test(r.url()))errors.push(r.url())});
  await page.goto(pathToFileURL(path.join(dir,`${name}.html`)).href);
  const report=await page.locator('#spire-report-data').evaluate(n=>JSON.parse(n.textContent));
  expect(report.source.kind).not.toBe('authored_fixture');return {report,errors};
}
async function slide(page,index){await page.getByRole('slider',{name:'Recorded slide',exact:true}).evaluate((n,index)=>{n.value=String(index);n.dispatchEvent(new Event('input'))},index);}
test('native_every_frame_offline',async({page})=>{
  for(const name of ['combat','run-prefix','death','unfinished']){
    const {report,errors}=await open(page,name);
    const visited=await page.evaluate(()=>{
      const report=JSON.parse(document.getElementById('spire-report-data').textContent);const slider=document.querySelector('input[type=range]');const seen=[];
      for(let i=0;i<report.frames.length;i++){slider.value=String(i);slider.dispatchEvent(new Event('input'));seen.push(document.querySelector('[data-frame-id]')?.dataset.frameId);}
      return seen;
    });
    expect(visited).toEqual(report.frames.map(f=>f.id));
    await expect(page.getByRole('button',{name:'Next recorded frame',exact:true})).toBeDisabled();
    if(name==='run-prefix'){expect(report.game.outcome).toBeNull();expect(report.game.boundary).toBe('act2_prefix_complete');await expect(page.getByRole('status')).toContainText('prefix');}
    if(name==='death'){expect(report.game.outcome).toBe('death');await expect(page.getByRole('region',{name:'Recorded final summary'})).toContainText('Run death');}
    if(name==='unfinished'){expect(report.capture.status).toBe('unfinished');expect(report.game.outcome).toBeNull();}
    expect(errors).toEqual([]);
  }
});
test('native_resolved_move_and_effect_identity',async({page})=>{
  const {report,errors}=await open(page,'combat');
  const index=report.frames.findIndex(f=>f.effects.some(e=>e.kind==='enemy_move'));
  expect(index).toBeGreaterThan(0);await slide(page,index);
  const move=report.frames[index].effects.find(e=>e.kind==='enemy_move');
  await expect(page.getByRole('complementary',{name:'Recorded resolution'})).toContainText(`Resolved enemy move: ${move.resolvedMove.name}`);
  await expect(page.getByRole('complementary',{name:'Recorded resolution'})).not.toContainText('[object Object]');expect(errors).toEqual([]);
});
test('native_current_cost_and_terminal_resources',async({page})=>{
  const {report,errors}=await open(page,'run-prefix');
  const played=report.frames.findIndex(f=>f.action?.card);await slide(page,played);
  await expect(page.getByRole('region',{name:'Recorded chosen action'})).toContainText(`Current cost: ${report.frames[played].action.card.currentCost}`);
  const end=report.frames.findIndex(f=>f.combatFinal);await slide(page,end);
  await expect(page.getByRole('region',{name:'Recorded player'})).not.toContainText('Block Unknown');
  await expect(page.getByRole('region',{name:'Recorded player'})).not.toContainText('Energy: Unknown');expect(errors).toEqual([]);
});
test('native_ordered_primitive_values',async({page})=>{
  const {report}=await open(page,'combat');
  const index=report.frames.findIndex(f=>f.effects.some(e=>e.kind==='energy'));await slide(page,index);
  const energy=report.frames[index].effects.find(e=>e.kind==='energy');
  const line=page.getByRole('complementary',{name:'Recorded resolution'}).getByRole('listitem').filter({hasText:'Energy changed'}).first();
  await expect(line).toContainText(`Before: ${energy.before}`);await expect(line).toContainText(`After: ${energy.after}`);
});
test('native_terminal_combat_and_reward_choices_are_both_visible',async({page})=>{
  const {report}=await open(page,'run-prefix');
  const index=report.frames.findIndex(f=>f.combatFinal&&f.state.screen.kind==='rewards');expect(index).toBeGreaterThan(0);await slide(page,index);
  await expect(page.getByRole('region',{name:'Recorded enemies'})).toBeVisible();
  await expect(page.getByRole('region',{name:'Current recorded screen'})).toContainText('rewards');
  await expect(page.getByRole('region',{name:'Recorded inventory'})).toHaveCount(1);
});
test('native_multi_enemy_desktop_and_mobile',async({page},info)=>{
  const {report,errors}=await open(page,'run-prefix');
  const index=report.frames.findIndex(f=>f.state.combat?.enemies.filter(e=>e.lifeState==='alive').length>1);expect(index).toBeGreaterThan(0);
  await slide(page,index);
  const frames=report.frames;expect(frames[index].state.combat.enemies.length).toBeGreaterThan(1);
  for(const width of [1440,390]){
    await page.setViewportSize({width,height:900});await expect(page.locator('[data-frame-id]')).toHaveAttribute('data-frame-id',String(index));
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.screenshot({path:info.outputPath(`native-multi-${width}.png`),fullPage:true});
  }
  const endTurn=frames.findIndex((f,i)=>i>index&&f.combatId===frames[index].combatId&&f.action?.type==='end_turn');expect(endTurn).toBeGreaterThan(index);await slide(page,endTurn);
  for(const width of [1440,390]){await page.setViewportSize({width,height:900});await page.screenshot({path:info.outputPath(`native-end-turn-${width}.png`),fullPage:true});}
  expect(errors).toEqual([]);
});
