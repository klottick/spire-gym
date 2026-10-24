"use strict";
(()=>{
  const listeners=new WeakMap();
  const {text}=SpireReportDom;
  const known=value=>value===null||value===undefined?"Unknown":String(value);
  const context=group=>`Act ${known(group.act)} · Floor ${known(group.floor)} · ${group.combatId??"No combat"} · Turn ${known(group.turn)}`;
  function effectText(report,frame,previous,effect){
    const refs=new Map([["player","Player"]]);
    for(const view of [previous,frame]){
      const player=view?.state.player;if(player?.ref)refs.set(player.ref,"Player");
      for(const combat of [view?.state.combat,view?.combatFinal]){
        for(const enemy of combat?.enemies??[])refs.set(enemy.ref,enemy.name??enemy.monsterId??enemy.ref);
        for(const card of [...(combat?.hand??[]),...(combat?.piles?.discard??[]),...(combat?.piles?.exhaust??[]),...(combat?.stasis??[]).map(s=>s.card).filter(Boolean)])refs.set(card.ref,card.name??card.cardId??card.ref);
      }
      if(view?.action?.card)refs.set(view.action.card.ref,view.action.card.name??view.action.card.cardId);
    }
    const label=ref=>ref===null||ref===undefined?"Unattributed":refs.get(typeof ref==="object"?ref.ref:ref)??known(typeof ref==="object"?ref.name??ref.ref:ref);
    const parts=[`${label(effect.source)} → ${label(effect.target)}`,effect.description??`Recorded ${known(effect.kind)}`];
    for(const [field,title] of [["resolvedMove","Resolved enemy move"],["incoming","Incoming"],["absorbedBlock","Block absorbed"],["hpLost","HP lost"],["overkill","Overkill"]]){
      if(Object.hasOwn(effect,field)){
        const value=field==="resolvedMove"&&effect[field]&&typeof effect[field]==="object"?effect[field].name??effect[field].id:effect[field];
        parts.push(`${title}: ${known(value)}`);
      }
    }
    if(effect.card)parts.push(`Card: ${label(effect.card)}`);
    for(const [field,title] of [["before","Before"],["after","After"]])if(Object.hasOwn(effect,field))parts.push(`${title}: ${known(effect[field])}`);
    if(effect.kind==="heal")parts.push(`Healing: ${known(effect.amount)}`);
    return parts.join(" · ");
  }
  function fallback(frame){
    const panel=document.createElement("section");panel.className="recorded-screen";
    panel.append(text("h3",frame.state.screen?.name??"Unknown screen"));
    panel.append(text("p",`HP ${known(frame.state.player?.hp)} / ${known(frame.state.player?.maxHp)} · Block ${known(frame.state.player?.block)} · Energy ${known(frame.state.player?.energy)}`));
    const combat=frame.state.combat;
    if(combat?.piles){
      const piles=document.createElement("details");piles.append(text("summary","Public piles"));
      piles.append(text("p",`Draw: ${known(combat.piles.draw?.count)} cards; order unknown`));
      for(const name of ["discard","exhaust"]){
        piles.append(text("h4",name));
        if(Array.isArray(combat.piles[name]))for(const card of combat.piles[name])piles.append(text("p",card.name??card.cardId??"Unknown card"));
        else piles.append(text("p","Unknown"));
      }
      panel.append(piles);
    }
    return panel;
  }
  function mount(doc,report){
    listeners.get(doc)?.abort();const controller=new AbortController();listeners.set(doc,controller);
    const root=doc.getElementById("spire-report")??doc.body.appendChild(doc.createElement("main"));
    root.replaceChildren();
    const error=SpireReportModel.validateReport(report);
    if(error){const message=text("p",error,"error");message.setAttribute("role","alert");root.append(message);return root;}
    root.append(text("h1","Spire Gym — recorded history"));
    root.append(text("p",`${known(report.source.kind)} · ${known(report.source.profile)} · Evidence: ${known(report.source.evidenceGrade)} · Seed ${known(report.run.seed?.value)}`,"source"));
    const status=text("p",`Capture: ${known(report.capture?.status)}${report.game?.boundary?` · Prefix/boundary: ${report.game.boundary}`:""}${report.game?.outcome?` · Recorded outcome: ${report.game.outcome}`:""}`,"capture-status");
    status.setAttribute("role","status");root.append(status);
    root.append(text("p","Read-only history. Controls navigate recorded slides; they do not play or simulate.","muted"));
    if(report.warnings?.length){
      const warnings=doc.createElement("details");warnings.className="warnings";warnings.append(text("summary",`Artwork/report warnings (${report.warnings.length})`));
      const list=doc.createElement("ul");for(const warning of report.warnings)list.append(text("li",warning));warnings.append(list);root.append(warnings);
    }
    const controls=doc.createElement("div");controls.className="navigation-controls";
    const previous=text("button","Previous");previous.setAttribute("aria-label","Previous recorded frame");
    const next=text("button","Next");next.setAttribute("aria-label","Next recorded frame");
    const slider=doc.createElement("input");slider.type="range";slider.min="0";slider.max=String(Math.max(0,report.frames.length-1));slider.value="0";slider.setAttribute("aria-label","Recorded slide");
    const position=text("span","");position.className="slide-position";
    const jump=doc.createElement("select");jump.setAttribute("aria-label","Jump to recorded context");
    controls.append(previous,slider,next,position,jump);root.append(controls);
    const workspace=doc.createElement("div");workspace.className="workspace";
    const outline=doc.createElement("nav");outline.setAttribute("aria-label","Recorded timeline");outline.append(text("h2","Timeline"));
    const board=doc.createElement("div");board.className="board";
    const inspector=doc.createElement("aside");inspector.setAttribute("aria-label","Recorded resolution");inspector.className="inspector";
    workspace.append(outline,board,inspector);root.append(workspace);
    let index=0;
    const groups=SpireReportModel.groupFrames(report);
    for(const group of groups){
      const option=text("option",context(group));option.value=String(group.indices[0]);jump.append(option);
      const entry=text("button",context(group));entry.addEventListener("click",()=>show(group.indices[0]));outline.append(entry);
    }
    function show(requested){
      index=Math.max(0,Math.min(requested,report.frames.length-1));
      const frame=SpireReportModel.frameAt(report,index);const prior=SpireReportModel.frameAt(report,index-1);
      previous.disabled=!frame||index===0;next.disabled=!frame||index===report.frames.length-1;slider.disabled=!frame;jump.disabled=!frame;slider.value=String(index);
      position.textContent=frame?`${index+1} / ${report.frames.length}`:"0 / 0";
      inspector.replaceChildren(text("h2","Recorded resolution"));
      if(!frame){board.replaceChildren(text("p","No recorded frames"));return;}
      const currentGroup=groups.find(group=>group.indices.includes(index));jump.value=String(currentGroup.indices[0]);
      const slide=doc.createElement("article");slide.className="slide";slide.dataset.frameId=frame.id;
      slide.append(text("h2",`Slide ${index+1}: ${known(frame.kind)}`));
      slide.append(text("p",frame.action?.description??"Boundary — no chosen action", "chosen-action"));
      const renderer=(frame.state.combat||frame.combatFinal)?globalThis.SpireReportCombat:globalThis.SpireReportScreens;
      slide.append(renderer?.render?renderer.render(report,frame,prior):fallback(frame));
      if(frame.combatFinal&&!frame.state.combat&&frame.state.screen.kind!=="combat")slide.append(globalThis.SpireReportScreens.render(report,frame,prior,false));
      const summary=globalThis.SpireReportScreens?.finalSummary(report,frame);if(summary)slide.append(summary);
      board.replaceChildren(slide);
      if(frame.stateValidity==="last_known")inspector.append(text("p","Last known state — resolution may be partial","error"));
      if(!frame.effects.length)inspector.append(text("p","No recorded effects"));
      const effects=doc.createElement("ol");
      for(const effect of [...frame.effects].sort((a,b)=>a.sequence-b.sequence))effects.append(text("li",effectText(report,frame,prior,effect)));
      inspector.append(effects);
    }
    previous.addEventListener("click",()=>show(index-1));next.addEventListener("click",()=>show(index+1));
    slider.addEventListener("input",()=>show(Number(slider.value)));jump.addEventListener("change",()=>show(Number(jump.value)));
    doc.addEventListener("keydown",event=>{
      if(event.defaultPrevented||event.ctrlKey||event.metaKey||event.altKey||event.shiftKey)return;
      const focused=doc.activeElement;
      if(focused?.matches("input,select,textarea")||focused?.isContentEditable)return;
      if(event.key==="ArrowLeft"||event.key==="ArrowRight"){event.preventDefault();show(index+(event.key==="ArrowRight"?1:-1));}
    },{signal:controller.signal});
    show(0);return root;
  }
  globalThis.SpireReportViewer=Object.freeze({mount});
  const data=document.getElementById("spire-report-data");
  if(data){
    try{mount(document,JSON.parse(data.textContent));}
    catch(error){const root=document.getElementById("spire-report");const message=text("p","Invalid display report data","error");message.setAttribute("role","alert");root.replaceChildren(message);}
  }
})();
