use crate::ReportError;
use serde_json::{Map,Value,json};
use std::{collections::{BTreeMap,BTreeSet},fs,io::Read,path::{Component,Path,PathBuf}};

const MAX_IMAGE_BYTES:u64=16*1024*1024;
pub struct AssetCatalog {root:fs::File,entries:BTreeMap<String,Asset>,attribution:Vec<String>}
struct Asset {path:PathBuf,source:Value}

fn invalid(message:&str)->ReportError {ReportError::Invalid(message.into())}

/// Only passive local images; this module never acquires artwork or opens a URL.
pub fn load_assets(manifest:&Path)->Result<AssetCatalog,ReportError>{
    let manifest=manifest.canonicalize()?;
    let root=open_directory(manifest.parent().ok_or_else(||invalid("manifest needs a directory"))?)?;
    let name=manifest.file_name().ok_or_else(||invalid("manifest needs a filename"))?;
    let mut bytes=Vec::new();open_relative(&root,Path::new(name))?.take(MAX_IMAGE_BYTES+1).read_to_end(&mut bytes)?;
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
        validate_relative(&path)?;
        match open_relative(&catalog.root,&path) {
            Ok(_)=>{},
            Err(ReportError::Io(error)) if error.kind()==std::io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error),
        }
        let source=entry.get("source").ok_or_else(||invalid("missing asset source"))?;
        if !source.is_null() && !source.is_string(){return Err(invalid("asset source must be text or null"));}
        catalog.entries.insert(key.clone(),Asset{path,source:source.clone()});
    }
    Ok(catalog)
}

fn validate_relative(path:&Path)->Result<(),ReportError>{
    if path.as_os_str().is_empty() || path.components().any(|part|!matches!(part,Component::Normal(_))) {
        return Err(invalid("asset path must be relative without traversal"));
    }
    Ok(())
}

#[cfg(unix)]
fn open_child(directory:&fs::File,name:&std::ffi::OsStr,is_directory:bool)->Result<fs::File,ReportError>{
    use std::os::{fd::{AsRawFd,FromRawFd},unix::ffi::OsStrExt};
    let name=std::ffi::CString::new(name.as_bytes()).map_err(|_|invalid("asset path contains NUL"))?;
    let flags=libc::O_RDONLY|libc::O_CLOEXEC|libc::O_NOFOLLOW|libc::O_NONBLOCK
        |if is_directory {libc::O_DIRECTORY}else{0};
    // SAFETY: directory is a live owned descriptor, name is a live NUL-terminated
    // single component, and O_CREAT is absent, so no variadic mode is required.
    let fd=unsafe {libc::openat(directory.as_raw_fd(),name.as_ptr(),flags)};
    if fd<0 {return Err(std::io::Error::last_os_error().into());}
    // SAFETY: a successful openat returned a fresh descriptor owned by this call.
    Ok(unsafe {fs::File::from_raw_fd(fd)})
}

#[cfg(unix)]
fn open_directory(path:&Path)->Result<fs::File,ReportError>{
    use std::os::unix::fs::OpenOptionsExt;
    if !path.is_absolute(){return Err(invalid("manifest directory must be absolute"));}
    let mut directory=fs::OpenOptions::new().read(true)
        .custom_flags(libc::O_DIRECTORY|libc::O_CLOEXEC|libc::O_NOFOLLOW).open("/")?;
    for component in path.components() {
        match component {Component::RootDir=>{},Component::Normal(name)=>directory=open_child(&directory,name,true)?,
            _=>return Err(invalid("invalid manifest directory")),}
    }
    Ok(directory)
}

#[cfg(unix)]
fn open_relative(root:&fs::File,path:&Path)->Result<fs::File,ReportError>{
    validate_relative(path)?;
    let mut directory=root.try_clone()?;let mut parts=path.components().peekable();
    while let Some(Component::Normal(name))=parts.next() {
        let more=parts.peek().is_some();let file=open_child(&directory,name,more)?;
        if more {directory=file;}else {
            if !file.metadata()?.is_file(){return Err(invalid("artwork must be a regular file"));}
            return Ok(file);
        }
    }
    Err(invalid("empty asset path"))
}

// No path-check/open fallback on unsupported hosts: no-art export still works.
#[cfg(not(unix))]
fn open_directory(_:&Path)->Result<fs::File,ReportError>{Err(invalid("secure local artwork attachment requires a Unix host"))}
#[cfg(not(unix))]
fn open_relative(_:&fs::File,_:&Path)->Result<fs::File,ReportError>{Err(invalid("secure local artwork attachment requires a Unix host"))}

