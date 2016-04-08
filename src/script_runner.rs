use iron::prelude::*;
use std::env;
use std::io;
use std::fs;
use std::path::PathBuf;

static IMMEDIATE_RET_PATH: &'static str = "ret_immediately";

pub trait ScriptRunner {
    fn get_path(script_name: &str) ->  io::Result<PathBuf>;
    fn run(path: PathBuf) -> IronResult<Response>;
}