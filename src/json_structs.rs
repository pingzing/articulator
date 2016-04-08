use std::string::*;

#[derive(RustcEncodable, Debug)]
pub struct Script {
    pub name: String,
    pub path: String,
}

impl Script {
    pub fn new(n: String, p: String) -> Script {
        return Script { name: n, path: p };
    }
}
