extern crate iron;
#[macro_use]
extern crate router;
extern crate rustc_serialize;
extern crate walkdir;
#[macro_use]
extern crate horrorshow;
#[macro_use]
extern crate mopa;
extern crate docopt;
extern crate urlencoded;

mod scripts;
mod mainpage_generator;
mod handlers;
mod constants;

use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use router::Router;
use urlencoded::UrlEncodedQuery;

use walkdir::WalkDir;

use docopt::Docopt;

use std::env;
use std::io;
use std::thread;
use std::time::Duration;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use rustc_serialize::json;
use std::str::FromStr;

use scripts::Script;
use mainpage_generator::MainPageHtml;

static DEFAULT_SERVER_HOSTNAME: &'static str = "localhost:3000";

#[cfg_attr(rustfmt, rustfmt_skip)]
const USAGE: &'static str = "
Articulator
A small server program that can run arbitrary scripts on the hosting server.

Usage:
    articulator.exe [<hostname>]
    articulator.exe (-h | --help)
    
Options:
    hostname        (Optional) An IPv4-compatible hostname string.
    -h --help       Show this screen.    
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_hostname: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                         .and_then(|d| d.decode())
                         .unwrap_or_else(|e| {                             
                             e.exit();
                         });

    let hostname = match args.arg_hostname {
        Some(name) => {
            println!("Staring server on {}", &name);
            name
        }
        None => {
            println!("Starting server on {}", &DEFAULT_SERVER_HOSTNAME);
            String::from(DEFAULT_SERVER_HOSTNAME)
        }
    };                
    
    let mut router = Router::new();
    router.get("/", show_mainpage_handler, "show_mainpage_handler");
    router.get("/scr", show_scripts_handler, "show_scripts_handler");
    router.get("/scr/:scriptName", script_handler, "script_handler");
    router.get("/scr/:scriptName/?arg=:argValue", script_with_args_handler, "script_with_args_handler");

    let chain = Chain::new(router);

    Iron::new(chain).http(hostname.as_str()).unwrap();
}

fn show_mainpage_handler(_: &mut Request) -> IronResult<Response> {    
    if let Ok(scripts) = get_script_list() {
        let mainpage = MainPageHtml::new(scripts);
        let content_type = "text/html".parse::<Mime>().unwrap();
        return Ok(Response::with((content_type, status::Ok, mainpage.html_string)));
    } else {
        script_error_handler()
    }
}

fn show_scripts_handler(_: &mut Request) -> IronResult<Response> {
    println!("Getting scripts...");
    if let Ok(scripts) = get_script_list() {
        let scripts = json::encode(&scripts).unwrap();
        Ok(Response::with((status::Ok, scripts)))
    } else {
        script_error_handler()
    }
}

fn script_handler(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>().unwrap().find("scriptName").unwrap_or("/");
    let script = get_script(query);
    if script.is_err() {
        return script_error_handler();
    }
    let script = script.unwrap();
    let script_path = script.get_full_path();

    match script_path {
        Ok(path) => {
            if path.to_string_lossy().contains(constants::IMMEDIATE_RET_PATH) {                
                run_early_return_script(script)
            } else {
                script.run()
            }
        }
        Err(_) => {
            println!("Could not find script.");
            script_error_handler()
        } 
    }
}

fn script_with_args_handler(req: &mut Request) -> IronResult<Response> {    
    let arg = match req.get::<UrlEncodedQuery>(){
        Ok(mut hashmap) => {
            hashmap.remove("arg").unwrap()
        },
        Err(_) => vec![String::from("")]
    };
    let arg = &arg[0].as_str();
    let script_name = req.extensions.get::<Router>().unwrap().find("scriptName").unwrap_or("/");
    let script = get_script(script_name);
    if script.is_err() {
        return script_error_handler();
    }
    let script = script.unwrap();
    let script_path = script.get_full_path();

    match script_path {
        Ok(path) => {
            if path.to_string_lossy().contains(constants::IMMEDIATE_RET_PATH) {                
                run_early_return_script_with_args(script, arg)
            } else {
                script.run_with_arg(arg.to_string())
            }
        }
        Err(_) => {
            println!("Could not find script.");
            script_error_handler()
        } 
    }
}

fn run_early_return_script(script: Box<Script>) -> IronResult<Response> {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        script.run().ok();
    });
    Ok(Response::with((status::Ok, "Attempted to kick off early-return script.")))
}

fn run_early_return_script_with_args(script: Box<Script>, args: &str) -> IronResult<Response> {    
    let string_arg = args.to_string();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        script.run_with_arg(string_arg).ok();
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

// todo: fix unwrap infestation in here
#[cfg_attr(rustfmt, rustfmt_skip)]
fn get_script_list() -> io::Result<Vec<Box<Script>>> {
    let folder = try!(get_script_folder());
    let paths = WalkDir::new(&folder);

    let mut scripts = Vec::new();
    for p in paths {
        let p: walkdir::DirEntry = p.unwrap();
        if !p.file_type().is_file() {
            continue;
        };
        let split_str = String::from(p.file_name().to_str().unwrap());
        let split_str: Vec<&str> = split_str.split('.').collect();
        let name = *split_str.first().unwrap();

        let mut path = PathBuf::new();
        path.push(p.path());
        let path_root = Path::new(path.strip_prefix(folder.as_path().parent().unwrap())
                                      .unwrap());
        let rel_path = String::from(path_root.to_str().unwrap()).replace("\\", "/");

        let path_ext = p.path()
                        .extension()
                        .unwrap_or(OsStr::new(""))
                        .to_str()
                        .unwrap_or("");
        if path_ext == "" {
            if let Some(boxed_script) = scripts::construct_script_binary(String::from_str(name).unwrap(),
                                                                         String::from(rel_path),
                                                                         p.path()) {
                scripts.push(boxed_script);
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      format!("Unable to determine file type for script {:?}",
                                              p.path())));
            }           
        } else if let Some(boxed_script) = scripts::construct_script(String::from_str(name).unwrap(),
                                                              String::from(rel_path),
                                                              String::from(path_ext)) {
            scripts.push(boxed_script);
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      format!("Unable to determine file type for script {:?}",
                                              p.path())));
        }
    }
    return Ok(scripts);
}

fn get_script(name: &str) -> io::Result<Box<Script>> {
    let script_list: Vec<Box<Script>> = try!(get_script_list());
    if let Some(script) = script_list.into_iter().find(|s| s.get_name() == name) {
        Ok(script)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound,
                           format!("Script with name {} not found.", name)))
    }
}