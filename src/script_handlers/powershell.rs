use json_structs;
use json_structs::{Script, ScriptKind};

use std::io;
use std::path::PathBuf;
use std::process::Command;

use iron::prelude::*;
use iron::status;

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
            script_kind: json_structs::get_type_kind_for_ext(&e),
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
        json_structs::generic_get_full_path::<PowerShellScript>(self)
    }

    fn run(&self) -> IronResult<Response> {
        let full_path = self.get_full_path();
        if full_path.is_err() {
            return script_error_handler();
        }
        let full_path = full_path.unwrap();
        let output = Command::new("powershell.exe")
                         .arg("-executionpolicy")
                         .arg("bypass")
                         .arg("-File")
                         .arg(full_path)
                         .output();
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
}

fn script_error_handler() -> IronResult<Response> {
    Ok(Response::with(status::InternalServerError))
}