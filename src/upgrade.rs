use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

use chrono::naive::NaiveDateTime;
use regex::Regex;
use std::io::prelude::*;

use crate::package::{get_installed_packages_by_name, Package};

#[derive(Debug)]
pub struct Upgrade {
    pub timestamp: NaiveDateTime,
    pub package_name: String,
    pub old_version: String,
    pub new_version: String,
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
            static ref UPGRADE_PARSE_REGEX: Regex = Regex::new(
                r"^\[(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2})\] \[ALPM\] upgraded (?P<name>[^ ]*) \((\d:)?(?P<old>[^-+]+).* -> (\d:)?(?P<new>[^-+]+).*\)$",)
                .unwrap();
        }

        let maybe_line_captures = UPGRADE_PARSE_REGEX.captures(s);
        maybe_line_captures
            // prevent duplicates from package updates causing duplicates
            .filter(|caps| caps["old"] != caps["new"])
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

pub fn get_upgrades_since(since_time: NaiveDateTime) -> BTreeMap<Package, Vec<Upgrade>> {
    let installed_packages_by_name = get_installed_packages_by_name();
    let mut accumulator = BTreeMap::new();
    let upgrades_by_name = get_upgrades_by_name(since_time, &installed_packages_by_name);

    for (name, upgrades) in upgrades_by_name {
        accumulator.insert(
            installed_packages_by_name.get(&name).unwrap().to_owned(),
            upgrades,
        );
    }

    accumulator
}

fn get_upgrades_by_name(
    since_time: NaiveDateTime,
    installed_packages_by_name: &HashMap<String, Package>,
) -> HashMap<String, Vec<Upgrade>> {
    let f = BufReader::new(File::open("/var/log/pacman.log").unwrap());

    let mut accumulator = HashMap::new();
    let upgrades = f
        .lines()
        .filter_map(|result_str| result_str.ok().and_then(|s| Upgrade::from_str(&s).ok()))
        .skip_while(|upgrade| upgrade.timestamp < since_time)
        .filter(|upgrade| installed_packages_by_name.contains_key(&upgrade.package_name));

    for upgrade in upgrades {
        let vec = accumulator
            .entry(upgrade.package_name.clone())
            .or_insert(Vec::new());
        vec.push(upgrade);
    }

    accumulator
}
