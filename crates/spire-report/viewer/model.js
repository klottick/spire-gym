"use strict";
globalThis.SpireReportModel=Object.freeze({
  validateReport(report){
    if(!report||report.schemaId!=="spire-report-v1"||report.schemaVersion!==1)return "Unsupported display report schema";
    if(!Array.isArray(report.frames)||!report.source||!report.run)return "Invalid display report structure";
    for(let i=0;i<report.frames.length;i++){
      const frame=report.frames[i];
      if(!frame||frame.id!==String(i)||frame.previousFrameId!==(i?String(i-1):null)||!frame.state||!Array.isArray(frame.effects))return "Invalid recorded frame history";
    }
    return null;
  },
  frameAt(report,index){return Number.isInteger(index)&&index>=0?report.frames[index]??null:null;},
  groupFrames(report){
    const groups=[];
    for(let i=0;i<report.frames.length;i++){
      const {act,floor,combatId,turn}=report.frames[i];
      const last=groups.at(-1);
      if(last&&last.act===act&&last.floor===floor&&last.combatId===combatId&&last.turn===turn)last.indices.push(i);
      else groups.push({act,floor,combatId,turn,indices:[i]});
    }
    return groups;
  },
  healthDelta(beforeHp,afterHp,maxHp){
    if(![beforeHp,afterHp,maxHp].every(n=>typeof n==="number"&&Number.isFinite(n))||maxHp<=0)return null;
    const before=Math.max(0,Math.min(beforeHp,maxHp));
    const remaining=Math.max(0,Math.min(afterHp,maxHp));
    return {remaining,lost:Math.max(0,before-remaining),gained:Math.max(0,remaining-before),maxHp};
  },
});
