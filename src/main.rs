use clap::{App, Arg};
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::str::FromStr;

use std::process::Command;

use chrono::naive::NaiveDateTime;
use regex::Regex;

mod formatting;

#[derive(Debug)]
struct Upgrade {
    timestamp: NaiveDateTime,
    package_name: String,
    old_version: String,
    new_version: String,
}

#[derive(Debug)]
enum ParseUpgradeError {
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Package {
    name: String,
    home_page_url: String,
}

fn main() -> io::Result<()> {
    let exe_name = std::env::current_exe()?
        .file_name()
        .unwrap()
        .to_owned()
        .into_string()
        .unwrap();
    let matches = App::new(exe_name.clone())
        .author("Ratan Rai Sur <ratan.r.sur@gmail.com>")
        .about("What's new?")
        .arg(
            Arg::with_name("since")
                .long("since")
                .value_name("'YYYY-MM-DD HH:MM'")
                .help("The date from which to show updates")
                .takes_value(true),
        )
        .get_matches();
    let date_format = "%Y-%m-%d %H:%M";
    let mut cache_file = dirs::home_dir().unwrap();
    cache_file.push(".cache");
    cache_file.push(exe_name);
    let since_time = NaiveDateTime::parse_from_str(
        matches.value_of("since").unwrap_or(
            &fs::read_to_string(&cache_file).unwrap_or(String::from("2002-03-11 00:00")),
        ),
        date_format,
    )
    .unwrap();

    let current_briefing_time = chrono::offset::Local::now().naive_local();

    let upgrades_by_package = get_upgrades_since(since_time);

    let margin_width = upgrades_by_package
        .keys()
        .map(|package| package.name.len())
        .max()
        .unwrap_or(0)
        + 1;

    let mut home_page_group = Vec::new();
    let mut mono_page_group = Vec::new();
    let mut template_group = Vec::new();

    for (package, upgrades) in upgrades_by_package {
        if project_urls::TEMPLATES.contains_key(package.name.as_str()) {
            template_group.push((package, upgrades))
        } else if project_urls::MONO_PAGES.contains_key(package.name.as_str()) {
            let url = project_urls::MONO_PAGES.get(package.name.as_str()).unwrap();
            mono_page_group.push((package, url))
        } else {
            home_page_group.push(package)
        }
    }

    let home_page_outputs: Vec<String> = home_page_group
        .iter()
        .map(|package| (&package.name, vec![package.home_page_url.clone()]))
        .map(|(name, urls)| formatting::package_output(margin_width, &name, &urls))
        .collect();
    let template_outputs: Vec<String> = template_group
        .iter()
        .map(|(package, upgrades)| {
            (
                &package.name,
                upgrades
                    .iter()
                    .map(|upgrade| {
                        formatting::format_url(
                            project_urls::TEMPLATES.get(package.name.as_str()).unwrap(),
                            &upgrade.old_version,
                            &upgrade.new_version,
                        )
                    })
                    .collect(),
            )
        })
        .map(|(name, urls)| formatting::package_output(margin_width, &name, &urls))
        .collect();
    let mono_page_outputs = mono_page_group.iter().map(|(package, mono_page_url)| {
        formatting::package_output(
            margin_width,
            &package.name,
            &vec![String::from(**mono_page_url)],
        )
    });

    if !home_page_group.is_empty() {
        println!("");
        println!("{}", formatting::section_header(margin_width, "Homepages"));
        home_page_outputs.iter().for_each(|s| print!("{}", s));
    }
    if !template_group.is_empty() {
        println!("");
        println!(
            "{}",
            formatting::section_header(margin_width, "Release Notes")
        );
        template_outputs.iter().for_each(|s| print!("{}", s));
    }
    if !mono_page_group.is_empty() {
        mono_page_outputs.for_each(|s| print!("{}", s));
    }

    // don't write to cache file if since is used
    if !matches.is_present("since") {
        fs::write(
            cache_file,
            current_briefing_time.format(date_format).to_string(),
        )
        .expect("Something went wrong in updating the cache file.");
    }

    Ok(())
}

fn get_upgrades_since(since_time: NaiveDateTime) -> BTreeMap<Package, Vec<Upgrade>> {
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

fn get_installed_packages_by_name() -> HashMap<String, Package> {
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
            },
        );
    }
    packages
}
