use script_runner::ScriptRunner;
use json_structs::Script;

use std::io;
use std::path::PathBuf;
use iron::prelude::*;

impl ScriptRunner for Script{
    fn get_path(script_name: &str) -> io::Result<PathBuf> {
        
    }
    fn run(path: PathBuf) -> IronResult<Response> {
        
    }
}