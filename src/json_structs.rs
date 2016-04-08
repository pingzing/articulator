use std::io;
use std::string::*;
use std::path::{Path,PathBuf};
use std::env;
use std::fs;

static IMMEDIATE_RET_PATH: &'static str = "ret_immediately";

pub trait Script {    
    fn get_name(&self) -> &str;
    fn get_relative_path(&self) -> &str;
    fn get_script_type() -> TypedScript; 
    fn get_extension(&self) -> &str; 
    fn get_full_path(&self) -> io::Result<PathBuf>;  
}

pub struct PowerShellScript {
    name : String,
    relative_path: String,    
}

#[derive(RustcEncodable, Debug)]
pub enum TypedScript {
    PowerShell(PowerShellScript),    
    Unknown
}

pub enum ScriptKind {
    PowerShell,
    Unknown
}

impl TypedScript {
    fn new(type_of_script: ScriptKind, name: String, path: String) -> TypedScript {
        match type_of_script {
            TypedScript::PowerShell => TypedScript::PowerShell::new(name, path)
        }
    }
    
    fn get_script_extension(&self) -> &str {
        match self {
            &TypedScript::PowerShell => ".ps1",
            &TypedScript::Unknown => ""
        }
    }
    
    fn get_type_kind_for_ext(string: &str) -> ScriptKind {
        match string {
            ".ps1" => ScriptKind::PowerShell,
            "" => ScriptKind::Unknown,
            _ => ScriptKind::Unknown
        }
    }
}

impl PowerShellScript {
    pub fn new(n: String, p: String) -> PowerShellScript {        
        return PowerShellScript { name: n, relative_path: p};
    }
}

impl Script for PowerShellScript {        
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    pub fn get_relative_path(&self) -> &str {
        &self.relative_path
    }
    
    pub fn get_extension(&self) -> &str {
        &self.script_type.get_script_extension()
    }
    
    pub fn get_full_path(&self) -> io::Result<PathBuf> {
    let current_exe = try!(env::current_exe());
    match current_exe.parent() {
        Some(parent_dir) => {
            let script_path = parent_dir.join("scripts")
                                        .join(format!("{}{}", self.get_name(), self.get_extension()));
            // check in scripts dir
            if fs::metadata(&script_path).is_ok() {
                Ok(script_path)
            } else {
                //check in subdir
                let script_path = parent_dir.join("scripts")
                                            .join(IMMEDIATE_RET_PATH)
                                            .join(format!("{}{}", self.get_name(), self.get_extension()));
                if fs::metadata(&script_path).is_ok() {
                    Ok(script_path)
                } else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Unable to generate script path."))
                }
            }
        }
        None => Err(io::Error::new(io::ErrorKind::NotFound, "Unable to generate script path.")),
    }
}
}
