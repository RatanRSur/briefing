use clap::{App, Arg};
use std::fs;
use std::io;
use std::process::exit;

use chrono::NaiveDate;

mod distribution;
mod formatting;
mod package;
mod upgrade;

fn main() -> io::Result<()> {
    let exe_name = std::env::current_exe()?
        .file_name()
        .unwrap()
        .to_owned()
        .into_string()
        .unwrap();
    let date_cmdline_format = "YYYY-MM-DD";
    let matches = App::new(exe_name.clone())
        .author("Ratan Rai Sur <ratan.r.sur@gmail.com>")
        .about("What's new?")
        .args(&[
            Arg::with_name("since")
                .long("since")
                .value_name(&date_cmdline_format)
                .help("The date from which to show updates")
                .takes_value(true),
            Arg::with_name("no-cache")
                .long("no-cache")
                .help("Don't update the cache file to record the time of this run")
                .takes_value(false),
        ])
        .get_matches();
    let naive_date_format = "%F";
    let mut cache_file = dirs::home_dir().unwrap();
    cache_file.push(".cache");
    cache_file.push(exe_name);
    let current_briefing_time = chrono::Local::today().naive_local();
    let since_time = matches
        .value_of("since")
        .map(|s| String::from(s))
        .or(fs::read_to_string(&cache_file).ok())
        .map(|date_string| {
            NaiveDate::parse_from_str(&date_string, naive_date_format).unwrap_or_else(|_| {
                println!("Invalid date format. Use {}", date_cmdline_format);
                exit(1)
            })
        })
        .unwrap_or(current_briefing_time);

    let upgrades_by_package = upgrade::get_upgrades_since(since_time);

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

    if !home_page_outputs.is_empty() {
        println!();
        println!("{}", formatting::section_header(margin_width, "Homepages"));
        home_page_outputs.iter().for_each(|s| print!("{}", s));
    }
    if !template_outputs.is_empty() {
        println!();
        println!(
            "{}",
            formatting::section_header(margin_width, "Release Notes")
        );
        template_outputs.iter().for_each(|s| print!("{}", s));
    }
    mono_page_outputs.for_each(|s| print!("{}", s));

    // don't write to cache file if since is used
    let write_to_cache = !(matches.is_present("since") || matches.is_present("no-cache"));
    if write_to_cache {
        fs::write(
            cache_file,
            current_briefing_time.format(naive_date_format).to_string(),
        )
        .expect("Something went wrong in updating the cache file.");
    }

    Ok(())
}
