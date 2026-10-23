"use strict";
(()=>{
  const {text,asset}=SpireReportDom;
  const known=value=>value===null||value===undefined?"Unknown":String(value);
  function region(name,className){const node=document.createElement("section");node.setAttribute("aria-label",name);node.className=className??"";return node;}
  function cardView(report,card){
    const node=region(card.name??card.cardId??"Unknown card","recorded-card");
    if(card.ref!==null&&card.ref!==undefined)node.dataset.cardRef=card.ref;
    node.append(text("h4",`${card.name??card.cardId??"Unknown card"}${card.upgrades?` +${card.upgrades}`:""}`));
    node.append(asset(report,card.assetKey,card.name??"Card artwork","card-art"));
    node.append(text("p",`Current cost: ${known(card.currentCost)} · Base cost: ${known(card.baseCost)}`));
    node.append(text("p",`Free to play: ${card.freeToPlay===true?"yes":card.freeToPlay===false?"no":"Unknown"}`));
    return node;
  }
  function inventory(report,items){
    const node=region("Recorded inventory","inventory");node.append(text("h3","Inventory"));
    if(!items){node.append(text("p","Inventory unknown"));return node;}
    const relics=region("Recorded relics","relics");
    if(Array.isArray(items.relics)){
      if(!items.relics.length)relics.append(text("p","No relics"));
      for(const relic of items.relics){
        const item=document.createElement("div");item.append(asset(report,relic.assetKey,relic.name??relic.relicId??"Unknown relic","item-art"));
        if(relic.counters&&typeof relic.counters==="object")for(const [name,value] of Object.entries(relic.counters))item.append(text("span",`${name}: ${known(value)}`));
        relics.append(item);
      }
    }else relics.append(text("p","Relics unknown"));
    const potions=region("Recorded potion slots","potions");
    if(Array.isArray(items.potions))items.potions.forEach((potion,index)=>{
      const item=document.createElement("div");
      item.append(potion===null?text("span",`Empty potion slot ${index+1}`):asset(report,potion.assetKey,potion.name??potion.potionId??`Potion slot ${index+1}`,"item-art"));potions.append(item);
    });else potions.append(text("p","Potion inventory unknown"));
    node.append(relics,potions);
    if(items.keys!==null&&items.keys!==undefined)node.append(text("p",`Keys: ${JSON.stringify(items.keys)}`));
    return node;
  }
  function health(node,current,previous){
    node.append(text("p",`HP ${known(current?.hp)} / ${known(current?.maxHp)} · Block ${known(current?.block)}`));
    const delta=SpireReportModel.healthDelta(previous?.hp,current?.hp,current?.maxHp);
    if(![current?.hp,current?.maxHp].every(value=>typeof value==="number"&&Number.isFinite(value))||current.maxHp<=0)return;
    const hp=Math.max(0,Math.min(current.hp,current.maxHp));
    const bar=document.createElement("div");bar.className="health-bar";bar.setAttribute("role","img");bar.setAttribute("aria-label",`HP ${hp} of ${current.maxHp}${delta?`; net HP loss ${delta.lost}`:"; prior checkpoint unknown"}`);
    const remaining=text("span","","hp-remaining");remaining.style.width=`${hp/current.maxHp*100}%`;bar.append(remaining);
    if(delta?.lost){const lost=text("span","","hp-lost");lost.style.width=`${delta.lost/delta.maxHp*100}%`;bar.append(lost);}
    node.append(bar);
    if(delta)node.append(text("p",`Net HP loss: ${delta.lost} · Net healing: ${delta.gained}`));
  }
  function retainedCard(report,frame,combat){
    if(frame.action?.card)return frame.action.card;
    if(typeof frame.action?.parentActionId==="string"){
      const parent=report.frames.find(item=>item.id===frame.action.parentActionId);
      if(parent?.combatId===frame.combatId&&parent.action?.card)return parent.action.card;
    }
    if(combat.lastPlayedCard)return combat.lastPlayedCard;
    const index=report.frames.findIndex(item=>item.id===frame.id);
    for(let i=index-1;i>=0;i--){
      const prior=report.frames[i];
      if(prior.combatId!==frame.combatId)break;
      if(prior.action?.card)return prior.action.card;
    }
    return null;
  }
  function render(report,frame,previous){
    const combat=frame.state.combat??frame.combatFinal??{};
    const root=document.createElement("div");root.append(inventory(report,frame.state.inventory));
    const board=document.createElement("div");board.className="tactical";
    const player=region("Recorded player","player");player.append(text("h3",report.run.character??"Player"));
    if(frame.state.player?.assetKey)player.append(asset(report,frame.state.player.assetKey,report.run.character??"Player","actor-art"));
    const sameCombat=previous?.combatId===frame.combatId;
    const playerValues=frame.state.combat?frame.state.player:(combat.player??frame.state.player);
    if(!frame.state.combat&&combat.player)player.append(text("p","Terminal combat checkpoint"),text("p",`Settled run HP: ${known(frame.state.player?.hp)} · Gold: ${known(frame.state.player?.gold)}`));
    health(player,playerValues,sameCombat?previous.state.player:null);
    player.append(text("p",`Energy: ${known(playerValues?.energy)}`));
    for(const power of playerValues?.statuses??[])player.append(text("p",typeof power==="string"?power:`${power.name??power.id}: ${known(power.amount)}`));
    if(combat.character)player.append(text("p",`Character state: ${JSON.stringify(combat.character)}`));
    const action=region("Recorded chosen action","chosen-card");action.append(text("h3",frame.action?.description??"Recorded action"));
    const chosen=retainedCard(report,frame,combat);if(chosen)action.append(cardView(report,chosen));else action.append(text("p","No played card recorded"));
    if(combat.selection){action.append(text("p",`Pending selection: ${known(combat.selection.kind)}`));for(const card of combat.selection.cards??[])action.append(text("p",card.name??card.cardId??"Unknown card"));}
    const enemies=region("Recorded enemies","enemies");enemies.append(text("h3","Enemy roster"));
    if(!Array.isArray(combat.enemies))enemies.append(text("p","Enemies unknown"));
    for(const enemy of combat.enemies??[]){
      const actor=region(enemy.name??enemy.monsterId??"Unknown enemy","enemy");if(enemy.ref!==null&&enemy.ref!==undefined)actor.dataset.actorRef=enemy.ref;
      actor.append(text("h4",`${enemy.name??enemy.monsterId??"Unknown enemy"} · Slot ${known(enemy.logicalSlot)} · ${known(enemy.lifeState)}`));
      actor.append(asset(report,enemy.assetKey,enemy.name??"Enemy artwork","actor-art"));
      const old=previous?.state.combat?.enemies?.find(item=>item.ref===enemy.ref);
      health(actor,enemy,sameCombat?old:null);
      for(const power of enemy.statuses??[])actor.append(text("p",typeof power==="string"?power:`${power.name??power.id}: ${known(power.amount)}`));
      if(enemy.lifeState==="alive"&&enemy.hp>0){
        actor.append(text("p",`Upcoming intent: ${known(enemy.intent?.name)}`));
        actor.append(text("p",`Damage per hit: ${known(enemy.intent?.damagePerHit)} · Hit count: ${known(enemy.intent?.hitCount)}`));
      }
      enemies.append(actor);
    }
    board.append(player,action,enemies);root.append(board);
    const hand=region("Recorded hand","hand");hand.append(text("h3","Hand"));
    if(Array.isArray(combat.hand))for(const card of combat.hand)hand.append(cardView(report,card));else hand.append(text("p","Hand unknown"));root.append(hand);
    const piles=document.createElement("details");piles.append(text("summary","Public piles"));
    piles.append(text("p",`Draw: ${known(combat.piles?.draw?.count)} cards; order unknown`));
    for(const name of ["discard","exhaust"]){
      piles.append(text("h4",name));
      if(Array.isArray(combat.piles?.[name]))for(const card of combat.piles[name])piles.append(cardView(report,card));else piles.append(text("p","Unknown"));
    }root.append(piles);
    if(combat.stasis?.length){const held=region("Recorded Stasis holdings");held.append(text("h3","Stasis holdings"));for(const item of combat.stasis){if(item.card)held.append(cardView(report,item.card));}root.append(held);}
    return root;
  }
  globalThis.SpireReportCombat=Object.freeze({render,inventory});
})();
