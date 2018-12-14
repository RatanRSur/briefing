use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::process::Command;

use chrono::naive::NaiveDateTime;
use regex::Regex;

#[derive(Debug)]
struct Upgrade {
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
            .arg("-Qqe")
            .output()
            .expect("failed to execute process")
            .stdout,
    )
    .unwrap();

    let installed_packages: Vec<&str> = installed_packages_output.split_whitespace().collect();

    let f = BufReader::new(File::open("/var/log/pacman.log")?);

    let regex = Regex::new(
        r"^\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2})\] \[ALPM\] upgraded ([^ ]*) \((.+) -> ((.+))\)$",
    )
    .unwrap();

    let upgrades = f
        .lines()
        .filter_map(|result_str| result_str.map(|s| extract_data(&s, &regex)).unwrap())
        .skip_while(|upgrade| last_briefing < upgrade.timestamp)
        .filter(|upgrade| installed_packages.contains(&upgrade.package_name.as_ref()))
        .map(|upgrade| upgrade.package_name)
        .collect::<HashSet<String>>();

    upgrades.into_iter().for_each(|u| println!("{:?}", u));

    Ok(())
}
