use scripts;
use scripts::{Script, ScriptKind};

use std::io;
use std::path::PathBuf;
use std::process::Command;

use iron::prelude::*;

#[derive(RustcEncodable, Debug)]
pub struct PowerShellScript {
    name: String,
    relative_path: String,
    script_kind: ScriptKind,
}

impl PowerShellScript {
    pub fn new(n: String, p: String, e: String) -> PowerShellScript {
        return PowerShellScript {
            name: n,
            relative_path: p,
            script_kind: scripts::get_type_kind_for_ext(&e),
        };
    }
}

impl Script for PowerShellScript {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_relative_path(&self) -> &str {
        &self.relative_path
    }

    fn get_extension(&self) -> &str {
        ".ps1"
    }

    fn get_full_path(&self) -> io::Result<PathBuf> {
        scripts::generic_get_full_path::<PowerShellScript>(self)
    }
    
    fn run(&self) -> IronResult<Response> {
        let full_path = self.get_full_path();
        if full_path.is_err() {
            return scripts::generic_error_handler();
        }
        
        let full_path = full_path.unwrap();
        let output = Command::new("powershell.exe")
                         .arg("-executionpolicy")
                         .arg("bypass")
                         .arg("-File")
                         .arg(full_path)
                         .output();
        return scripts::generic_run(output);
    }

    fn run_with_arg(&self, arg: String) -> IronResult<Response> {
        let full_path = self.get_full_path();
        if full_path.is_err() {
            return scripts::generic_error_handler();
        }

        let full_path = full_path.unwrap();
        let output = Command::new("powershell.exe")
                         .arg("-executionpolicy")
                         .arg("bypass")
                         .arg("-File")
                         .arg(full_path)
                         .arg(arg)
                         .output();
        return scripts::generic_run(output);
    }
}