use horrorshow::prelude::*;
use json_structs::Script;

pub struct MainPageHtml {
    pub html_string: String,
}

impl MainPageHtml {
    pub fn new(scripts: Vec<Box<Script>>) -> MainPageHtml {
        MainPageHtml {
            html_string: html! {
                : raw!("<!DOCTYPE html>");
                html {
                    head {
                        title { : "Runnable scripts"}
                    }
                    body {
                        h1(id="main_header") {
                            : "Runnable Scripts"
                        }
                        h2(id="scripts_header") {
                            : "Scripts"
                        }
                        ul(id="normal_scripts_list") {
                            @ for script in scripts {
                                li {
                                    : raw!(format!("<a href=\"/scr/{}\">{}</a>", script.get_name(), script.get_name()))
                                }
                            }
                        }
                    }
                }
            }.into_string().unwrap()
        }
    }
}
