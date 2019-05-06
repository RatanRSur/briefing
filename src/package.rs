use crate::distribution::Distribution;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Package {
    pub name: String,
    pub home_page_url: String,
}

pub fn get_installed_packages_by_name(distro: Distribution) -> HashMap<String, Package> {
    match distro {
        Arch => arch(),
    }
}

/// Distibution Specific Package Retrieval
fn arch() -> HashMap<String, Package> {
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
