use colored::*;
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::collections::HashMap;
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
            static ref upgrade_parse_regex: Regex = Regex::new(
                r"^\[(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2})\] \[ALPM\] upgraded (?P<name>[^ ]*) \((\d:)?(?P<old>[^-+]+).* -> (\d:)?(?P<new>[^-+]+).*\)$",)
                .unwrap();
        }

        let maybe_line_captures = upgrade_parse_regex.captures(s);
        maybe_line_captures
            .map(|caps| Upgrade {
                timestamp: NaiveDateTime::parse_from_str(&caps["timestamp"], "%Y-%m-%d %H:%M")
                    .unwrap(),
                package_name: caps["name"].to_string(),
                old_version: caps["old"].to_string(),
                new_version: caps["new"].to_string(),
            })
            .ok_or(ParseUpgradeError::Error)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Package {
    name: String,
    home_page_url: String,
    url_template: Option<&'static str>,
}

fn main() -> io::Result<()> {
    let last_briefing =
        NaiveDateTime::parse_from_str("2018-12-01 00:00", "%Y-%m-%d %H:%M").unwrap();

    let upgrades_by_package: BTreeMap<Package, Vec<Upgrade>> = {
        let installed_packages = {
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

            let regex = Regex::new(r"(^|\n)(Name|URL) +: (?P<value>.*)\n").unwrap();

            let mut packages = HashMap::new();
            let mut captures_iter = regex.captures_iter(&installed_packages_output);
            while let Some(captures) = captures_iter.next() {
                let package_name = String::from(&captures["value"]);
                packages.insert(
                    package_name.clone(),
                    Package {
                        name: package_name.clone(),
                        home_page_url: captures_iter.next().unwrap()["value"].to_string(),
                        url_template: url_templates::RELEASE_NOTES_TEMPLATES
                            .get(package_name.as_str())
                            .map(|&s| s),
                    },
                );
            }
            packages
        };

        let mut accumulator = BTreeMap::new();
        let upgrades_by_name = {
            let f = BufReader::new(File::open("/var/log/pacman.log")?);

            let mut accumulator = HashMap::new();
            let upgrades = f
                .lines()
                .filter_map(|result_str| result_str.ok().and_then(|s| Upgrade::from_str(&s).ok()))
                .skip_while(|upgrade| upgrade.timestamp < last_briefing)
                .filter(|upgrade| installed_packages.contains_key(&upgrade.package_name));

            for upgrade in upgrades {
                let vec = accumulator
                    .entry(upgrade.package_name.clone())
                    .or_insert(Vec::new());
                vec.push(upgrade);
            }

            accumulator
        };

        for (name, upgrades) in upgrades_by_name {
            accumulator.insert(installed_packages.get(&name).unwrap().to_owned(), upgrades);
        }

        accumulator
    };

    let margin_width = upgrades_by_package
        .keys()
        .map(|package| package.name.len())
        .max()
        .unwrap();

    for (package, upgrades) in upgrades_by_package {
        match package.url_template {
            Some(template) => {
                let formatted_urls = {
                    let versions = upgrades.iter().map(|upgrade| &upgrade.new_version);
                    versions
                        .map(|version| url_templates::format_url(&template, &version))
                        .collect()
                };
                print_package_block(margin_width, &package.name, &formatted_urls);
            }
            None => print_package_block(
                margin_width,
                &package.name,
                &vec![package.home_page_url.to_string()],
            ),
        }
    }

    Ok(())
}

fn print_package_block(margin_width: usize, package_name: &str, urls: &Vec<String>) {
    print_with_margin(margin_width, package_name);
    for (i, url) in urls.iter().enumerate() {
        if i != 0 {
            print_with_margin(margin_width, "");
        }
        println!("{}", url);
    }
}

fn print_with_margin(margin_width: usize, str: &str) {
    let spaces = (0..(margin_width - str.len()))
        .map(|_| " ")
        .collect::<String>();
    print!(" {}{} ", spaces, str.bold().magenta());
}
