use chrono::NaiveDate;
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

use regex::Regex;
use std::io::prelude::*;

use crate::date_utils;
use crate::distribution;
use crate::distribution::Distribution::*;
use crate::package::{get_installed_packages_by_name, Package};

#[derive(Debug, PartialEq)]
pub struct Upgrade {
    pub date: NaiveDate,
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
                r"^\[(?P<date>\d{4}-\d{2}-\d{2})( |T)\d{2}:\d{2}(:\d\d-\d\d\d\d)?\] \[ALPM\] upgraded (?P<name>[^ ]*) \((\d:)?(?P<old>[.\d]+\d).* -> (\d:)?(?P<new>[.\d]+\d).*\)$")
                .unwrap();
        }

        let maybe_line_captures = UPGRADE_PARSE_REGEX.captures(s);
        maybe_line_captures
            // prevent duplicates from package updates causing duplicates
            .filter(|caps| caps["old"] != caps["new"])
            .map(|caps| Upgrade {
                date: NaiveDate::parse_from_str(&caps["date"], date_utils::NAIVE_DATE_FORMAT)
                    .unwrap(),
                package_name: caps["name"].to_string(),
                old_version: caps["old"].to_string(),
                new_version: caps["new"].to_string(),
            })
            .ok_or(ParseUpgradeError::Error)
    }
}

pub fn get_upgrades_since(since_time: NaiveDate) -> BTreeMap<Package, Vec<Upgrade>> {
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
    since_time: NaiveDate,
    installed_packages_by_name: &HashMap<String, Package>,
) -> HashMap<String, Vec<Upgrade>> {
    match distribution::current() {
        Arch => arch(since_time, installed_packages_by_name),
    }
}

/// Arch Linux
fn arch(
    since_time: NaiveDate,
    installed_packages_by_name: &HashMap<String, Package>,
) -> HashMap<String, Vec<Upgrade>> {
    let upgrades = BufReader::new(File::open("/var/log/pacman.log").unwrap())
        .lines()
        .filter_map(|result_str| result_str.ok().and_then(|s| Upgrade::from_str(&s).ok()))
        .skip_while(|upgrade| upgrade.date < since_time)
        .filter(|upgrade| installed_packages_by_name.contains_key(&upgrade.package_name));

    let mut accumulator = HashMap::new();
    for upgrade in upgrades {
        let vec = accumulator
            .entry(upgrade.package_name.clone())
            .or_insert(Vec::new());
        vec.push(upgrade);
    }

    accumulator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_linux_old_format() {
        assert_eq!(
            Upgrade::from_str(
                "[2019-08-18 14:59] [ALPM] upgraded linux (5.2.8.arch1-1 -> 5.2.9.arch1-1)"
            )
            .unwrap(),
            Upgrade {
                date: NaiveDate::from_ymd(2019, 08, 18),
                package_name: "linux".to_string(),
                old_version: "5.2.8".to_string(),
                new_version: "5.2.9".to_string()
            }
        )
    }

    #[test]
    fn parse_new_alpm_format() {
        assert_eq!(
            Upgrade::from_str(
                "[2020-06-24T13:14:18-0400] [ALPM] upgraded linux (5.7.4.arch1-1 -> 5.7.5.arch1-1)"
            )
            .unwrap(),
            Upgrade {
                date: NaiveDate::from_ymd(2020, 06, 24),
                package_name: "linux".to_string(),
                old_version: "5.7.4".to_string(),
                new_version: "5.7.5".to_string()
            }
        )
    }
}
