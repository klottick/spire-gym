const fs=require('node:fs');
const path=require('node:path');
const {pathToFileURL}=require('node:url');
const {spawnSync}=require('node:child_process');
const roots=[];
function frame(index,changes={}){
  return {id:String(index),previousFrameId:index?String(index-1):null,kind:index?'boundary':'initial',
    act:1,floor:index,combatId:index?'combat-0':null,turn:index?1:null,action:null,beforeScreen:null,
    state:{screen:{kind:index?'combat':'map',name:index?'Combat':'Map',options:[],selection:null},
      player:{hp:80-index,maxHp:80,gold:99,block:0,energy:3,statuses:[]},inventory:null,map:null,combat:null,terminal:null},
    effects:[],combatFinal:null,stateValidity:'settled',...changes};
}
function document(overrides={}){
  return {schemaId:'spire-report-v1',schemaVersion:1,
    source:{kind:'authored_fixture',profile:'browser fixture',producerRevision:null,contentId:null,evidenceGrade:'fixture'},
    run:{character:'ironclad',ascension:0,seed:{encoding:'decimal',value:'18446744073709551616'},caller:null},
    catalog:{},frames:[frame(0),frame(1),frame(2)],capture:{status:'complete',reason:null},game:{outcome:null,boundary:null},...overrides};
}
function exported(report=document()){
  const root=fs.mkdtempSync('/tmp/opencode/spire-report-browser-');roots.push(root);
  const trace=path.join(root,'trace.jsonl');const output=path.join(root,'report.html');
  const records=[{record:'header',schemaId:report.schemaId,schemaVersion:report.schemaVersion,source:report.source,run:report.run}];
  if(Object.keys(report.catalog).length)records.push({record:'catalog',entries:report.catalog});
  records.push(...report.frames.map(frame=>({record:'frame',frame})),{record:'footer',capture:report.capture,game:report.game});
  fs.writeFileSync(trace,records.map(record=>JSON.stringify(record)+'\n').join(''));
  if(!process.env.SPIRE_REPORT_BIN)throw new Error('Set SPIRE_REPORT_BIN to the actual native exporter binary');
  const result=spawnSync(process.env.SPIRE_REPORT_BIN,['--trace',trace,'--output',output],{encoding:'utf8'});
  if(result.status!==0)throw new Error(`Native fixture export failed: ${result.stderr}`);
  return pathToFileURL(output).href;
}
function cleanup(){for(const root of roots.splice(0))fs.rmSync(root,{recursive:true});}
module.exports={frame,document,exported,cleanup};
