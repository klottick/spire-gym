const {test,expect}=require('@playwright/test');
const f=require('./fixtures.cjs');
const card=(ref,name='Strike')=>({ref,origin:'permanent-1',cardId:'strike',name,upgrades:0,baseCost:1,currentCost:1,freeToPlay:null,assetKey:'card/strike'});
function combatFrame(index,changes={}){
  const base=f.frame(index);
  return {...base,combatId:'battle-0',turn:1,state:{...base.state,
    player:{ref:'player',hp:10,maxHp:10,block:0,energy:3,statuses:[]},
    inventory:{deck:[card('card-1')],relics:[{name:'Burning Blood',counters:{charges:2},assetKey:'relic/burning_blood'}],potions:[null,null,null],keys:null},
    combat:{id:'battle-0',turn:1,enemies:[{ref:'enemy-1',logicalSlot:0,name:'Louse',monsterId:'louse',hp:8,maxHp:10,block:0,statuses:[],lifeState:'alive',intent:{name:'Bite',damagePerHit:6,hitCount:null}}],
      hand:[card('card-1')],piles:{draw:{count:17,knownCards:null,orderKnown:false},discard:[],exhaust:[]},selection:null,character:null,stasis:[],lastPlayedCard:null,outcome:null}},...changes};
}
function report(frames){
  for(const frame of frames)if(frame.id!=='0'&&frame.action)frame.kind='action';
  return f.document({frames,source:{...f.document().source,profile:'combat contract fixtures'}});
}
test.beforeEach(async({page})=>{
  page.errors=[];page.on('pageerror',e=>page.errors.push(String(e)));page.on('request',r=>{if(/^https?:/.test(r.url()))page.errors.push(r.url());});
  page.on('console',message=>{if(message.type()==='error')page.errors.push(message.text());});
});
test.afterEach(async({page})=>expect(page.errors).toEqual([]));
test.afterAll(f.cleanup);

test('played_card_persists_and_stops_at_next_combat',async({page})=>{
  const frames=[combatFrame(0),combatFrame(1),combatFrame(2),combatFrame(3)];
  frames[1].kind='action';frames[1].action={type:'play_card',description:'Play exact Strike',card:card('card-1'),targets:['enemy-1']};
  frames[1].state.combat.hand=[];frames[2].action={type:'end_turn',description:'End Turn',card:null};
  frames[3].combatId='battle-1';frames[3].state.combat.id='battle-1';
  await page.goto(f.exported(report(frames)));
  for(let i=1;i<=2;i++){
    await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
    await expect(page.getByRole('region',{name:'Recorded chosen action'}).locator('[data-card-ref="card-1"]')).toBeVisible();
  }
  await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
  await expect(page.getByRole('region',{name:'Recorded chosen action'}).locator('[data-card-ref]')).toHaveCount(0);
});

test('selection_parent_and_public_inventory',async({page})=>{
  const frames=[combatFrame(0),combatFrame(1),combatFrame(2)];
  frames[1].action={type:'play_card',description:'Choose a card',card:card('card-7','True Grit')};
  frames[2].action={type:'select_card',description:'Select Strike',parentActionId:'1',card:null};
  frames[2].state.combat.selection={kind:'exhaust',cards:[card('card-1')]};
  await page.goto(f.exported(report(frames)));
  await page.getByRole('combobox',{name:'Jump to recorded context'}).selectOption('2');
  await expect(page.getByRole('region',{name:'Recorded chosen action'})).toContainText('True Grit');
  await expect(page.getByText('Pending selection: exhaust',{exact:true})).toBeVisible();
  await expect(page.getByText('Empty potion slot 2',{exact:true})).toBeVisible();
  await expect(page.getByRole('region',{name:'Recorded inventory'})).toContainText('charges: 2');
});

test('multi_enemy_lifetimes_and_resolved_not_intent',async({page})=>{
  const frame=combatFrame(0);
  frame.state.combat.enemies.push({ref:'enemy-2',logicalSlot:1,name:'Old Slime',hp:0,maxHp:12,block:0,statuses:[],lifeState:'replaced',intent:{name:'Old attack',damagePerHit:99,hitCount:9}});
  frame.effects=[{sequence:0,kind:'enemy_move',source:'enemy-1',target:'player',resolvedMove:'Spit',description:'Resolved enemy move'},
    {sequence:1,kind:'damage',source:'enemy-1',target:'player',incoming:7,absorbedBlock:5,hpLost:2,overkill:0}];
  await page.goto(f.exported(report([frame])));
  await expect(page.getByRole('region',{name:'Recorded enemies'})).toContainText('Old Slime');
  await expect(page.locator('[data-actor-ref="enemy-2"]')).toContainText('replaced');
  await expect(page.locator('[data-actor-ref="enemy-2"]')).not.toContainText('Old attack');
  await expect(page.locator('[data-actor-ref="enemy-1"]')).toContainText('Upcoming intent: Bite');
  await expect(page.locator('[data-actor-ref="enemy-1"]')).toContainText('Hit count: Unknown');
  await expect(page.getByRole('complementary',{name:'Recorded resolution'})).toContainText('Spit');
  await expect(page.getByRole('complementary',{name:'Recorded resolution'})).toContainText('HP lost: 2');
});

