use scripts;
use scripts::{Script, ScriptKind};

use std::io;
use std::path::PathBuf;
use std::process::Command;

use iron::prelude::*;

#[derive(RustcEncodable, Debug)]
pub struct BinaryScript {
    name: String,
    relative_path: String,
    script_kind: ScriptKind,
}

impl BinaryScript {
    pub fn new(n: String, p: String) -> BinaryScript {
        return BinaryScript {
            name: n,
            relative_path: p,
            script_kind: scripts::ScriptKind::Binary
        };
    }
}

impl Script for BinaryScript {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_relative_path(&self) -> &str {
        &self.relative_path
    }
    
    fn get_extension(&self) -> &str {
        get_platform_dependent_extension()
    }
    
    fn get_full_path(&self) -> io::Result<PathBuf> {
        scripts::generic_get_full_path::<BinaryScript>(self)
    }
    
    fn run(&self) -> IronResult<Response> {
        let full_path = self.get_full_path();
        if full_path.is_err() {
            return scripts::generic_error_handler();
        }
        
        let full_path = full_path.unwrap();
        let output = Command::new(full_path).output();
        return scripts::generic_run(output);
    }
}

#[cfg(target_family="windows")]
fn get_platform_dependent_extension<'a>() -> &'a str {
    ".exe"
}

#[cfg(target_family="unix")]
fn get_platform_dependent_extension<'a>() -> &'a str {
    ""
}
