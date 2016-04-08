extern crate iron;
#[macro_use]
extern crate router;
extern crate logger;
extern crate rustc_serialize;
extern crate walkdir;

use iron::prelude::*;
use iron::status;
use router::Router;
use logger::Logger;

use walkdir::WalkDir;

use std::process::Command;
use std::env;
use std::io;
use std::fs;
use std::thread;
use std::time::Duration;
use std::path::{Path, PathBuf};
use rustc_serialize::json;
use std::str::FromStr;

mod json_structs;
use json_structs::Script;

static IMMEDIATE_RET_PATH : &'static str = "ret_immediately";

fn main() {    
    let (logger_before, logger_after) = Logger::new(None);
    let router = router!(get "/" => show_scripts_handler,                          
                         get "/scr/:scriptName" => script_handler);

    let mut chain = Chain::new(router);

    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("10.6.1.25:3000").unwrap();
}

// todo: eliminate the unwrap-infestation in here
fn show_scripts_handler(_: &mut Request) -> IronResult<Response> {
    println!("Getting scripts...");
    match get_script_folder() {
        Ok(folder) => {
            let paths = WalkDir::new(&folder);

            let mut scripts = Vec::new();
            for p in paths {                
                let p: walkdir::DirEntry = p.unwrap();
                if !p.file_type().is_file() {
                    continue
                };
                let split_str = String::from(p.file_name().to_str().unwrap());
                let split_str: Vec<&str> = split_str.split('.').collect();
                let name = *split_str.first().unwrap();
                let mut path = PathBuf::new();
                path.push(p.path());
                let path_root = Path::new(path.strip_prefix(folder.as_path().parent().unwrap()).unwrap());
                let path = String::from(path_root.to_str().unwrap()).replace("\\", "/");
                scripts.push(Script::new(String::from_str(name).unwrap(), String::from(path)));
            }
            let scripts = json::encode(&scripts).unwrap();
            Ok(Response::with((status::Ok, scripts)))
        }
        Err(_) => script_error_handler(),
    }
}

fn script_handler(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>().unwrap().find("scriptName").unwrap_or("/");
    let script_path = get_script_path(query);    

    match script_path {
        Ok(path) => {
            if path.to_string_lossy().contains(IMMEDIATE_RET_PATH) {
                run_early_return_script(path)
            }
            else {
                run_normal_script(path)
            }            
        }
        Err(_) => {
            println!("Could not find script.");
            script_error_handler()
        } 
    }
}

fn run_normal_script(path: PathBuf) -> IronResult<Response> {
    let output = Command::new("powershell.exe")
                             .arg("-executionpolicy")
                             .arg("bypass")
                             .arg("-File")
                             .arg(path)
                             .output();
            match output {
                Ok(output) => {
                    match output.status.success() {
                        true => {
                            let script_output = String::from_utf8_lossy(&output.stdout)
                                                    .into_owned();
                            println!("Script success. Output:\n{}", script_output);
                            Ok(Response::with((status::Ok, script_output)))
                        }
                        false => {
                            println!("{}", String::from_utf8_lossy(&output.stderr).into_owned());
                            script_error_handler()
                        }
                    }
                }
                Err(_) => {
                    println!("No output from PowerShell script.");
                    script_error_handler()
                }                
            }
}

fn run_early_return_script(path: PathBuf) -> IronResult<Response> {
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(500));
        run_normal_script(path).ok(); //explicitly ignoring the Result. Can't do anything about it at this point
    });
    Ok(Response::with((status::Ok, "Attempted to kick off early-return script.")))
}


fn script_error_handler() -> IronResult<Response> {
    Ok(Response::with(status::InternalServerError))
}

fn get_script_folder() -> io::Result<PathBuf> {
    let current_exe = try!(env::current_exe());
    match current_exe.parent() {
        Some(parent_dir) => Ok(parent_dir.join("scripts")),
        None => Err(io::Error::new(io::ErrorKind::NotFound, "Unable to find scripts folder.")),
    }
}

fn get_script_path(script_name: &str) -> io::Result<PathBuf> {
    let current_exe = try!(env::current_exe());
    match current_exe.parent() {
        Some(parent_dir) => {
            let script_path = parent_dir.join("scripts").join(format!("{}.ps1", script_name)); 
            //check in scripts dir
            if fs::metadata(&script_path).is_ok() {
                Ok(script_path)
            }
            else {
                let script_path = parent_dir.join("scripts").join(IMMEDIATE_RET_PATH).join(format!("{}.ps1", script_name));
                if fs::metadata(&script_path).is_ok() {
                    Ok(script_path)
                }
                else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Unable to generate script path."))
                }
            }
        },
        None => Err(io::Error::new(io::ErrorKind::NotFound, "Unable to generate script path.")),
    }
}
