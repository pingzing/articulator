use horrorshow::prelude::*;
use horrorshow::helper::doctype;
use scripts::Script;

pub struct MainPageHtml {
    pub html_string: String,
}

impl MainPageHtml {
    pub fn new(scripts: Vec<Box<Script>>) -> MainPageHtml {
        MainPageHtml {
            html_string: html! {
                : doctype::HTML;
                html {
                    head {
                        meta(name="viewport", content="width=device-width, initial-scale=1.0");                     
                        title : "Runnable scripts";
                    }
                    body(style="display: table") {
                        h1(id="main_header", style="display: table") {
                            : "Runnable Scripts"
                        }
                        h2(id="scripts_header", style="display: table") {
                            : "Scripts"
                        }
                        ul(id="normal_scripts_list", style="display: table") {
                            @ for script in scripts {
                                li {
                                    : Raw(format!("<a href=\"/scr/{}\">{}</a>", script.get_name(), script.get_name()))
                                }
                            }
                        }
                    }
                }
            }.into_string().unwrap()
        }
    }
}