test('terminal_death_does_not_advertise_survivor_intent_as_upcoming',async({page})=>{
  const frame=combatFrame(0);frame.state.player.hp=0;frame.state.terminal='death';
  frame.combatFinal=frame.state.combat;frame.combatFinal.outcome='death';frame.state.combat=null;
  const doc=report([frame]);doc.game.outcome='death';
  await page.goto(f.exported(doc));
  const roster=page.getByRole('region',{name:'Recorded enemies'});
  await expect(roster).toContainText('Louse');await expect(roster).toContainText('alive');
  await expect(roster).not.toContainText('Upcoming intent');
  await expect(roster).not.toContainText('Damage per hit');
  await expect(page.getByRole('region',{name:'Recorded final summary'})).toContainText('Run death');
});

test('health_semantics_and_unknown_resources',async({page})=>{
  const frames=[combatFrame(0),combatFrame(1)];frames[1].state.player.hp=7;
  frames[1].effects=[{sequence:0,kind:'damage',source:null,target:'player',hpLost:7,incoming:7,absorbedBlock:0,overkill:0},
    {sequence:1,kind:'heal',source:null,target:'player',amount:4}];
  await page.goto(f.exported(report(frames)));
  await expect(page.getByRole('region',{name:'Recorded player'}).locator('.health-bar')).toBeVisible();
  await expect(page.getByRole('region',{name:'Recorded player'})).not.toContainText('Net HP loss: 0');
  await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
  expect(await page.evaluate(()=>SpireReportModel.healthDelta(10,7,10))).toEqual({remaining:7,lost:3,gained:0,maxHp:10});
  await expect(page.getByRole('region',{name:'Recorded player'})).toContainText('Net HP loss: 3');
  await expect(page.locator('.hp-lost')).toHaveAttribute('style',/30%/);
  const effects=page.getByRole('complementary',{name:'Recorded resolution'});
  await expect(effects).toContainText('HP lost: 7');await expect(effects).toContainText('Healing: 4');await expect(effects).toContainText('Unattributed');
  expect(await page.evaluate(()=>SpireReportModel.healthDelta(null,7,10))).toBeNull();
  expect(await page.evaluate(()=>SpireReportModel.healthDelta(2,0,10).lost)).toBe(2);
});

test('blocked_hits_and_overkill_do_not_inflate_the_loss_segment',async({page})=>{
  for(const [before,after,incoming,absorbed,lost,overkill] of [[10,3,7,0,7,0],[10,10,7,7,0,0],[2,0,9,0,2,7]]){
    const frames=[combatFrame(0),combatFrame(1)];frames[0].state.player.hp=before;frames[1].state.player.hp=after;
    frames[1].effects=[{sequence:0,kind:'damage',source:null,target:'player',incoming,absorbedBlock:absorbed,hpLost:lost,overkill}];
    await page.goto(f.exported(report(frames)));await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
    const player=page.getByRole('region',{name:'Recorded player'});
    await expect(player).toContainText(`Net HP loss: ${lost}`);
    if(lost)await expect(player.locator('.hp-lost')).toHaveAttribute('style',new RegExp(`${lost*10}%`));
    else await expect(player.locator('.hp-lost')).toHaveCount(0);
  }
});

test('current_metadata_and_character_state_are_recorded_values',async({page})=>{
  const frame=combatFrame(1);const chosen=card('fixture-x','Fixture card');
  chosen.upgrades=3;chosen.baseCost='X';chosen.currentCost=0;chosen.freeToPlay=true;
  frame.action={type:'fixture_play',description:'Schema-only X/free/+3 fixture, not native mechanics',card:chosen};
  frame.state.combat.character={stance:'Calm',orbs:[{kind:'Frost',value:2}]};
  frame.state.combat.stasis=[{enemyRef:'enemy-1',card:card('held-1','Held card')}];
  await page.goto(f.exported(report([combatFrame(0),frame])));
  await page.getByRole('button',{name:'Next recorded frame',exact:true}).click();
  const action=page.getByRole('region',{name:'Recorded chosen action'});
  await expect(action).toContainText('+3');await expect(action).toContainText('Current cost: 0');await expect(action).toContainText('Base cost: X');await expect(action).toContainText('Free to play: yes');
  await expect(page.locator('[data-frame-id]')).toContainText('Calm');await expect(page.locator('[data-frame-id]')).toContainText('Frost');
  await expect(page.locator('[data-frame-id]')).toContainText('Held card');
});