pub fn embed_referenced(catalog:&AssetCatalog,keys:&BTreeSet<String>)->Result<Value,ReportError>{
    let mut by_key=Map::new();let mut blobs=Map::new();let mut sources=Map::new();let mut warnings=Vec::new();
    let mut unique:Vec<Vec<u8>>=Vec::new();
    for key in keys {
        let Some(asset)=catalog.entries.get(key) else {warnings.push(format!("No artwork entry for {key}"));continue;};
        validate_relative(&asset.path)?;
        #[cfg(test)]
        BEFORE_ASSET_OPEN.with(|hook| {let callback=hook.borrow_mut().take();if let Some(callback)=callback{callback();}});
        // Every component is opened relative to a pinned directory descriptor;
        // O_NOFOLLOW binds rejection to the actual open, including parent swaps.
        let file=match open_relative(&catalog.root,&asset.path) {
            Ok(file)=>file,
            Err(ReportError::Io(error)) if error.kind()==std::io::ErrorKind::NotFound=>{warnings.push(format!("Missing local artwork for {key}"));continue;},
            Err(error)=>return Err(error),
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

// Deterministic filesystem replacement at the real validation/open boundary.
// This seam and its callbacks do not exist in production builds.
#[cfg(test)]
thread_local! {static BEFORE_ASSET_OPEN:std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const {std::cell::RefCell::new(None)};}
#[cfg(all(test,unix))]
#[path="../tests/common/mod.rs"]
mod test_common;
#[cfg(all(test,unix))]
mod race_tests {
    use super::*;
    use super::test_common as common;
    use std::os::unix::fs::symlink;
    const JPEG:&[u8]=b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00\xff\xd9";
    fn key()->BTreeSet<String>{BTreeSet::from(["card/strike".into()])}

    #[test]
    fn assets_leaf_symlink_swap_before_open_is_rejected() {
        let root=common::Scratch::new();let outside=common::Scratch::new();
        fs::write(root.file("one.png"),common::PNG).unwrap();fs::write(outside.file("private.png"),JPEG).unwrap();
        let manifest=common::manifest(&root,json!({"card/strike":{"path":"one.png","source":null}}));
        let catalog=load_assets(&manifest).unwrap();let inside=root.0.clone();let private=outside.file("private.png");
        BEFORE_ASSET_OPEN.with(|hook|*hook.borrow_mut()=Some(Box::new(move|| {
            fs::rename(inside.join("one.png"),inside.join("original.png")).unwrap();symlink(private,inside.join("one.png")).unwrap();
        })));
        assert!(embed_referenced(&catalog,&key()).is_err(),"a checked filename must not become an outside image");
    }

    #[test]
    fn assets_parent_symlink_swap_before_open_is_rejected() {
        let root=common::Scratch::new();let outside=common::Scratch::new();fs::create_dir(root.file("art")).unwrap();
        fs::write(root.file("art/one.png"),common::PNG).unwrap();fs::write(outside.file("one.png"),JPEG).unwrap();
        let manifest=common::manifest(&root,json!({"card/strike":{"path":"art/one.png","source":null}}));
        let catalog=load_assets(&manifest).unwrap();let inside=root.0.clone();let external=outside.0.clone();
        BEFORE_ASSET_OPEN.with(|hook|*hook.borrow_mut()=Some(Box::new(move|| {
            fs::rename(inside.join("art"),inside.join("original-art")).unwrap();symlink(external,inside.join("art")).unwrap();
        })));
        assert!(embed_referenced(&catalog,&key()).is_err(),"no-follow must protect intermediate directories, not only the leaf");
    }

    #[test]
    fn assets_loaded_root_cannot_be_rebound_by_path_replacement() {
        let outer=common::Scratch::new();let outside=common::Scratch::new();let root=outer.file("catalog");fs::create_dir(&root).unwrap();
        fs::write(root.join("one.png"),common::PNG).unwrap();fs::write(outside.file("one.png"),JPEG).unwrap();
        let manifest=root.join("assets.json");fs::write(&manifest,serde_json::to_vec(&json!({"schemaVersion":1,"assets":{"card/strike":{"path":"one.png","source":null}},"attribution":[]})).unwrap()).unwrap();
        let catalog=load_assets(&manifest).unwrap();let old=outer.file("original-catalog");let external=outside.0.clone();
        BEFORE_ASSET_OPEN.with(|hook|*hook.borrow_mut()=Some(Box::new(move|| {fs::rename(&root,old).unwrap();symlink(external,root).unwrap();})));
        if let Ok(value)=embed_referenced(&catalog,&key()) {
            assert_eq!(value["blobs"].as_object().unwrap().len(),1);
            assert!(value["blobs"].as_object().unwrap().values().all(|v|v.as_str().unwrap().starts_with("data:image/png;")),"only the original directory's PNG may be embedded");
        }
    }
}
