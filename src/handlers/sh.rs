use scripts;
use scripts::{Script, ScriptKind};

use std::io;
use std::path::PathBuf;
use std::process::Command;

use iron::prelude::*;

#[derive(RustcEncodable, Debug)]
pub struct ShellScript {
    name: String,
    relative_path: String,
    script_kind: ScriptKind,
}

impl ShellScript {
    pub fn new(n: String, p: String, e: String) -> ShellScript {
        ShellScript {
            name: n,
            relative_path: p,
            script_kind: scripts::get_type_kind_for_ext(&e),
        }
    }
}

impl Script for ShellScript {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_relative_path(&self) -> &str {
        &self.relative_path
    }

    fn get_extension(&self) -> &str {
        ".sh"
    }

    fn get_full_path(&self) -> io::Result<PathBuf> {
        scripts::generic_get_full_path::<ShellScript>(self)
    }

    fn run(&self) -> IronResult<Response> {
        let full_path = self.get_full_path();
        if full_path.is_err() {
            return scripts::generic_error_handler();
        }

        let full_path = full_path.unwrap();
        let output = Command::new("sh")
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
        let output = Command::new("sh")
                         .arg(full_path)
                         .arg(arg)
                         .output();

        return scripts::generic_run(output);
    }
}
