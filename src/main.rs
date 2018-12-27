use colored::*;
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::str::FromStr;

use std::process::Command;

use chrono::naive::NaiveDateTime;
use regex::Regex;

mod url_templates;

#[derive(Debug)]
pub struct Upgrade {
    timestamp: NaiveDateTime,
    package_name: String,
    old_version: String,
    new_version: String,
}

#[derive(Debug)]
pub enum ParseUpgradeError {
    Error,
}
impl Error for ParseUpgradeError {}
impl Display for ParseUpgradeError {
    fn fmt(&self, _f: &mut Formatter) -> std::fmt::Result {
        Ok(())
    }
}

impl FromStr for Upgrade {
    type Err = ParseUpgradeError;

    fn from_str(s: &str) -> Result<Upgrade, Self::Err> {
        lazy_static! {
            static ref upgrade_parse_regex: Regex = Regex::new(r"^\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2})\] \[ALPM\] upgraded ([^ ]*) \((.+) -> (\d:)?([^-+]+).*\)$",).unwrap();
        }

        let maybe_line_captures = upgrade_parse_regex.captures(s);
        maybe_line_captures
            .map(|caps| Upgrade {
                timestamp: NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M").unwrap(),
                package_name: caps[2].to_string(),
                old_version: caps[3].to_string(),
                new_version: caps[5].to_string(),
            })
            .ok_or(ParseUpgradeError::Error)
    }
}

#[derive(Debug)]
struct Package {
    name: String,
    home_page_url: String,
    url_template: Option<&'static str>,
}

fn main() -> io::Result<()> {
    let last_briefing =
        NaiveDateTime::parse_from_str("2018-12-01 00:00", "%Y-%m-%d %H:%M").unwrap();

    let installed_packages: HashMap<String, Package> = {
        let installed_packages_output = String::from_utf8(
            Command::new("/usr/bin/pacman")
                .arg("--query")
                .arg("--explicit")
                .arg("--info")
                .output()
                .expect("failed to execute process")
                .stdout,
        )
        .unwrap();

        let regex = Regex::new(r"(^|\n)(Name|URL) +: (.*)\n").unwrap();

        let mut packages = HashMap::new();
        let mut captures_iter = regex.captures_iter(&installed_packages_output);
        while let Some(captures) = captures_iter.next() {
            let package_name = String::from(&captures[3]);
            packages.insert(
                package_name.clone(),
                Package {
                    name: package_name.clone(),
                    home_page_url: captures_iter.next().unwrap()[3].to_string(),
                    url_template: url_templates::RELEASE_NOTES_TEMPLATES
                        .get(package_name.as_str())
                        .map(|&s| s),
                },
            );
        }
        packages
    };

    let upgrades_by_name = {
        let f = BufReader::new(File::open("/var/log/pacman.log")?);

        let installed_package_names: HashSet<_> = installed_packages
            .iter()
            .map(|name_and_package| name_and_package.0)
            .cloned()
            .collect();

        let mut accumulator: BTreeMap<String, Vec<Upgrade>> = BTreeMap::new();
        let upgrades = f
            .lines()
            .filter_map(|result_str| result_str.ok().and_then(|s| Upgrade::from_str(&s).ok()))
            .skip_while(|upgrade| upgrade.timestamp < last_briefing)
            .filter(|upgrade| installed_package_names.contains(&upgrade.package_name));

        for upgrade in upgrades {
            let vec = accumulator
                .entry(upgrade.package_name.clone())
                .or_insert(Vec::new());
            vec.push(upgrade);
        }

        accumulator
    };

    let max_upgrade_name_len = upgrades_by_name
        .keys()
        .map(|name| name.len())
        .max()
        .unwrap();

    for (package_name, upgrades) in upgrades_by_name {
        let package = installed_packages.get(&package_name).unwrap();
        print_with_margin(&package_name, max_upgrade_name_len);
        match package.url_template {
            Some(template) => {
                let versions = upgrades.iter().map(|upgrade| &upgrade.new_version);

                for (i, version) in versions.enumerate() {
                    let url_formatted = url_templates::format_url(&template, &version);
                    if i != 0 {
                        print_with_margin("", max_upgrade_name_len);
                    }
                    println!("{}", url_formatted);
                }
            }
            None => println!("{}", package.home_page_url),
        }
    }

    Ok(())
}

fn print_with_margin(str: &str, max_len: usize) {
    let spaces = (0..(max_len - str.len())).map(|_| " ").collect::<String>();
    print!(" {}{} ", spaces, str.bold().magenta());
}
