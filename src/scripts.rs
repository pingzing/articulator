use iron::prelude::*;
use iron::status;

use std::io;
use std::string::*;
use std::path::{PathBuf, Path};
use std::env;
use std::fs;
use rustc_serialize::{Encodable, Encoder};
use std::process::Output;

use mopa::Any;

use handlers::powershell::PowerShellScript;
use handlers::python::PythonScript;
use handlers::sh::ShellScript;
use constants;

pub trait Script : Send + Sync + Any {
    fn get_name(&self) -> &str;
    fn get_relative_path(&self) -> &str;
    fn get_extension(&self) -> &str;
    fn get_full_path(&self) -> io::Result<PathBuf>;
    fn run(&self) -> IronResult<Response>;
}
mopafy!(Script);

// -- New script types must be added here for proper serialization
// Implements encoding for Script trait objects
impl Encodable for Box<Script> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        if let Some(script) = self.downcast_ref::<PowerShellScript>() {
            script.encode(s)
        } else if let Some(script) = self.downcast_ref::<PythonScript>() {
            script.encode(s)
        } else if let Some(script) = self.downcast_ref::<ShellScript>() {
            script.encode(s)
        } else {
            panic!("Unknown concrete script type.")
        }
    }
}

// -- New script types must be added here so they can be constructed
pub fn construct_script(name: String, path: String, extension: String) -> Option<Box<Script>> {
    let script_kind = get_type_kind_for_ext(&extension);
    match script_kind {
        ScriptKind::PowerShell => Some(Box::new(PowerShellScript::new(name, path, extension))),
        ScriptKind::Python => Some(Box::new(PythonScript::new(name, path, extension))),
        ScriptKind::Shell => Some(Box::new(ShellScript::new(name, path, extension))),
        ScriptKind::Unknown => None,
    }
}

pub fn construct_script_binary(name: String,
                               rel_path: String,
                               full_path: &Path)
                               -> Option<Box<Script>> {
    match check_is_executable(full_path) {
        true => None,//Some(Box::new(BinaryScript::new(name, path))),
        false => None,
    }
}

fn check_is_executable(path: &Path) -> bool {
    return false;
}

// -- New script types must be added here so they can be identified
#[derive(RustcEncodable, Debug)]
pub enum ScriptKind {
    PowerShell,
    Python,
    Shell,
    Unknown,
}

// -- New script types must be added here so we can associate an extension with a ScriptKind
pub fn get_type_kind_for_ext(string: &str) -> ScriptKind {
    match string {
        "ps1" => ScriptKind::PowerShell,
        "py" => ScriptKind::Python,
        "sh" => ScriptKind::Shell,
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
                                            .join(constants::IMMEDIATE_RET_PATH)
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

pub fn generic_run(output: io::Result<Output>) -> IronResult<Response> {
    match output {
        Ok(output) => {
            match output.status.success() {
                true => {
                    let script_output = String::from_utf8_lossy(&output.stdout).into_owned();
                    println!("Script success. Output:\n{}", script_output);
                    Ok(Response::with((status::Ok, script_output)))
                }
                false => {
                    println!("{}", String::from_utf8_lossy(&output.stderr).into_owned());
                    generic_error_handler()
                }
            }
        }
        Err(_) => {
            println!("No output from PowerShell script.");
            generic_error_handler()
        }                
    }
}

pub fn generic_error_handler() -> IronResult<Response> {
    Ok(Response::with(status::InternalServerError))
}
