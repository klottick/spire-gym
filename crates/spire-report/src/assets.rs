use crate::ReportError;
use serde_json::{Map,Value,json};
use std::{collections::{BTreeMap,BTreeSet},fs,io::Read,path::{Component,Path,PathBuf}};

const MAX_IMAGE_BYTES:u64=16*1024*1024;
pub struct AssetCatalog {root:PathBuf,entries:BTreeMap<String,Asset>,attribution:Vec<String>}
struct Asset {path:PathBuf,source:Value}

fn invalid(message:&str)->ReportError {ReportError::Invalid(message.into())}

/// Only passive local images; this module never acquires artwork or opens a URL.
pub fn load_assets(manifest:&Path)->Result<AssetCatalog,ReportError>{
    let manifest=manifest.canonicalize()?;
    let root=manifest.parent().ok_or_else(||invalid("manifest needs a directory"))?.to_path_buf();
    let mut bytes=Vec::new();fs::File::open(manifest)?.take(MAX_IMAGE_BYTES+1).read_to_end(&mut bytes)?;
    if bytes.len() as u64>MAX_IMAGE_BYTES {return Err(invalid("asset manifest too large"));}
    let value:Value=serde_json::from_slice(&bytes)?;
    if value["schemaVersion"].as_u64()!=Some(1) {return Err(invalid("unsupported asset manifest version"));}
    let entries=value["assets"].as_object().ok_or_else(||invalid("asset entries must be an object"))?;
    let mut catalog=AssetCatalog{root,entries:BTreeMap::new(),attribution:vec!["Original Slay the Spire artwork belongs to Mega Crit Games.".into()]};
    for text in value["attribution"].as_array().ok_or_else(||invalid("attribution must be an array"))? {
        catalog.attribution.push(text.as_str().ok_or_else(||invalid("attribution must be text"))?.into());
    }
    for (key,entry) in entries {
        if key.is_empty(){return Err(invalid("empty asset key"));}
        let path=PathBuf::from(entry["path"].as_str().ok_or_else(||invalid("asset path must be text"))?);
        safe_path(&catalog.root,&path)?;
        let source=entry.get("source").ok_or_else(||invalid("missing asset source"))?;
        if !source.is_null() && !source.is_string(){return Err(invalid("asset source must be text or null"));}
        catalog.entries.insert(key.clone(),Asset{path,source:source.clone()});
    }
    Ok(catalog)
}

fn safe_path(root:&Path,path:&Path)->Result<PathBuf,ReportError>{
    if path.as_os_str().is_empty() || path.components().any(|part|!matches!(part,Component::Normal(_))) {
        return Err(invalid("asset path must be relative without traversal"));
    }
    let mut current=root.to_path_buf();
    for part in path.components() {
        current.push(part.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(_) => {
                if !current.canonicalize()?.starts_with(root){return Err(invalid("asset symlink escapes manifest directory"));}
            }
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error.into()),
        }
    }
    match fs::metadata(&current) {
        Ok(metadata) if !metadata.is_file()=>return Err(invalid("artwork must be a regular file")),
        Ok(_)=>{},
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
        Err(error)=>return Err(error.into()),
    }
    Ok(current)
}

pub fn embed_referenced(catalog:&AssetCatalog,keys:&BTreeSet<String>)->Result<Value,ReportError>{
    let mut by_key=Map::new();let mut blobs=Map::new();let mut sources=Map::new();let mut warnings=Vec::new();
    let mut unique:Vec<Vec<u8>>=Vec::new();
    for key in keys {
        let Some(asset)=catalog.entries.get(key) else {warnings.push(format!("No artwork entry for {key}"));continue;};
        // Recheck paths at use time, not just while loading the manifest.
        let path=safe_path(&catalog.root,&asset.path)?;
        let file=match fs::File::open(&path) {
            Ok(file)=>file,
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{warnings.push(format!("Missing local artwork for {key}"));continue;},
            Err(error)=>return Err(error.into()),
        };
        if !file.metadata()?.is_file(){return Err(invalid("artwork must be a regular file"));}
        let mut bytes=Vec::new();file.take(MAX_IMAGE_BYTES+1).read_to_end(&mut bytes)?;
        if bytes.len() as u64>MAX_IMAGE_BYTES{return Err(invalid("artwork exceeds size limit"));}
        let mime=image_type(&bytes)?;
        let index=if let Some(index)=unique.iter().position(|old|old==&bytes){index}else{
            let index=unique.len();blobs.insert(format!("blob-{index}"),json!(format!("data:{mime};base64,{}",base64(&bytes))));unique.push(bytes);index
        };
        by_key.insert(key.clone(),json!(format!("blob-{index}")));sources.insert(key.clone(),asset.source.clone());
    }
    Ok(json!({"byKey":by_key,"blobs":blobs,"sources":sources,"attribution":catalog.attribution,"warnings":warnings}))
}

// Header/structure screening is not an image decoder. Browser decode failures still
// fall back to names. Never embed SVG, HTML or a caller-supplied MIME type.
fn image_type(bytes:&[u8])->Result<&'static str,ReportError>{
    if bytes.len()>=45 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") && &bytes[8..16]==b"\0\0\0\rIHDR"
        && bytes[16..20]!=[0,0,0,0] && bytes[20..24]!=[0,0,0,0]
        && bytes.ends_with(b"\0\0\0\0IEND\xaeB`\x82") {return Ok("image/png");}
    if bytes.len()>=20 && bytes.starts_with(b"\xff\xd8\xff") && bytes.ends_with(b"\xff\xd9"){return Ok("image/jpeg");}
    if bytes.len()>=20 && &bytes[..4]==b"RIFF" && &bytes[8..12]==b"WEBP"
        && matches!(&bytes[12..16],b"VP8 "|b"VP8L"|b"VP8X")
        && u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize+8==bytes.len(){return Ok("image/webp");}
    Err(invalid("artwork must have a valid passive PNG/JPEG/WebP header"))
}

fn base64(bytes:&[u8])->String{
    const ALPHABET:&[u8;64]=b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out=String::new();
    for chunk in bytes.chunks(3){
        let a=chunk[0];let b=chunk.get(1).copied().unwrap_or(0);let c=chunk.get(2).copied().unwrap_or(0);
        out.push(ALPHABET[(a>>2) as usize] as char);out.push(ALPHABET[(((a&3)<<4)|(b>>4)) as usize] as char);
        out.push(if chunk.len()>1{ALPHABET[(((b&15)<<2)|(c>>6)) as usize] as char}else{'='});
        out.push(if chunk.len()>2{ALPHABET[(c&63) as usize] as char}else{'='});
    }out
}
