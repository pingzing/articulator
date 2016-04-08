extern crate walkdir;

use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::fs;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let cargo_manifest_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let exe_dir = Path::new(out_dir.to_str().unwrap())
                      .parent().unwrap()
                      .parent().unwrap()
                      .parent().unwrap();
                      
    let prefix_path = Path::new(cargo_manifest_dir.to_str().unwrap());

    let mut script_folder = PathBuf::new();
    {
        script_folder.push(&cargo_manifest_dir);
        script_folder.push("scripts");
        println!("{}", script_folder.display());
    }
    
    let script_folder = script_folder;
    let dst = Path::new(exe_dir);        
    let script_files = WalkDir::new(script_folder);
    
    //create desination dir(s)
    match fs::create_dir_all(dst.join(Path::new("scripts").join(Path::new("ret_immediately")))) {
        Ok(_) => {
            for file in script_files {
                if file.is_ok() {            
                    let file = file.unwrap();
                    if !file.file_type().is_file() {continue;}
                    let src = &file.path();
                    let prefixless_dest = src.strip_prefix(prefix_path).unwrap();
                    let dst = Path::new(&dst).join(&prefixless_dest);
                    match fs::copy(&src, &dst) {
                        Ok(_) => println!("Copied script from {:?} to {:?}", src, dst),
                        Err(e) => println!("FAILED to copy script from {:?} to {:?}\nDetails: {:?}", src, dst, e)
                    }
                }
            }            
        }
        Err(e) => panic!("FAILED to create scripts folder in output directory.\nDetails: {:?}", e)
    }        
}   
