use std::collections::HashMap;
use strfmt::strfmt;

use crate::Upgrade;

pub static RELEASE_NOTES_TEMPLATES: &'static [(&str, &str)] = &[(
    "git",
    "https://raw.githubusercontent.com/git/git/master/Documentation/RelNotes/{version}.txt",
)];

pub fn format_url(template: &String, version: &String) -> String {
    let format_args: HashMap<String, &String> = [(String::from("version"), version)]
        .iter()
        .cloned()
        .collect();

    strfmt(template, &format_args).unwrap()
}
