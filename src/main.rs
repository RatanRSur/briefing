use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
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

fn string_to_upgrade(s: &str, regex: &Regex) -> Option<Upgrade> {
    let maybe_line_captures = regex.captures(s);
    maybe_line_captures.map(|caps| Upgrade {
        timestamp: NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M").unwrap(),
        package_name: caps[2].to_string(),
        old_version: caps[3].to_string(),
        new_version: caps[5].to_string(),
    })
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
struct Package {
    name: String,
    url_template: String,
}

fn main() -> io::Result<()> {
    let last_briefing =
        NaiveDateTime::parse_from_str("2018-12-01 00:00", "%Y-%m-%d %H:%M").unwrap();

    let release_notes_templates_map: HashMap<&str, &str> = url_templates::RELEASE_NOTES_TEMPLATES
        .iter()
        .cloned()
        .collect();

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
                    name: package_name,
                    url_template: release_notes_templates_map
                        .get(&captures[3])
                        .unwrap_or(&&captures_iter.next().unwrap()[3])
                        .to_string(),
                },
            );
        }
        packages
    };

    let upgrades_by_name: HashMap<String, Vec<Upgrade>> = {
        let f = BufReader::new(File::open("/var/log/pacman.log")?);

        let regex = Regex::new(r"^\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2})\] \[ALPM\] upgraded ([^ ]*) \((.+) -> (\d:)?([^-+]+).*\)$",).unwrap();

        let installed_package_names: HashSet<_> = installed_packages
            .iter()
            .map(|name_and_package| name_and_package.0)
            .cloned()
            .collect();

        let mut accumulator: HashMap<String, Vec<Upgrade>> = HashMap::new();
        let upgrades = f
            .lines()
            .filter_map(|result_str| result_str.ok().and_then(|s| string_to_upgrade(&s, &regex)))
            .skip_while(|upgrade| upgrade.timestamp < last_briefing)
            .filter(|upgrade| installed_package_names.contains(&upgrade.package_name));

        for upgrade in upgrades {
            let val = accumulator.get_mut(&upgrade.package_name);
            match val {
                Some(foo) => foo.push(upgrade),
                None => {
                    accumulator.insert(upgrade.package_name.clone(), vec![upgrade]);
                    ()
                }
            }
        }

        accumulator
    };

    for (package_name, upgrades) in upgrades_by_name {
        let url_template = installed_packages
            .get(&package_name)
            .map(|package| &package.url_template)
            .unwrap();
        let versions = upgrades.iter().map(|upgrade| &upgrade.new_version);
        println!("{}:", package_name);
        for version in versions {
            let url_formatted = url_templates::format_url(&url_template, &version);
            println!("\t{}", url_formatted);
        }
    }

    Ok(())
}
