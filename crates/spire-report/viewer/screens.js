"use strict";
(()=>{
  const {text}=SpireReportDom;
  const known=value=>value===null||value===undefined?"Unknown":String(value);
  function region(name){const node=document.createElement("section");node.setAttribute("aria-label",name);return node;}
  function values(parent,data){
    if(!data||typeof data!=="object")return;
    for(const [name,value] of Object.entries(data))parent.append(text("p",`${name}: ${value!==null&&typeof value==="object"?JSON.stringify(value):known(value)}`));
  }
  function options(screen,selectedKey,name){
    const panel=region(name);panel.className="recorded-options";panel.append(text("h3",`${name}: ${screen?.name??screen?.kind??"Unknown"}`));
    const list=document.createElement("ul");
    for(const option of screen?.options??[]){
      const item=document.createElement("li");
      if(typeof option.key==="string")item.dataset.optionKey=option.key;
      const chosen=typeof selectedKey==="string"&&typeof option.key==="string"&&selectedKey===option.key;
      if(chosen)item.dataset.chosen="true";
      item.append(text("h4",`${chosen?"Chosen — ":""}${option.label??"Unknown option"} · ${known(option.status)}`));
      values(item,option.data);list.append(item);
    }
    panel.append(list);
    if(screen?.selection){
      const selection=region("Recorded selection");selection.append(text("h4",`Selection: ${known(screen.selection.kind)}`));values(selection,screen.selection);panel.append(selection);
    }
    return panel;
  }
  function finalSummary(report,frame){
    const last=report.frames.at(-1)?.id===frame.id;
    if(!last&&!frame.state.terminal&&frame.kind!=="end"&&frame.kind!=="fault")return null;
    const summary=region("Recorded final summary");summary.dataset.finalSummary="true";summary.className="final-summary";
    summary.append(text("h3",last?"Final recorded boundary":"Recorded terminal/fault boundary"));
    summary.append(text("p",`Capture status: ${known(report.capture?.status)}${report.capture?.reason?` · ${report.capture.reason}`:""}`));
    const outcomes={combat_victory:"Combat victory (not run completion)",death:"Run death",heart_victory:"Heart victory",empty_deck:"Run ended: empty deck",act3_no_keys:"Act 3 completion without required keys"};
    summary.append(text("p",outcomes[report.game?.outcome]??"No run terminal outcome recorded"));
    if(report.game?.boundary)summary.append(text("p",`Recorded boundary: ${report.game.boundary}`));
    if(report.source.kind==="run_prefix")summary.append(text("p","Captured prefix — not a complete run"));
    return summary;
  }
  function render(report,frame,previous,includeInventory=true){
    const root=document.createElement("div");if(includeInventory)root.append(SpireReportCombat.inventory(report,frame.state.inventory));
    const current=frame.state.player??{};const before=previous?.state.player;
    if(includeInventory)root.append(text("p",`HP: ${known(current.hp)} / ${known(current.maxHp)} · Gold: ${before?`${known(before.gold)} → `:""}${known(current.gold)}`));
    if(frame.beforeScreen)root.append(options(frame.beforeScreen,frame.action?.optionKey,"Before chosen action"));
    if(frame.action?.preActionOffer){const offer=region("Recorded pre-action offer");offer.append(text("h3","Exact pre-action offer"));values(offer,frame.action.preActionOffer);root.append(offer);}
    root.append(options(frame.state.screen,null,"Current recorded screen"));
    const map=frame.state.map;
    if(map){
      const panel=region("Public map");panel.append(text("h3","Public map"),text("p",`Current node: ${known(map.current)}`));
      for(const node of map.nodes??[])panel.append(text("p",`${known(node.id)} · ${known(node.kind)}`));
      for(const edge of map.edges??[])panel.append(text("p",`${known(edge.from)} → ${known(edge.to)}`));
      const route=region("Recorded route history");route.append(text("h4","Recorded route history"));
      route.append(text("p",Array.isArray(map.visited)?map.visited.map(known).join(" → "):"Unknown"));panel.append(route);root.append(panel);
    }
    const deck=frame.state.inventory?.deck;
    if(Array.isArray(deck)){
      const details=document.createElement("details");details.append(text("summary",`Current recorded deck (${deck.length})`));
      for(const card of deck)details.append(text("p",`${card.name??card.cardId??"Unknown card"}${card.upgrades?` +${card.upgrades}`:""}`));root.append(details);
    }
    return root;
  }
  globalThis.SpireReportScreens=Object.freeze({render,finalSummary});
})();
