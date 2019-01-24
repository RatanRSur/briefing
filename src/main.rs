use ansi_term::Style;
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
use strfmt::strfmt;

use std::process::Command;

use chrono::naive::NaiveDateTime;
use regex::Regex;

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
}

fn main() -> io::Result<()> {
    let last_briefing =
        NaiveDateTime::parse_from_str("2019-01-01 00:00", "%Y-%m-%d %H:%M").unwrap();

    let upgrades_by_package: BTreeMap<Package, Vec<Upgrade>> = {
        let installed_packages_by_name = get_installed_packages_by_name();
        let mut accumulator = BTreeMap::new();
        let upgrades_by_name = get_upgrades_by_name(last_briefing, &installed_packages_by_name);

        for (name, upgrades) in upgrades_by_name {
            accumulator.insert(
                installed_packages_by_name.get(&name).unwrap().to_owned(),
                upgrades,
            );
        }

        accumulator
    };

    let margin_width = upgrades_by_package
        .keys()
        .map(|package| package.name.len())
        .max()
        .unwrap_or(0);

    let mut home_page_group = Vec::new();
    let mut mono_page_group = Vec::new();
    let mut template_group = Vec::new();

    for (package, upgrades) in upgrades_by_package {
        if templates::RELEASE_NOTES_TEMPLATES.contains_key(package.name.as_str()) {
            template_group.push((package, upgrades))
        } else if templates::RELEASE_NOTES_MONO_PAGES.contains_key(package.name.as_str()) {
            let url = templates::RELEASE_NOTES_MONO_PAGES
                .get(package.name.as_str())
                .unwrap();
            mono_page_group.push((package, url))
        } else {
            home_page_group.push(package)
        }
    }

    let home_page_outputs: Vec<String> = home_page_group
        .iter()
        .map(|package| (&package.name, vec![package.home_page_url.clone()]))
        .map(|(name, urls)| package_output(margin_width, &name, &urls))
        .collect();
    let template_outputs: Vec<String> = template_group
        .iter()
        .map(|(package, upgrades)| {
            (
                &package.name,
                upgrades
                    .iter()
                    .map(|upgrade| {
                        format_url(
                            project_urls::TEMPLATES.get(package.name.as_str()).unwrap(),
                            &upgrade.new_version,
                        )
                    })
                    .collect(),
            )
        })
        .map(|(name, urls)| package_output(margin_width, &name, &urls))
        .collect();
    let mono_page_outputs = mono_page_group.iter().map(|(package, mono_page_url)| {
        package_output(
            margin_width,
            &package.name,
            &vec![String::from(**mono_page_url)],
        )
    });

    if home_page_group.len() > 0 {
        println!("");
        println!("{}", section_header(margin_width, "Homepages"));
        home_page_outputs.iter().for_each(|s| print!("{}", s));
    }
    if template_group.len() > 0 {
        println!("");
        println!("{}", section_header(margin_width, "Release Notes"));
        template_outputs.iter().for_each(|s| print!("{}", s));
    }
    if mono_page_group.len() > 0 {
        mono_page_outputs.for_each(|s| print!("{}", s));
    }

    Ok(())
}

fn section_header(margin_width: usize, header: &str) -> String {
    let mut buf = String::new();
    buf.push_str(&left_pad_to_width(margin_width, ""));
    buf.push(' ');
    buf.push_str(header);
    bold(buf)
}

fn package_output(margin_width: usize, package_name: &str, urls: &Vec<String>) -> String {
    let mut buf = String::new();
    buf.push_str(&bold(left_pad_to_width(margin_width, package_name)));
    for (i, url) in urls.iter().enumerate() {
        if i != 0 {
            buf.push_str(&left_pad_to_width(margin_width, ""));
        }
        buf.push(' ');
        buf.push_str(url);
        buf.push('\n');
    }
    buf
}

fn left_pad_to_width(width: usize, str: &str) -> String {
    let mut buf = String::new();
    buf.push(' ');
    (0..(width - str.len())).for_each(|_| buf.push(' '));
    buf.push_str(str);
    buf
}

fn bold(str: String) -> String {
    Style::new().bold().paint(str).to_string()
}

pub fn format_url(template: &str, version: &str) -> String {
    let format_args: HashMap<String, &str> = [(String::from("version"), version)]
        .iter()
        .cloned()
        .collect();

    strfmt(template, &format_args).unwrap()
}

fn get_upgrades_by_name(
    last_briefing: NaiveDateTime,
    installed_packages_by_name: &HashMap<String, Package>,
) -> HashMap<String, Vec<Upgrade>> {
    let f = BufReader::new(File::open("/var/log/pacman.log").unwrap());

    let mut accumulator = HashMap::new();
    let upgrades = f
        .lines()
        .filter_map(|result_str| result_str.ok().and_then(|s| Upgrade::from_str(&s).ok()))
        .skip_while(|upgrade| upgrade.timestamp < last_briefing)
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
