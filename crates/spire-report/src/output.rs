use crate::ReportError;
use std::{fs::{self,File,OpenOptions},io::Write,path::{Path,PathBuf},sync::atomic::{AtomicU64,Ordering}};

/// A visible recoverable trace is never overwritten or removed by error cleanup.
pub fn create_trace(path:&Path)->Result<File,ReportError>{Ok(OpenOptions::new().write(true).create_new(true).open(path)?)}

static NEXT:AtomicU64=AtomicU64::new(0);
pub struct OutputReservation {file:File,temporary:PathBuf,destination:PathBuf}
impl OutputReservation {
    pub fn new(html:&Path)->Result<Self,ReportError>{
        match fs::symlink_metadata(html){
            Ok(_)=>return Err(ReportError::Invalid("HTML destination already exists".into())),
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error.into()),
        }
        let parent=html.parent().filter(|path|!path.as_os_str().is_empty()).unwrap_or_else(||Path::new("."));
        let name=html.file_name().ok_or_else(||ReportError::Invalid("HTML destination needs a filename".into()))?;
        loop{
            let temporary=parent.join(format!(".{}.{}-{}.tmp",name.to_string_lossy(),std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
            match OpenOptions::new().write(true).create_new(true).open(&temporary){
                Ok(file)=>return Ok(Self{file,temporary,destination:html.to_path_buf()}),
                Err(error) if error.kind()==std::io::ErrorKind::AlreadyExists=>continue,
                Err(error)=>return Err(error.into()),
            }
        }
    }
    pub fn writer(&mut self)->&mut File {&mut self.file}
    pub fn publish(mut self)->Result<(),ReportError>{
        self.file.flush()?;self.file.sync_all()?;
        // Same-directory hard-link creation is atomic and never replaces an existing
        // destination, including a concurrently created file or dangling symlink.
        fs::hard_link(&self.temporary,&self.destination)?;
        Ok(())
    }
}
impl Drop for OutputReservation {fn drop(&mut self){let _=fs::remove_file(&self.temporary);}}
