use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::process::Command;

use chrono::naive::NaiveDateTime;
use regex::Regex;

mod url_formats;

#[derive(Debug)]
pub struct Upgrade {
    timestamp: NaiveDateTime,
    package_name: String,
    old_version: String,
    new_version: String,
}

fn extract_data(s: &str, regex: &Regex) -> Option<Upgrade> {
    let maybe_line_captures = regex.captures(s);
    maybe_line_captures.map(|caps| Upgrade {
        timestamp: NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M").unwrap(),
        package_name: caps[2].to_string(),
        old_version: caps[3].to_string(),
        new_version: caps[4].to_string(),
    })
}

fn main() -> io::Result<()> {
    let last_briefing =
        NaiveDateTime::parse_from_str("2018-12-01 00:00", "%Y-%m-%d %H:%M").unwrap();

    let installed_packages_output = String::from_utf8(
        Command::new("/usr/bin/pacman")
            .arg("--query")
            .arg("--quiet")
            .arg("--explicit")
            .output()
            .expect("failed to execute process")
            .stdout,
    )
    .unwrap();

    let installed_packages: HashSet<&str> = installed_packages_output.split_whitespace().collect();

    let upgraded_packages = {
        let f = BufReader::new(File::open("/var/log/pacman.log")?);

        let regex = Regex::new(
            r"^\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2})\] \[ALPM\] upgraded ([^ ]*) \((.+) -> ([^-]+)(-\d+)?\)$",
        )
        .unwrap();

        f.lines()
            .filter_map(|result_str| result_str.ok().and_then(|s| extract_data(&s, &regex)))
            .skip_while(|upgrade| upgrade.timestamp < last_briefing)
            .filter(|upgrade| installed_packages.contains(&upgrade.package_name.as_ref()))
            .collect::<Vec<Upgrade>>()
    };

    let upgraded_package_urls: Vec<String> = {
        let command_output = String::from_utf8(
            Command::new("/usr/bin/pacman")
                .arg("--query")
                .arg("--info")
                .args(
                    &upgraded_packages
                        .iter()
                        .map(|upgrade| &upgrade.package_name)
                        .collect::<Vec<&String>>(),
                )
                .output()
                .expect("failed to execute process")
                .stdout,
        )
        .unwrap();

        let regex = Regex::new(r"\nURL *: (.*)\n").unwrap();

        regex
            .captures_iter(&command_output)
            .map(|captures| String::from(&captures[1]))
            .collect()
    };

    //println!("{}", upgraded_package_urls);
    for (upgrade, _url) in upgraded_packages.iter().zip(upgraded_package_urls.iter()) {
        println!(
            "{}: {}",
            upgrade.package_name,
            url_formats::get_release_notes_url(&upgrade)
        );
    }

    Ok(())
}
