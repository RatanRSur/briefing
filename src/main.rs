use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::process::Command;
use std::env;

use regex::Regex;
use chrono::naive::NaiveDateTime;

#[derive(Debug)]
struct Upgrade {
    timestamp: NaiveDateTime,
    package_name: String,
    old_version: String,
    new_version: String
}

fn extract_data(s: &str, regex: &Regex) -> Option<Upgrade> {
    let maybe_line_captures = regex.captures(s);
    maybe_line_captures.map(|caps| {

        Upgrade { timestamp:    NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M").unwrap(),
                package_name: caps[3].to_string(),
                old_version:  caps[4].to_string(),
                new_version:  caps[5].to_string()
        }
    })
}

fn main() -> io::Result<()> {
    let f = BufReader::new(File::open("/var/log/pacman.log")?);

    let regex = Regex::new(
        r"^\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2})\] \[ALPM\] upgraded ([^ ]*) \((.+) -> ((.+))\)$"
    ).unwrap();

    let upgrades = f.lines()
        .filter_map(|result_str| result_str.map(|s| extract_data(&s, &regex)).unwrap());

    let installed_packages_output = String::from_utf8(Command::new("/usr/bin/pacman")
                                                .arg("-Qqe")
                                                .output()
                                                .expect("failed to execute process")
                                                .stdout)
                                    .unwrap();

    let installed_packages = installed_packages_output.split_whitespace();

    Ok(())
}
