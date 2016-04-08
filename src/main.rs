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
use std::path::PathBuf;
use std::fs;
use std::fs::DirEntry;
use rustc_serialize::json;
use std::str::FromStr;

mod json_structs;
use json_structs::Script;

fn main() {    
    let (logger_before, logger_after) = Logger::new(None);
    let router = router!(get "/" => show_scripts_handler,                          
                         get "/scr/:scriptName" => script_handler);

    let mut chain = Chain::new(router);

    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("192.168.0.7:3000").unwrap();
}

// todo: eliminate the unwrap-infestation in here
fn show_scripts_handler(_: &mut Request) -> IronResult<Response> {
    println!("Getting scripts...");
    match get_script_folder() {
        Ok(folder) => {
            let paths = WalkDir::new(folder);

            let mut scripts = Vec::new();
            for p in paths {                
                let p: walkdir::DirEntry = p.unwrap();
                if !p.file_type().is_file() {continue};
                let split_str = String::from(p.file_name().to_str().unwrap());
                let split_str: Vec<&str> = split_str.split('.').collect();
                let name = *split_str.first().unwrap();
                let mut path = PathBuf::new();
                path.push(p.path());
                let path = path.strip_prefix(path.parent().unwrap().parent().unwrap()).unwrap();
                let path = String::from(path.to_str().unwrap()).replace("\\", "/");
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
        Err(_) => {
            println!("Could not find script.");
            script_error_handler()
        } 
    }
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
        Some(parent_dir) => Ok(parent_dir.join("scripts").join(format!("{}.ps1", script_name))),
        None => Err(io::Error::new(io::ErrorKind::NotFound, "Unable to generate script path.")),
    }
}
