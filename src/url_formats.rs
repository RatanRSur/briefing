use std::collections::HashMap;
use strfmt::strfmt;

use crate::Upgrade;

pub fn get_release_notes_url(upgrade: &Upgrade) -> String {
    let url_formats: HashMap<&str, &str> = [(
        "git",
        "https://raw.githubusercontent.com/git/git/master/Documentation/RelNotes/{version}.txt",
    )]
    .iter()
    .cloned()
    .collect();

    let format_args: HashMap<String, &String> = [(String::from("version"), &upgrade.new_version)]
        .iter()
        .cloned()
        .collect();

    url_formats
        .get(&upgrade.package_name.as_ref())
        .map(|format_str| strfmt(format_str, &format_args).unwrap_or(String::from("")))
        .unwrap_or(String::from(""))
}
