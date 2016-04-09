use iron::prelude::*;

use std::io;
use std::string::*;
use std::path::PathBuf;
use std::env;
use std::fs;
use rustc_serialize::{Encodable, Encoder};

use mopa::Any;

use script_handlers::powershell::PowerShellScript;

static IMMEDIATE_RET_PATH: &'static str = "ret_immediately";

pub trait Script : Send + Sync + Any {
    fn get_name(&self) -> &str;
    fn get_relative_path(&self) -> &str;
    fn get_extension(&self) -> &str;
    fn get_full_path(&self) -> io::Result<PathBuf>;
    fn run(&self) -> IronResult<Response>;
}
mopafy!(Script);

// Implements encoding for Script trait objects, and adds a tag indicating what script type they are, for when we deserialize them
impl Encodable for Box<Script> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        if let Some(script) = self.downcast_ref::<PowerShellScript>() {
            script.encode(s)
        } else {
            panic!("Unknown concrete script type.")
        }
    }
}

pub fn construct_script(name: String, path: String, extension: String) -> Option<Box<Script>> {
    let script_kind = get_type_kind_for_ext(&extension);
    match script_kind {
        ScriptKind::PowerShell => Some(Box::new(PowerShellScript::new(name, path, extension))),
        ScriptKind::Unknown => None,
    }
}

#[derive(RustcEncodable, Debug)]
pub enum ScriptKind {
    PowerShell,
    Unknown,
}

pub fn get_type_kind_for_ext(string: &str) -> ScriptKind {
    match string {
        "ps1" => ScriptKind::PowerShell,
        "" => ScriptKind::Unknown,
        _ => ScriptKind::Unknown,
    }
}

pub fn generic_get_full_path<T: Script>(script: &T) -> io::Result<PathBuf> {
    let current_exe = try!(env::current_exe());
    match current_exe.parent() {
        Some(parent_dir) => {
            let script_path = parent_dir.join("scripts")
                                        .join(format!("{}{}",
                                                      script.get_name(),
                                                      script.get_extension()));
            // check in scripts dir
            if fs::metadata(&script_path).is_ok() {
                Ok(script_path)
            } else {
                // check in subdir
                let script_path = parent_dir.join("scripts")
                                            .join(IMMEDIATE_RET_PATH)
                                            .join(format!("{}{}",
                                                          script.get_name(),
                                                          script.get_extension()));
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
