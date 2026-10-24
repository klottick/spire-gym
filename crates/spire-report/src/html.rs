use crate::ReportError;
use crate::{read_report,load_assets,embed_referenced};
use serde_json::{Value,json};
use std::{collections::BTreeSet,io::{BufRead,Write},path::Path};

pub fn json_for_html(value:&Value)->Result<String,ReportError>{
    Ok(serde_json::to_string(value)?.replace('&',"\\u0026").replace('<',"\\u003c").replace('>',"\\u003e")
        .replace('\u{2028}',"\\u2028").replace('\u{2029}',"\\u2029"))
}

fn asset_keys(value:&Value,keys:&mut BTreeSet<String>){
    match value{
        Value::Object(fields)=>for (key,value) in fields{
            if key=="assetKey" {if let Some(key)=value.as_str(){keys.insert(key.into());}}
            else{asset_keys(value,keys);}
        },
        Value::Array(values)=>for value in values{asset_keys(value,keys);},
        _=>{},
    }
}

pub fn export_html<R:BufRead,W:Write>(trace:R,manifest:Option<&Path>,mut out:W)->Result<(),ReportError>{
    let mut report=read_report(trace,64*1024*1024)?;
    let mut keys=BTreeSet::new();asset_keys(&report,&mut keys);
    if let Some(manifest)=manifest{
        let mut assets=embed_referenced(&load_assets(manifest)?,&keys)?;
        if let Some(warnings)=assets.as_object_mut().unwrap().remove("warnings"){
            report["warnings"].as_array_mut().unwrap().extend(warnings.as_array().unwrap().iter().cloned());
        }
        report["assets"]=assets;
    }else if !keys.is_empty(){
        report["warnings"].as_array_mut().unwrap().push(json!(format!("No artwork manifest supplied: {} referenced keys use labels",keys.len())));
    }
    let data=json_for_html(&report)?;
    let scripts=[include_str!("../viewer/model.js"),include_str!("../viewer/dom.js"),include_str!("../viewer/combat.js"),
        include_str!("../viewer/screens.js"),include_str!("../viewer/viewer.js")].join("\n");
    // Walk only the maintained template, never substitute within untrusted data.
    for (index,part) in include_str!("../viewer/viewer.html").split("@@").enumerate(){
        let text=if index%2==0{part}else{match part{
            "REPORT_JSON"=>&data,"VIEWER_CSS"=>include_str!("../viewer/viewer.css"),"VIEWER_JS"=>&scripts,
            _=>return Err(ReportError::Invalid("unknown maintained template marker".into())),
        }};
        out.write_all(text.as_bytes())?;
    }
    out.flush()?;Ok(())
}
